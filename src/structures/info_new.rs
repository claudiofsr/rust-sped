use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};

use crate::{
    ALIQ_BASICA_COF,
    // CODIGO_DA_NATUREZA_BC, CFOP_VENDA_DE_IMOBILIZADO, CST_ALL, CST_CREDITO, CST_RECEITA_BRUTA,
    // Constantes e utilitários
    ALIQ_BASICA_PIS,
    DocsFiscais,
    EFDResult,
    SpedFile,
    SpedRecord,
    blocos::*, // Importa todos os structs de registros (Registro0000, C100, etc.)
};

// ============================================================================
// 1. Contexto Imutável (Dados Globais e Tabelas)
// ============================================================================

/// Mantém as tabelas de referência carregadas do Bloco 0.
/// É construído uma vez e passado como referência imutável para os processadores de blocos.
#[derive(Debug)]
pub struct SpedContext {
    pub arquivo_path: PathBuf,
    pub cnpj_base: String,
    pub pa: Option<NaiveDate>, // Período de Apuração
    pub messages: Vec<String>, // Mensagens de erro/aviso coletadas na construção

    // Tabelas de consulta (Lookups)
    pub participantes: HashMap<String, Registro0150>,
    pub produtos: HashMap<String, Registro0200>,
    pub contas: HashMap<String, Registro0500>,
    pub naturezas: HashMap<String, Registro0400>,
    pub estabelecimentos: HashMap<String, Registro0140>,
    pub info_complementar: HashMap<String, Registro0450>,

    // Cache de Nomes
    pub nome_do_cnpj: BTreeMap<String, String>,
    pub nome_do_cpf: BTreeMap<String, String>,
}

impl SpedContext {
    /// Constrói o contexto lendo apenas o Bloco 0 do arquivo SPED.
    pub fn new(file: &SpedFile, path: &Path) -> EFDResult<Self> {
        let mut ctx = SpedContext {
            arquivo_path: path.to_path_buf(),
            cnpj_base: String::new(),
            pa: None,
            messages: Vec::new(),
            participantes: HashMap::new(),
            produtos: HashMap::new(),
            contas: HashMap::new(),
            naturezas: HashMap::new(),
            estabelecimentos: HashMap::new(),
            info_complementar: HashMap::new(),
            nome_do_cnpj: BTreeMap::new(),
            nome_do_cpf: BTreeMap::new(),
        };

        let bloco_0 = match file.obter_bloco_option('0') {
            Some(recs) => recs,
            None => return Ok(ctx),
        };

        // Itera sobre os registros do Bloco 0 para popular as tabelas
        for record in bloco_0 {
            // Usa o SpedRecord::downcast_ref via helper interno ou match direto se exposto
            // Assumindo a estrutura do model.rs fornecido:
            if let SpedRecord::Generic(inner) = record {
                // Pattern Matching no nome do registro para performance e segurança
                match inner.registro_name() {
                    "0000" => {
                        if let Ok(r) = record.downcast_ref::<Registro0000>() {
                            ctx.pa = Some(r.dt_ini);
                            ctx.cnpj_base = r.get_cnpj_base();
                        }
                    }
                    "0140" => {
                        if let Ok(r) = record.downcast_ref::<Registro0140>()
                            && let Some(cnpj) = &r.cnpj {
                                ctx.estabelecimentos.insert(cnpj.clone(), r.clone());
                            }
                    }
                    "0150" => {
                        if let Ok(r) = record.downcast_ref::<Registro0150>() {
                            if let Some(cod) = r.cod_part.clone() {
                                ctx.participantes.insert(cod, r.clone());
                            }
                            if let Some(nome) = &r.nome {
                                if let Some(cnpj) = &r.cnpj {
                                    ctx.nome_do_cnpj.insert(cnpj.clone(), nome.clone());
                                }
                                if let Some(cpf) = &r.cpf {
                                    ctx.nome_do_cpf.insert(cpf.clone(), nome.clone());
                                }
                            }
                        }
                    }
                    "0200" => {
                        if let Ok(r) = record.downcast_ref::<Registro0200>()
                            && let Some(cod_item) = &r.cod_item {
                                ctx.produtos.insert(cod_item.clone(), r.clone());
                            }
                    }
                    "0400" => {
                        if let Ok(r) = record.downcast_ref::<Registro0400>()
                            && let Some(cod_nat) = &r.cod_nat {
                            ctx.naturezas.insert(cod_nat.clone(), r.clone());
                            }
                    }
                    "0450" => {
                        if let Ok(r) = record.downcast_ref::<Registro0450>()
                            && let Some(cod_inf) = &r.cod_inf {
                            ctx.info_complementar.insert(cod_inf.clone(), r.clone());
                            }
                    }
                    "0500" => {
                        if let Ok(r) = record.downcast_ref::<Registro0500>()
                            && let Some(cod_cta) = &r.cod_cta {
                            ctx.contas.insert(cod_cta.clone(), r.clone());
                            }
                    }
                    _ => {}
                }
            }
        }

        Ok(ctx)
    }

    /// Obtém o nome do participante baseado no CNPJ/CPF se não encontrado pelo código
    pub fn obter_nome_participante(&self, cnpj: Option<&str>, cpf: Option<&str>) -> String {
        if let Some(c) = cnpj {
            if let Some(nome) = self.nome_do_cnpj.get(c) {
                return nome.clone();
            }
            // Lógica "CNPJ Base" simplificada aqui se necessário
            if c.len() >= 8 {
                let cnpj_base = c[0..8].to_string();
                // A lógica complexa de "most frequent value" pode ser aplicada aqui se desejar,
                // mas por simplicidade retornamos vazio ou busca exata.
                return cnpj_base
            }
        }
        if let Some(c) = cpf
            && let Some(nome) = self.nome_do_cpf.get(c) {
                return nome.clone();
            }
        String::new()
    }
}

// ============================================================================
// 2. Estado do Processador de Bloco (Substitui info global mutável)
// ============================================================================

/// Mantém o estado hierárquico (Pai -> Filho) e caches de correlação
/// durante a iteração linear de um bloco.
#[derive(Default)]
struct BlockState<'a> {
    // Parents
    a100: Option<&'a RegistroA100>,
    c100: Option<&'a RegistroC100>,
    c180: Option<&'a RegistroC180>,
    c190: Option<&'a RegistroC190>,
    c380: Option<&'a RegistroC380>,
    c395: Option<&'a RegistroC395>,
    c400: Option<&'a RegistroC400>, // Nota: C400 e C405 são pais de C481/C485
    c405: Option<&'a RegistroC405>,
    c490: Option<&'a RegistroC490>,
    c500: Option<&'a RegistroC500>,
    c600: Option<&'a RegistroC600>,
    c860: Option<&'a RegistroC860>,
    d100: Option<&'a RegistroD100>,
    d200: Option<&'a RegistroD200>,
    d500: Option<&'a RegistroD500>,
    d600: Option<&'a RegistroD600>,
    m100: Option<&'a RegistroM100>,
    m500: Option<&'a RegistroM500>,

    // Correlation Cache (Key -> [Aliq, Valor])
    // Usado para correlacionar registros como C191 (PIS) e C195 (COFINS)
    // Chave pode ser fraca (CST + Valor) ou forte (CST + Valor + CFOP + Part)
    correlation_cache: HashMap<String, (f64, f64)>,

    // Controle auxiliar para registros que "inserem linhas" logicamente
    linhas_inseridas: Vec<usize>,
}

impl<'a> BlockState<'a> {
    fn clear_correlation(&mut self) {
        self.correlation_cache.clear();
    }

    fn clear_linhas(&mut self) {
        self.linhas_inseridas.clear();
    }
}

// ============================================================================
// 3. Motor de Processamento (Substitui Dispatch Table)
// ============================================================================

/// Processa um bloco inteiro e retorna o vetor de DocsFiscais resultante.
pub fn process_block_lines(bloco: char, file: &SpedFile, ctx: &SpedContext) -> Vec<DocsFiscais> {
    let records = match file.obter_bloco_option(bloco) {
        Some(list) => list,
        None => return Vec::new(),
    };

    let mut docs = Vec::with_capacity(records.len());
    let mut state = BlockState::default();

    // Itera sequencialmente garantindo ordem de registros
    for record in records {
        if let SpedRecord::Generic(inner) = record {
            // Dispatch baseado no nome do registro
            match inner.registro_name() {
                // --- BLOCO A ---
                "A100" => {
                    if let Ok(r) = record.downcast_ref::<RegistroA100>() {
                        state.a100 = Some(r);
                    }
                }
                "A170" => {
                    if let Ok(r) = record.downcast_ref::<RegistroA170>()
                        && let Some(parent) = state.a100 {
                            docs.push(mappers::from_a170(r, parent, ctx));
                        }
                }

                // --- BLOCO C ---
                "C100" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC100>() {
                        state.c100 = Some(r);
                    }
                }
                "C170" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC170>()
                        && let Some(parent) = state.c100 {
                            docs.push(mappers::from_c170(r, parent, ctx));
                        }
                }
                "C175" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC175>()
                        && let Some(parent) = state.c100 {
                            docs.push(mappers::from_c175(r, parent, ctx));
                        }
                }

                // C180 - C188 (Visão Consolidada)
                "C180" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC180>() {
                        state.c180 = Some(r);
                        state.clear_correlation();
                    }
                }
                "C181" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC181>() {
                        // Armazena correlação PIS
                        helpers::store_correlation_pis(
                            &mut state.correlation_cache,
                            r.cst_pis.clone(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            r.cfop.as_deref(),
                            None,
                        );
                    }
                }
                "C185" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC185>()
                        && let Some(parent) = state.c180 {
                            docs.push(mappers::from_c185(r, parent, &state.correlation_cache, ctx));
                        }
                }

                // C190 - C199 (Analítico)
                "C190" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC190>() {
                        state.c190 = Some(r);
                        state.clear_correlation();
                        state.clear_linhas();
                    }
                }
                "C191" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC191>() {
                        helpers::store_correlation_pis(
                            &mut state.correlation_cache,
                            r.cst_pis.clone(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            r.cfop.as_deref(),
                            r.cnpj_cpf_part.as_deref(),
                        );
                    }
                }
                "C195" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC195>()
                        && let Some(parent) = state.c190 {
                            // Processa C195 aplicando correlação do C191
                            let doc = mappers::from_c195(r, parent, &state.correlation_cache, ctx);
                            docs.push(doc);
                            // Salva índice para possível atualização pelo C199 (filho posterior)
                            state.linhas_inseridas.push(docs.len() - 1);
                        }
                }
                "C199" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC199>() {
                        // C199 complementa C195. Como já inserimos o C195 em `docs`, precisamos atualizar a última entrada
                        // se ela corresponder ao pai deste registro.
                        // Lógica simplificada: Atualiza o último Docs inserido se compatível.
                        if let Some(&last_idx) = state.linhas_inseridas.last()
                            && let Some(doc) = docs.get_mut(last_idx) {
                                mappers::update_with_c199(doc, r);
                            }
                    }
                }

                // Outros grupos C
                "C380" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC380>() {
                    state.c380 = Some(r);
                    state.clear_correlation();
                    }
                }
                "C381" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC381>() {
                        helpers::store_correlation_pis(
                            &mut state.correlation_cache,
                            r.cst_pis.clone(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            None,
                            None,
                        );
                    }
                }
                "C385" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC385>()
                        && let Some(p) = state.c380 {
                            docs.push(mappers::from_c385(r, p, &state.correlation_cache, ctx));
                        }
                }

                "C395" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC395>() {
                    state.c395 = Some(r);
                    }
                }
                "C396" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC396>()
                        && let Some(p) = state.c395 {
                            docs.push(mappers::from_c396(r, p, ctx));
                        }
                }

                "C400" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC400>() {
                    state.c400 = Some(r);
                    state.clear_correlation();
                    }
                }
                "C405" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC405>() {
                    state.c405 = Some(r);
                    }
                }
                "C481" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC481>() {
                        helpers::store_correlation_pis(
                            &mut state.correlation_cache,
                            r.cst_pis.clone(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            None,
                            None,
                        );
                    }
                }
                "C485" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC485>()
                        && let Some(p400) = state.c400
                            && let Some(p405) = state.c405 {
                                // C485 precisa de C400 e C405
                                docs.push(mappers::from_c485(
                                    r,
                                    p400,
                                    p405,
                                    &state.correlation_cache,
                                    ctx,
                                ));
                            }
                }

                "C490" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC490>() {
                    state.c490 = Some(r);
                    state.clear_correlation();
                    }
                }
                "C491" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC491>() {
                        helpers::store_correlation_pis(
                            &mut state.correlation_cache,
                            r.cst_pis.clone(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            None,
                            None,
                        );
                    }
                }
                "C495" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC495>()
                        && let Some(p) = state.c490 {
                            docs.push(mappers::from_c495(r, p, &state.correlation_cache, ctx));
                        }
                }

                "C500" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC500>() {
                    state.c500 = Some(r);
                    state.clear_correlation();
                    }
                }
                "C501" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC501>() {
                        helpers::store_correlation_pis(
                            &mut state.correlation_cache,
                            r.cst_pis.clone(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            None,
                            None,
                        );
                    }
                }
                "C505" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC505>()
                        && let Some(p) = state.c500 {
                            docs.push(mappers::from_c505(r, p, &state.correlation_cache, ctx));
                        }
                }

                "C600" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC600>() {
                    state.c600 = Some(r);
                    state.clear_correlation();
                    }
                }
                "C601" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC601>() {
                        helpers::store_correlation_pis(
                            &mut state.correlation_cache,
                            r.cst_pis.clone(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            None,
                            None,
                        );
                    }
                }
                "C605" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC605>()
                        && let Some(p) = state.c600 {
                            docs.push(mappers::from_c605(r, p, &state.correlation_cache, ctx));
                        }
                }

                "C860" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC860>() {
                    state.c860 = Some(r);
                    }
                }
                "C870" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC870>()
                        && let Some(p) = state.c860 {
                            docs.push(mappers::from_c870(r, p, ctx));
                        }
                }
                "C880" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC880>() {
                        docs.push(mappers::from_c880(r, ctx));
                    }
                }

                // --- BLOCO D ---
                "D100" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD100>() {
                    state.d100 = Some(r);
                    state.clear_correlation();
                    }
                }
                "D101" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD101>() {
                        helpers::store_correlation_pis(
                            &mut state.correlation_cache,
                            r.cst_pis.clone(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            None,
                            None,
                        );
                    }
                }
                "D105" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD105>()
                        && let Some(p) = state.d100 {
                            docs.push(mappers::from_d105(r, p, &state.correlation_cache, ctx));
                        }
                }

                "D200" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD200>() {
                    state.d200 = Some(r);
                    state.clear_correlation();
                    }
                }
                "D201" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD201>() {
                        // D201 é um registro completo, mas pode ser correlacionado internamente
                        // Aqui tratamos como item direto
                        docs.push(mappers::from_d201(r, state.d200, ctx));
                    }
                }
                "D205" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD205>()
                        && let Some(p) = state.d200 {
                            docs.push(mappers::from_d205(r, p, ctx));
                        }
                }

                "D300" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD300>() {
                        docs.push(mappers::from_d300(r, ctx));
                    }
                }
                "D350" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD350>() {
                        docs.push(mappers::from_d350(r, ctx));
                    }
                }

                "D500" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD500>() {
                    state.d500 = Some(r);
                    state.clear_correlation();
                    }
                }
                "D501" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD501>() {
                        helpers::store_correlation_pis(
                            &mut state.correlation_cache,
                            r.cst_pis.clone(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            None,
                            None,
                        );
                    }
                }
                "D505" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD505>()
                        && let Some(p) = state.d500 {
                            docs.push(mappers::from_d505(r, p, &state.correlation_cache, ctx));
                        }
                }

                "D600" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD600>() {
                    state.d600 = Some(r);
                    state.clear_correlation();
                    }
                }
                "D601" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD601>() {
                        helpers::store_correlation_pis(
                            &mut state.correlation_cache,
                            r.cst_pis.clone(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            None,
                            None,
                        );
                    }
                }
                "D605" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD605>()
                        && let Some(p) = state.d600 {
                            docs.push(mappers::from_d605(r, p, &state.correlation_cache, ctx));
                        }
                }

                // --- BLOCO F ---
                "F100" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF100>() {
                        docs.push(mappers::from_f100(r, ctx));
                    }
                }
                "F120" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF120>() {
                        docs.push(mappers::from_f120(r, ctx));
                    }
                }
                "F130" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF130>() {
                        docs.push(mappers::from_f130(r, ctx));
                    }
                }
                "F150" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF150>() {
                        docs.push(mappers::from_f150(r, ctx));
                    }
                }
                "F200" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF200>() {
                        docs.push(mappers::from_f200(r, ctx));
                    }
                }
                "F205" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF205>() {
                        docs.push(mappers::from_f205(r, ctx));
                    }
                }
                "F210" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF210>() {
                        docs.push(mappers::from_f210(r, ctx));
                    }
                }
                "F500" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF500>() {
                        docs.push(mappers::from_f500(r, ctx));
                    }
                }
                "F510" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF510>() {
                        docs.push(mappers::from_f510(r, ctx));
                    }
                }
                "F550" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF550>() {
                        docs.push(mappers::from_f550(r, ctx));
                    }
                }
                "F560" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF560>() {
                        docs.push(mappers::from_f560(r, ctx));
                    }
                }

                // --- BLOCO I ---
                "I100" => {
                    if let Ok(r) = record.downcast_ref::<RegistroI100>() {
                        docs.push(mappers::from_i100(r, ctx));
                    }
                }

                // --- BLOCO M ---
                "M100" => {
                    if let Ok(r) = record.downcast_ref::<RegistroM100>() {
                    state.m100 = Some(r);
                    docs.extend(mappers::from_m100(state.m100.unwrap(), ctx));
                    }
                }
                "M105" => {
                    if let Ok(_r) = record.downcast_ref::<RegistroM105>() {
                        // M105 detalha M100, lógica de correlação se necessário
                    }
                }
                "M500" => {
                    if let Ok(r) = record.downcast_ref::<RegistroM500>() {
                    state.m500 = Some(r);
                    docs.extend(mappers::from_m500(state.m500.unwrap(), ctx));
                    }
                }
                "M505" => {
                    if let Ok(r) = record.downcast_ref::<RegistroM505>()
                        && let Some(p) = state.m500 {
                            docs.push(mappers::from_m505(r, p, ctx));
                        }
                }

                // --- BLOCO 1 ---
                "1100" => {
                    if let Ok(r) = record.downcast_ref::<Registro1100>()
                        && let Some(d) = mappers::from_1100(r, ctx) {
                            docs.push(d);
                        }
                }
                "1500" => {
                    if let Ok(r) = record.downcast_ref::<Registro1500>()
                        && let Some(d) = mappers::from_1500(r, ctx) {
                            docs.push(d);
                        }
                }

                _ => {}
            }
        }
    }

    docs
}

// ============================================================================
// 4. Mappers (Funções de Transformação)
// ============================================================================

mod mappers {
    use super::*;
    use super::helpers::*;

    pub fn from_a170(reg: &RegistroA170, parent: &RegistroA100, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);

        // Pai A100
        doc.chave_doc = parent.chv_nfse.clone().unwrap_or_default();
        doc.data_emissao = parent.dt_doc;
        doc.data_lancamento = parent.dt_exe_serv;

        // A170
        doc.valor_item = dec_to_f64(reg.vl_item);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_pis = dec_to_f64(reg.vl_pis);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);
        doc.descr_item = reg.descr_compl.clone().unwrap_or_default();

        apply_common_rules(&mut doc, ctx, None, None);
        doc
    }

    pub fn from_c170(reg: &RegistroC170, parent: &RegistroC100, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);

        // Pai C100
        doc.chave_doc = parent.chv_nfe.clone().unwrap_or_default();
        doc.num_doc = parse_usize_opt(&parent.num_doc);
        doc.data_emissao = parent.dt_doc;
        doc.data_lancamento = parent.dt_e_s;
        doc.modelo_doc_fiscal = parent.cod_mod.clone().unwrap_or_default();

        // Item C170
        doc.num_item = parse_u32_opt(&reg.num_item);
        doc.cfop = parse_u16_opt(&reg.cfop);
        doc.cst = parse_u16_opt(&reg.cst_cofins);
        doc.cod_ncm = get_ncm(ctx, &reg.cod_item);
        doc.tipo_item = get_tipo_item(ctx, &reg.cod_item);
        doc.descr_item = get_descr_item(ctx, &reg.cod_item);
        doc.nat_operacao = get_nat_operacao(ctx, &reg.cod_nat);
        doc.nome_da_conta = get_conta_contabil(ctx, &reg.cod_cta);

        doc.valor_item = dec_to_f64(reg.vl_item);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_pis = dec_to_f64(reg.vl_pis);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.valor_bc_icms = dec_to_f64(reg.vl_bc_icms);
        doc.aliq_icms = dec_to_f64(reg.aliq_icms);
        doc.valor_icms = dec_to_f64(reg.vl_icms);

        enrich_participant(&mut doc, ctx, &parent.cod_part);
        apply_common_rules(&mut doc, ctx, reg.aliq_pis, reg.aliq_cofins);
        doc
    }

    pub fn from_c175(reg: &RegistroC175, parent: &RegistroC100, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);

        doc.chave_doc = parent.chv_nfe.clone().unwrap_or_default();
        doc.data_emissao = parent.dt_doc;
        doc.data_lancamento = parent.dt_e_s;

        doc.valor_item = dec_to_f64(reg.vl_opr);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_pis = dec_to_f64(reg.vl_pis);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);

        doc.cst = parse_u16_opt(&reg.cst_cofins);
        doc.cfop = parse_u16_opt(&reg.cfop);
        doc.complementar = reg.info_compl.clone().unwrap_or_default();

        apply_common_rules(&mut doc, ctx, reg.aliq_pis, reg.aliq_cofins);
        doc
    }

    pub fn from_c185(
        reg: &RegistroC185,
        parent: &RegistroC180,
        cache: &HashMap<String, (f64, f64)>,
        ctx: &SpedContext,
    ) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = parent.dt_doc_ini; // C180 é consolidado por período

        doc.valor_item = dec_to_f64(reg.vl_item);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);
        doc.cfop = parse_u16_opt(&reg.cfop);
        //doc.cod_ncm = get_ncm(ctx, &reg.cod_item);

        correlate_pis(
            &mut doc,
            cache,
            reg.vl_item,
            reg.cst_cofins.as_deref(),
            reg.cfop.as_deref(),
            None,
        );
        apply_common_rules(&mut doc, ctx, None, reg.aliq_cofins);
        doc
    }

    pub fn from_c195(
        reg: &RegistroC195,
        parent: &RegistroC190,
        cache: &HashMap<String, (f64, f64)>,
        ctx: &SpedContext,
    ) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = parent.dt_ref_ini;

        doc.valor_item = dec_to_f64(reg.vl_item);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);
        doc.cfop = parse_u16_opt(&reg.cfop);
        //doc.complementar = reg.obs.clone().unwrap_or_default();

        enrich_participant(&mut doc, ctx, &reg.cnpj_cpf_part);
        correlate_pis(
            &mut doc,
            cache,
            reg.vl_item,
            reg.cst_cofins.as_deref(),
            reg.cfop.as_deref(),
            reg.cnpj_cpf_part.as_deref(),
        );
        apply_common_rules(&mut doc, ctx, None, reg.aliq_cofins);
        doc
    }

    pub fn update_with_c199(doc: &mut DocsFiscais, reg: &RegistroC199) {
        // Adiciona informação de importação
        if let Some(num) = &reg.num_doc_imp {
            doc.complementar = format!("{} Num Doc Imp: {}", doc.complementar, num);
        }
    }

    pub fn from_c385(
        reg: &RegistroC385,
        parent: &RegistroC380,
        cache: &HashMap<String, (f64, f64)>,
        ctx: &SpedContext,
    ) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = parent.dt_doc_ini;
        doc.valor_item = dec_to_f64(reg.vl_item);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);

        correlate_pis(&mut doc, cache, reg.vl_item, None, None, None); // C381/C385 não tem CFOP/Part nos filhos geralmente
        apply_common_rules(&mut doc, ctx, None, reg.aliq_cofins);
        doc
    }

    pub fn from_c396(reg: &RegistroC396, parent: &RegistroC395, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = parent.dt_doc;

        doc.valor_item = dec_to_f64(reg.vl_item);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_pis = dec_to_f64(reg.vl_pis);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);
        //doc.cfop = parse_u16_opt(&parent.cfop); // CFOP no pai

        enrich_participant(&mut doc, ctx, &parent.cod_part);
        apply_common_rules(&mut doc, ctx, reg.aliq_pis, reg.aliq_cofins);
        doc
    }

    pub fn from_c485(
        reg: &RegistroC485,
        _p400: &RegistroC400,
        p405: &RegistroC405,
        cache: &HashMap<String, (f64, f64)>,
        ctx: &SpedContext,
    ) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = p405.dt_doc;
        doc.valor_item = dec_to_f64(reg.vl_item);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);
        //doc.cfop = parse_u16_opt(&reg.cfop);

        correlate_pis(
            &mut doc,
            cache,
            reg.vl_item,
            reg.cst_cofins.as_deref(),
            None,
            None,
        );
        apply_common_rules(&mut doc, ctx, None, reg.aliq_cofins);
        doc
    }

    pub fn from_c495(
        reg: &RegistroC495,
        parent: &RegistroC490,
        cache: &HashMap<String, (f64, f64)>,
        ctx: &SpedContext,
    ) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = parent.dt_doc_ini;
        doc.valor_item = dec_to_f64(reg.vl_item);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);
        doc.cfop = parse_u16_opt(&reg.cfop);

        correlate_pis(
            &mut doc,
            cache,
            reg.vl_item,
            reg.cst_cofins.as_deref(),
            reg.cfop.as_deref(),
            None,
        );
        apply_common_rules(&mut doc, ctx, None, reg.aliq_cofins);
        doc
    }

    pub fn from_c505(
        reg: &RegistroC505,
        parent: &RegistroC500,
        cache: &HashMap<String, (f64, f64)>,
        ctx: &SpedContext,
    ) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = parent.dt_doc; // Ou DT_E_S
        doc.data_lancamento = parent.dt_ent;
        doc.chave_doc = parent.chv_doce.clone().unwrap_or_default();

        doc.valor_item = dec_to_f64(reg.vl_item);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);
        //doc.cfop = parse_u16_opt(&reg.cfop);

        enrich_participant(&mut doc, ctx, &parent.cod_part);
        correlate_pis(
            &mut doc,
            cache,
            reg.vl_item,
            reg.cst_cofins.as_deref(),
            None,
            None,
        );
        apply_common_rules(&mut doc, ctx, None, reg.aliq_cofins);
        doc
    }

    pub fn from_c605(
        reg: &RegistroC605,
        _parent: &RegistroC600,
        cache: &HashMap<String, (f64, f64)>,
        ctx: &SpedContext,
    ) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.valor_item = dec_to_f64(reg.vl_item);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);

        correlate_pis(
            &mut doc,
            cache,
            reg.vl_item,
            reg.cst_cofins.as_deref(),
            None,
            None,
        );
        apply_common_rules(&mut doc, ctx, None, reg.aliq_cofins);
        doc
    }

    pub fn from_c870(reg: &RegistroC870, parent: &RegistroC860, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = parent.dt_doc;
        doc.valor_item = dec_to_f64(reg.vl_item);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_pis = dec_to_f64(reg.vl_pis);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);
        doc.cfop = parse_u16_opt(&reg.cfop);

        apply_common_rules(&mut doc, ctx, reg.aliq_pis, reg.aliq_cofins);
        doc
    }

    pub fn from_c880(reg: &RegistroC880, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.valor_item = dec_to_f64(reg.vl_item);
        //doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        //doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        //doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_pis = dec_to_f64(reg.vl_pis);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);
        doc.cfop = parse_u16_opt(&reg.cfop);

        apply_common_rules(&mut doc, ctx, None, None);
        doc
    }

    pub fn from_d105(
        reg: &RegistroD105,
        parent: &RegistroD100,
        cache: &HashMap<String, (f64, f64)>,
        ctx: &SpedContext,
    ) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = parent.dt_a_p; // Data Aquisição/Prestação
        doc.chave_doc = parent.chv_cte.clone().unwrap_or_default();

        doc.valor_item = dec_to_f64(reg.vl_item);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);

        enrich_participant(&mut doc, ctx, &parent.cod_part);
        correlate_pis(
            &mut doc,
            cache,
            reg.vl_item,
            reg.cst_cofins.as_deref(),
            None,
            None,
        );
        apply_common_rules(&mut doc, ctx, None, reg.aliq_cofins);
        doc
    }

    pub fn from_d201(
        reg: &RegistroD201,
        parent: Option<&RegistroD200>,
        ctx: &SpedContext,
    ) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        if let Some(p) = parent {
            doc.data_emissao = p.dt_ref;
        }

        doc.valor_item = dec_to_f64(reg.vl_item);
        //doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        //doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_pis = dec_to_f64(reg.vl_pis);
        //doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        //doc.cst = parse_u16_opt(&reg.cst_cofins);

        apply_common_rules(&mut doc, ctx, reg.aliq_pis, None);
        doc
    }

    pub fn from_d205(reg: &RegistroD205, parent: &RegistroD200, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = parent.dt_ref;
        doc.num_doc = parent.num_doc_ini.as_ref().and_then(|s| s.parse().ok());

        doc.valor_item = dec_to_f64(reg.vl_item);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        //doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        //doc.valor_pis = dec_to_f64(reg.vl_pis);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);

        apply_common_rules(&mut doc, ctx, None, reg.aliq_cofins);
        doc
    }

    pub fn from_d300(reg: &RegistroD300, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = reg.dt_ref;
        doc.valor_item = dec_to_f64(reg.vl_doc);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_pis = dec_to_f64(reg.vl_pis);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);

        apply_common_rules(&mut doc, ctx, reg.aliq_pis, reg.aliq_cofins);
        doc
    }

    pub fn from_d350(reg: &RegistroD350, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = reg.dt_doc;
        doc.valor_item = dec_to_f64(reg.vl_brt);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_pis = dec_to_f64(reg.vl_pis);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);

        apply_common_rules(&mut doc, ctx, reg.aliq_pis, reg.aliq_cofins);
        doc
    }

    pub fn from_d505(
        reg: &RegistroD505,
        parent: &RegistroD500,
        cache: &HashMap<String, (f64, f64)>,
        ctx: &SpedContext,
    ) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = parent.dt_a_p;
        doc.valor_item = dec_to_f64(reg.vl_item);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);

        enrich_participant(&mut doc, ctx, &parent.cod_part);
        correlate_pis(
            &mut doc,
            cache,
            reg.vl_item,
            reg.cst_cofins.as_deref(),
            None,
            None,
        );
        apply_common_rules(&mut doc, ctx, None, reg.aliq_cofins);
        doc
    }

    pub fn from_d605(
        reg: &RegistroD605,
        parent: &RegistroD600,
        cache: &HashMap<String, (f64, f64)>,
        ctx: &SpedContext,
    ) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = parent.dt_doc_ini;
        doc.valor_item = dec_to_f64(reg.vl_item);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);

        //enrich_participant(&mut doc, ctx, &parent.cod_part);
        correlate_pis(
            &mut doc,
            cache,
            reg.vl_item,
            reg.cst_cofins.as_deref(),
            None,
            None,
        );
        apply_common_rules(&mut doc, ctx, None, reg.aliq_cofins);
        doc
    }

    // --- BLOCO F Mappers ---

    pub fn from_f100(reg: &RegistroF100, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = reg.dt_oper;
        doc.valor_item = dec_to_f64(reg.vl_oper);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_pis = dec_to_f64(reg.vl_pis);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);
        doc.descr_item = reg.desc_doc_oper.clone().unwrap_or_default();

        enrich_participant(&mut doc, ctx, &reg.cod_part);
        apply_common_rules(&mut doc, ctx, reg.aliq_pis, reg.aliq_cofins);
        doc
    }

    pub fn from_f120(reg: &RegistroF120, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = ctx.pa; // Registro mensal
        doc.valor_item = dec_to_f64(reg.vl_oper_dep);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_pis = dec_to_f64(reg.vl_pis);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);
        doc.descr_item = reg.desc_bem_imob.clone().unwrap_or_default();

        apply_common_rules(&mut doc, ctx, reg.aliq_pis, reg.aliq_cofins);
        doc
    }

    pub fn from_f130(reg: &RegistroF130, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = ctx.pa;
        doc.valor_item = dec_to_f64(reg.vl_bc_cofins); // Valor Aquisição / Parcelas? Usando BC conforme regra antiga
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_pis = dec_to_f64(reg.vl_pis);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);
        doc.descr_item = reg.desc_bem_imob.clone().unwrap_or_default();

        //enrich_participant(&mut doc, ctx, &reg.cod_part);
        apply_common_rules(&mut doc, ctx, reg.aliq_pis, reg.aliq_cofins);
        doc
    }

    pub fn from_f150(reg: &RegistroF150, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = ctx.pa;
        doc.valor_item = dec_to_f64(reg.vl_tot_est);
        //doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        //doc.valor_pis = dec_to_f64(reg.vl_pis);
        //doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);
        doc.descr_item = reg.desc_est.clone().unwrap_or_default();

        //enrich_participant(&mut doc, ctx, &reg.cod_part);
        apply_common_rules(&mut doc, ctx, reg.aliq_pis, reg.aliq_cofins);
        doc
    }

    pub fn from_f200(reg: &RegistroF200, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = reg.dt_oper;
        doc.valor_item = dec_to_f64(reg.vl_tot_rec);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_pis = dec_to_f64(reg.vl_pis);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);
        doc.complementar = reg.inf_comp.clone().unwrap_or_default();

        //enrich_participant(&mut doc, ctx, &reg.cod_part); // Se existir campo
        apply_common_rules(&mut doc, ctx, reg.aliq_pis, reg.aliq_cofins);
        doc
    }

    pub fn from_f205(reg: &RegistroF205, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = ctx.pa;
        doc.valor_item = dec_to_f64(reg.vl_cus_inc_per_esc);
        //doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        //doc.valor_pis = dec_to_f64(reg.vl_pis);
        //doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);
        //doc.complementar = reg.inf_comp.clone().unwrap_or_default();

        apply_common_rules(&mut doc, ctx, reg.aliq_pis, reg.aliq_cofins);
        doc
    }

    pub fn from_f210(reg: &RegistroF210, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = ctx.pa;
        doc.valor_item = dec_to_f64(reg.vl_cus_orc);
        //doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        //doc.valor_pis = dec_to_f64(reg.vl_pis);
        //doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);
        //doc.complementar = reg.inf_comp.clone().unwrap_or_default();

        apply_common_rules(&mut doc, ctx, reg.aliq_pis, reg.aliq_cofins);
        doc
    }

    pub fn from_f500(reg: &RegistroF500, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = ctx.pa;
        doc.valor_item = dec_to_f64(reg.vl_rec_caixa);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_pis = dec_to_f64(reg.vl_pis);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);

        apply_common_rules(&mut doc, ctx, reg.aliq_pis, reg.aliq_cofins);
        doc
    }

    pub fn from_f510(reg: &RegistroF510, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = ctx.pa;
        doc.valor_item = dec_to_f64(reg.vl_rec_caixa);
        //doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        //doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        //doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_pis = dec_to_f64(reg.vl_pis);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);

        //apply_common_rules(&mut doc, ctx, reg.aliq_pis, reg.aliq_cofins);
        doc
    }

    pub fn from_f550(reg: &RegistroF550, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = ctx.pa;
        doc.valor_item = dec_to_f64(reg.vl_rec_comp);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_pis = dec_to_f64(reg.vl_pis);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);
        doc.complementar = reg.info_compl.clone().unwrap_or_default();

        apply_common_rules(&mut doc, ctx, reg.aliq_pis, reg.aliq_cofins);
        doc
    }

    pub fn from_f560(reg: &RegistroF560, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = ctx.pa;
        doc.valor_item = dec_to_f64(reg.vl_rec_comp);
        //doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        //doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        //doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_pis = dec_to_f64(reg.vl_pis);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);
        doc.complementar = reg.info_compl.clone().unwrap_or_default();

        //apply_common_rules(&mut doc, ctx, reg.aliq_pis, reg.aliq_cofins);
        doc
    }

    pub fn from_i100(reg: &RegistroI100, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.data_emissao = ctx.pa;
        doc.valor_item = dec_to_f64(reg.vl_rec);
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_pis = dec_to_f64(reg.vl_pis);
        doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        //doc.cst = parse_u16_opt(&reg.cst_cofins);
        doc.complementar = reg.info_compl.clone().unwrap_or_default();

        apply_common_rules(&mut doc, ctx, reg.aliq_pis, reg.aliq_cofins);
        doc
    }

    // --- BLOCO M Mappers (Apuração e Crédito) ---

    pub fn from_m100(reg: &RegistroM100, ctx: &SpedContext) -> Vec<DocsFiscais> {
        let mut docs = Vec::new();

        // Linha Principal (Crédito Apurado)
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.registro = "M100".to_string();
        doc.data_emissao = ctx.pa;
        doc.valor_bc = dec_to_f64(reg.vl_bc_pis); // Base PIS
        doc.aliq_pis = dec_to_f64(reg.aliq_pis);
        doc.valor_pis = dec_to_f64(reg.vl_cred); // Valor Crédito
        // Gambiarra de compatibilidade: coloca BC PIS na coluna COFINS para visualização única se desejado
        doc.valor_bc = dec_to_f64(reg.vl_bc_pis);

        doc.cod_credito = parse_u16_opt(&reg.cod_cred);

        apply_credit_rules(&mut doc, ctx, reg.cod_cred.as_deref());
        doc.format();
        docs.push(doc);

        // Linhas de Ajuste (Se houver valor)
        let acres = dec_to_f64(reg.vl_ajus_acres).unwrap_or(0.0);
        if acres.abs() > 0.0 {
            let mut d = create_adjustment_doc(ctx, reg.line_number, acres, 3); // 3: Acréscimo
            d.registro = "M100_AJ".to_string();
            d.format();
            docs.push(d);
        }
        let reduc = dec_to_f64(reg.vl_ajus_reduc).unwrap_or(0.0);
        if reduc.abs() > 0.0 {
            let mut d = create_adjustment_doc(ctx, reg.line_number, -reduc.abs(), 4); // 4: Redução
            d.registro = "M100_AJ".to_string();
            d.format();
            docs.push(d);
        }
        let desc = dec_to_f64(reg.vl_cred_desc).unwrap_or(0.0);
        if desc.abs() > 0.0 {
            let mut d = create_adjustment_doc(ctx, reg.line_number, -desc.abs(), 5); // 5: Desconto
            d.registro = "M100_AJ".to_string();
            d.format();
            docs.push(d);
        }

        docs
    }

    pub fn from_m500(reg: &RegistroM500, ctx: &SpedContext) -> Vec<DocsFiscais> {
        let mut docs = Vec::new();

        // Linha Principal
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.registro = "M500".to_string();
        doc.data_emissao = ctx.pa;
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        doc.aliq_cofins = dec_to_f64(reg.aliq_cofins);
        doc.valor_cofins = dec_to_f64(reg.vl_cred);

        doc.cod_credito = parse_u16_opt(&reg.cod_cred);

        apply_credit_rules(&mut doc, ctx, reg.cod_cred.as_deref());
        doc.format();
        docs.push(doc);

        // Ajustes
        let acres = dec_to_f64(reg.vl_ajus_acres).unwrap_or(0.0);
        if acres.abs() > 0.0 {
            docs.push(create_adjustment_doc(ctx, reg.line_number, acres, 3));
        }

        let reduc = dec_to_f64(reg.vl_ajus_reduc).unwrap_or(0.0);
        if reduc.abs() > 0.0 {
            docs.push(create_adjustment_doc(ctx, reg.line_number, -reduc.abs(), 4));
        }

        let desc = dec_to_f64(reg.vl_cred_desc).unwrap_or(0.0);
        if desc.abs() > 0.0 {
            docs.push(create_adjustment_doc(ctx, reg.line_number, -desc.abs(), 5));
        }

        docs
    }

    pub fn from_m505(reg: &RegistroM505, parent: &RegistroM500, ctx: &SpedContext) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, reg.line_number);
        doc.registro = "M505".to_string();
        doc.data_emissao = ctx.pa;

        // Herda dados do pai M500
        doc.aliq_cofins = dec_to_f64(parent.aliq_cofins);
        doc.cod_credito = parse_u16_opt(&parent.cod_cred);

        // Dados do M505
        doc.valor_bc = dec_to_f64(reg.vl_bc_cofins);
        //doc.valor_cofins = dec_to_f64(reg.vl_cofins);
        doc.cst = parse_u16_opt(&reg.cst_cofins);

        // Correlação: Tenta achar PIS baseado na aliq de COFINS (regra simples)
        correlate_pis_simple(&mut doc);

        doc.tipo_de_operacao = Some(7); // 7: Detalhamento
        apply_credit_rules(&mut doc, ctx, parent.cod_cred.as_deref());
        doc
    }

    // --- BLOCO 1 Mappers (Controle de Créditos) ---

    pub fn from_1100(reg: &Registro1100, ctx: &SpedContext) -> Option<DocsFiscais> {
        // Controle Crédito PIS
        let val_desc = dec_to_f64(reg.vl_cred_desc_efd).unwrap_or(0.0);
        if val_desc.abs() == 0.0 {
            return None;
        }

        let mut doc = create_adjustment_doc(ctx, reg.line_number, -val_desc.abs(), 6); // 6: Desconto Período Posterior
        doc.registro = "1100".to_string();
        doc.cod_credito = parse_u16_opt(&reg.cod_cred);

        // Check origem
        check_credit_origin_date(&mut doc, reg.per_apu_cred.as_deref(), ctx.pa);

        doc.format();
        Some(doc)
    }

    pub fn from_1500(reg: &Registro1500, ctx: &SpedContext) -> Option<DocsFiscais> {
        // Controle Crédito COFINS
        let val_desc = dec_to_f64(reg.vl_cred_desc_efd).unwrap_or(0.0);
        if val_desc.abs() == 0.0 {
            return None;
        }

        let mut doc = create_adjustment_doc(ctx, reg.line_number, -val_desc.abs(), 6);
        doc.registro = "1500".to_string();
        doc.cod_credito = parse_u16_opt(&reg.cod_cred);

        check_credit_origin_date(&mut doc, reg.per_apu_cred.as_deref(), ctx.pa);

        doc.format();
        Some(doc)
    }
}

// ============================================================================
// 5. Helpers (Lógica Auxiliar e de Negócio)
// ============================================================================

mod helpers {
    use crate::obter_cod_da_natureza_da_bc;

    use super::*;

    pub fn create_base_doc(ctx: &SpedContext, line_num: usize) -> DocsFiscais {
        let mut doc = DocsFiscais {
            arquivo_efd: ctx.arquivo_path.to_string_lossy().to_string(),
            num_linha_efd: Some(line_num),
            estabelecimento_cnpj: ctx.cnpj_base.clone(),
            periodo_de_apuracao: ctx.pa,
            linhas: 1,
            ..Default::default()
        };

        if let Some(date) = ctx.pa {
            doc.ano = Some(date.year());
            doc.mes = Some(date.month());
            doc.trimestre = Some((date.month() - 1) / 3 + 1);
        }

        doc
    }

    pub fn create_adjustment_doc(
        ctx: &SpedContext,
        line_num: usize,
        val: f64,
        tipo_op: u16,
    ) -> DocsFiscais {
        let mut doc = create_base_doc(ctx, line_num);
        doc.valor_item = Some(val);
        doc.tipo_de_operacao = Some(tipo_op);
        // Adiciona CNPJ da Matriz (geralmente o próprio do arquivo)
        doc.particante_cnpj = ctx.cnpj_base.clone();
        doc.particante_nome = "MATRIZ".to_string();
        doc
    }

    pub fn apply_common_rules(
        doc: &mut DocsFiscais,
        _ctx: &SpedContext,
        aliq_pis: Option<Decimal>,
        aliq_cofins: Option<Decimal>,
    ) {
        // Determina Tipo de Operação (Entrada/Saída) baseado no CST
        if doc.tipo_de_operacao.is_none() {
            doc.tipo_de_operacao = match doc.cst {
                Some(1..=49) => Some(2),  // Saída
                Some(50..=99) => Some(1), // Entrada
                _ => None,
            };
        }

        // Natureza BC
        if doc.natureza_bc.is_none() {
            doc.natureza_bc = obter_cod_da_natureza_da_bc(&doc.cfop, doc.cst);
        }

        // Indicador de Origem
        if doc.indicador_de_origem.is_none()
            && let Some(cfop) = doc.cfop {
                // 3000-3999 = Importação
                if (3000..=3999).contains(&cfop) {
                    doc.indicador_de_origem = Some(1);
                } else {
                    doc.indicador_de_origem = Some(0); // Mercado Interno default
                }
            }

        // Tipo de Crédito
        doc.tipo_de_credito = determinar_tipo_de_credito(
            doc.cst,
            dec_to_f64(aliq_pis),
            dec_to_f64(aliq_cofins),
            doc.cod_credito,
            doc.indicador_de_origem,
        );

        doc.format();
    }

    pub fn apply_credit_rules(
        doc: &mut DocsFiscais,
        _ctx: &SpedContext,
        cod_cred_str: Option<&str>,
    ) {
        if let Some(cod) = cod_cred_str
            && let Ok(c) = cod.parse::<u16>() {
                // Extrai o tipo pelo resto da divisão: 101 -> 01
                let tipo = c % 100;
                if tipo == 8 {
                    doc.indicador_de_origem = Some(1);
                } // Importação
                doc.tipo_de_credito = Some(tipo);
            }
    }

    pub fn enrich_participant(
        doc: &mut DocsFiscais,
        ctx: &SpedContext,
        cod_part_opt: &Option<String>,
    ) {
        if let Some(cod_part) = cod_part_opt
            && !cod_part.is_empty() {
                if let Some(p) = ctx.participantes.get(cod_part) {
                    doc.particante_cnpj = p.cnpj.clone().unwrap_or_default();
                    doc.particante_cpf = p.cpf.clone().unwrap_or_default();
                    doc.particante_nome = p.nome.clone().unwrap_or_default();
                    return;
                }
                // Se não achou no map, mas é um CPF/CNPJ direto (casos de C191/C195)
                if cod_part.len() == 14 {
                    doc.particante_cnpj = cod_part.clone();
                    doc.particante_nome = ctx.obter_nome_participante(Some(cod_part), None);
                } else if cod_part.len() == 11 {
                    doc.particante_cpf = cod_part.clone();
                    doc.particante_nome = ctx.obter_nome_participante(None, Some(cod_part));
                }
            }
    }

    // Utilitários de Parsing e Lookup

    pub fn dec_to_f64(d: Option<Decimal>) -> Option<f64> {
        d.and_then(|v| v.to_f64())
    }

    pub fn parse_u16_opt(s: &Option<String>) -> Option<u16> {
        s.as_ref().and_then(|v| v.parse().ok())
    }

    pub fn parse_u32_opt(s: &Option<String>) -> Option<u32> {
        s.as_ref().and_then(|v| v.parse().ok())
    }

    pub fn parse_usize_opt(s: &Option<String>) -> Option<usize> {
        s.as_ref().and_then(|v| v.parse().ok())
    }

    pub fn get_ncm(ctx: &SpedContext, cod_item: &Option<String>) -> String {
        cod_item
            .as_ref()
            .and_then(|c| ctx.produtos.get(c))
            .and_then(|p| p.cod_ncm.clone())
            .unwrap_or_default()
    }

    pub fn get_tipo_item(ctx: &SpedContext, cod_item: &Option<String>) -> String {
        cod_item
            .as_ref()
            .and_then(|c| ctx.produtos.get(c))
            .and_then(|p| p.tipo_item.clone()) // Assumindo campo tipo_item no Registro0200
            .unwrap_or_default()
    }

    pub fn get_descr_item(ctx: &SpedContext, cod_item: &Option<String>) -> String {
        cod_item
            .as_ref()
            .and_then(|c| ctx.produtos.get(c))
            .and_then(|p| p.descr_item.clone())
            .unwrap_or_default()
    }

    pub fn get_nat_operacao(ctx: &SpedContext, cod_nat: &Option<String>) -> String {
        cod_nat
            .as_ref()
            .and_then(|c| ctx.naturezas.get(c))
            .and_then(|n| n.descr_nat.clone())
            .unwrap_or_default()
    }

    pub fn get_conta_contabil(ctx: &SpedContext, cod_cta: &Option<String>) -> String {
        cod_cta
            .as_ref()
            .and_then(|c| ctx.contas.get(c))
            .and_then(|c| c.nome_cta.clone())
            .unwrap_or_default()
    }

    // --- Lógica de Correlação (PIS x COFINS) ---

    pub fn store_correlation_pis(
        cache: &mut HashMap<String, (f64, f64)>,
        cst: Option<String>,
        vl_item: Option<Decimal>,
        aliq: Option<Decimal>,
        vl_pis: Option<Decimal>,
        cfop: Option<&str>,
        part: Option<&str>,
    ) {
        if let (Some(cst_val), Some(vl_i), Some(a), Some(v)) = (cst, vl_item, aliq, vl_pis) {
            let aliq_f = a.to_f64().unwrap_or(0.0);
            let val_f = v.to_f64().unwrap_or(0.0);

            // Chave Fraca
            let weak_key = format!("{}_{}", cst_val, vl_i);
            cache.insert(weak_key.clone(), (aliq_f, val_f));

            // Chave Forte
            if let (Some(cf), Some(pt)) = (cfop, part) {
                let strong_key = format!("{}_{}_{}_{}", cst_val, vl_i, cf, pt);
                cache.insert(strong_key, (aliq_f, val_f));
            }
        }
    }

    pub fn correlate_pis(
        doc: &mut DocsFiscais,
        cache: &HashMap<String, (f64, f64)>,
        vl_item: Option<Decimal>,
        cst_cofins: Option<&str>,
        cfop: Option<&str>,
        part: Option<&str>,
    ) {
        if let (Some(vl), Some(cst)) = (vl_item, cst_cofins) {
            // Tenta chave forte
            let key = if let (Some(c), Some(p)) = (cfop, part) {
                format!("{}_{}_{}_{}", cst, vl, c, p)
            } else {
                format!("{}_{}", cst, vl)
            };

            if let Some((aliq, val)) = cache.get(&key) {
                doc.aliq_pis = Some(*aliq);
                doc.valor_pis = Some(*val);
                return;
            }

            // Fallback para chave fraca se forte falhou
            if cfop.is_some() {
                let weak = format!("{}_{}", cst, vl);
                if let Some((aliq, val)) = cache.get(&weak) {
                    doc.aliq_pis = Some(*aliq);
                    doc.valor_pis = Some(*val);
                }
            }
        }
    }

    pub fn correlate_pis_simple(doc: &mut DocsFiscais) {
        // Regra simples baseada na alíquota COFINS comum
        if let Some(ac) = doc.aliq_cofins {
            if (ac - 7.6).abs() < 0.001 {
                doc.aliq_pis = Some(1.65);
            } else if (ac - 3.0).abs() < 0.001 {
                doc.aliq_pis = Some(0.65);
            } // Financeira
        }
    }

    // --- Lógica de CST/Crédito ---

    fn determinar_tipo_de_credito(
        cst_cofins: Option<u16>,
        aliq_pis: Option<f64>,
        aliq_cofins: Option<f64>,
        cod_credito: Option<u16>,
        indicador_de_origem: Option<u16>,
    ) -> Option<u16> {
        // Lógica transcrita de determinar_tipo_de_credito do analyze_one.rs antigo
        if let Some(cod) = cod_credito {
            return Some(cod % 100);
        }

        match (aliq_pis, aliq_cofins) {
            (Some(ap), Some(ac)) if ap > 0.0 || ac > 0.0 => {
                match indicador_de_origem {
                    Some(0) => match cst_cofins {
                        Some(50..=56) => {
                            if (ap - ALIQ_BASICA_PIS).abs() < 0.001
                                && (ac - ALIQ_BASICA_COF).abs() < 0.001
                            {
                                Some(1) // Básica
                            } else {
                                Some(2) // Diferenciada
                            }
                        }
                        Some(60..=66) => Some(6), // Presumido (Simplificado)
                        _ => None,
                    },
                    Some(1) => Some(8), // Importação
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn check_credit_origin_date(
        doc: &mut DocsFiscais,
        per_apu: Option<&str>,
        pa_atual: Option<NaiveDate>,
    ) {
        // Verifica se período de origem é diferente do atual
        if let (Some(origem_str), Some(atual)) = (per_apu, pa_atual) {
            // Parse origem MMYYYY
            if origem_str.len() == 6
                && let (Ok(m), Ok(y)) = (
                    origem_str[0..2].parse::<u32>(),
                    origem_str[2..6].parse::<i32>(),
                )
                    && let Some(date_origem) = NaiveDate::from_ymd_opt(y, m, 1)
                        && date_origem != atual {
                            doc.complementar =
                                format!("{} [Origem Diferente: {}]", doc.complementar, origem_str);
                        }
        }
    }
}
