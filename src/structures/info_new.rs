use crate::{
    DocsFiscais,
    EFDResult,
    SpedFile,
    SpedRecord,
    blocos::*, // Importa todos os registros (Registro0000, C100, etc)
};
use chrono::{Datelike, NaiveDate};
use rust_decimal::prelude::ToPrimitive;
use std::{collections::HashMap, path::PathBuf};

// ============================================================================
// Contexto Imutável (Substitui parte do antigo Info mutável)
// ============================================================================

#[derive(Debug)]
pub struct SpedContext {
    pub arquivo_path: PathBuf,
    pub cnpj_base: String,     // Primeiros 8 dígitos
    pub pa: Option<NaiveDate>, // Período de Apuração
    pub messages: String,      // Acúmulo inicial de mensagens de erro do Bloco 0

    // Tabelas de consulta rápida (Read-Only durante processamento dos blocos)
    pub participantes: HashMap<String, Registro0150>, // Map<CodPart, Registro>
    pub produtos: HashMap<String, Registro0200>,      // Map<CodItem, Registro>
    pub contas: HashMap<String, Registro0500>,        // Map<CodCta, Registro>
    pub naturezas: HashMap<String, Registro0400>,     // Map<CodNat, Registro>
    pub estabelecimentos: HashMap<String, Registro0140>,
}

// ============================================================================
// Construção do Contexto (Bloco 0)
// ============================================================================

pub fn build_sped_context(file: &SpedFile, path: &std::path::Path) -> EFDResult<SpedContext> {
    let mut ctx = SpedContext {
        arquivo_path: path.to_path_buf(),
        cnpj_base: String::new(),
        pa: None,
        messages: String::new(),
        participantes: HashMap::new(),
        produtos: HashMap::new(),
        contas: HashMap::new(),
        naturezas: HashMap::new(),
        estabelecimentos: HashMap::new(),
    };

    // Extrai registros do Bloco 0
    let records_0 = match file.obter_bloco_option('0') {
        Some(recs) => recs,
        None => return Ok(ctx), // Arquivo vazio ou sem bloco 0?
    };

    // Itera sequencialmente para popular tabelas
    for rec in records_0 {
        if let SpedRecord::Generic(inner) = rec {
            // Pattern Matching pelo nome do registro para downcast seguro
            match inner.registro_name() {
                "0000" => {
                    if let Some(r) = inner.as_any().downcast_ref::<Registro0000>() {
                        ctx.pa = Some(r.dt_ini); // Simplificação: usa dt_ini como PA base
                        if r.cnpj.len() >= 8 {
                            ctx.cnpj_base = r.cnpj[0..8].to_string();
                        }
                    }
                }
                "0140" => {
                    if let Some(r) = inner.as_any().downcast_ref::<Registro0140>()
                        && let Some(cnpj) = &r.cnpj
                    {
                        ctx.estabelecimentos.insert(cnpj.clone(), r.clone());
                    }
                }
                "0150" => {
                    if let Some(r) = inner.as_any().downcast_ref::<Registro0150>() {
                        ctx.participantes
                            .insert(r.cod_part.clone().unwrap_or_default(), r.clone());
                    }
                }
                "0200" => {
                    if let Some(r) = inner.as_any().downcast_ref::<Registro0200>() {
                        ctx.produtos
                            .insert(r.cod_item.clone().unwrap_or_default(), r.clone());
                    }
                }
                "0400" => {
                    if let Some(r) = inner.as_any().downcast_ref::<Registro0400>() {
                        ctx.naturezas
                            .insert(r.cod_nat.clone().unwrap_or_default(), r.clone());
                    }
                }
                "0500" => {
                    if let Some(r) = inner.as_any().downcast_ref::<Registro0500>() {
                        ctx.contas
                            .insert(r.cod_cta.clone().unwrap_or_default(), r.clone());
                    }
                }
                _ => {}
            }
        }
    }

    if ctx.pa.is_none() {
        // Erro crítico ou aviso
        eprintln!("Aviso: Registro 0000 não encontrado ou Data Inicial inválida.");
    }

    Ok(ctx)
}

// ============================================================================
// Processamento de Blocos (Substitui Dispatch Table)
// ============================================================================

/// Estrutura para manter o estado hierárquico durante a iteração linear de um bloco
#[derive(Default)]
struct BlockProcessorState<'a> {
    // Bloco C/D
    c100: Option<&'a RegistroC100>,
    _c170: Option<&'a RegistroC170>, // Pode ser usado para sub-registros se houver
    _d100: Option<&'a RegistroD100>,
    // Bloco M
    m100: Option<&'a RegistroM100>,
    _m500: Option<&'a RegistroM500>,
    // Adicionar outros pais conforme necessidade
}

/// Processa todas as linhas de um bloco específico, retornando vetor de DocsFiscais
pub fn process_block_lines(bloco: char, file: &SpedFile, ctx: &SpedContext) -> Vec<DocsFiscais> {
    let records = match file.obter_bloco_option(bloco) {
        Some(list) => list,
        None => return Vec::new(),
    };

    let mut docs = Vec::with_capacity(records.len());
    let mut state = BlockProcessorState::default();

    // Itera sequencialmente sobre os registros do bloco (já ordenados)
    for record in records {
        // Dispatch baseado no tipo concreto do registro
        if let SpedRecord::Generic(inner) = record {
            match inner.registro_name() {
                // --- BLOCO C ---
                "C100" => {
                    if let Some(r) = inner.as_any().downcast_ref::<RegistroC100>() {
                        state.c100 = Some(r);
                        // C100 geralmente é cabeçalho, mas se quiser gerar doc dele:
                        // docs.push(from_c100(r, ctx));
                    }
                }
                "C170" => {
                    if let Some(r) = inner.as_any().downcast_ref::<RegistroC170>()
                        && let Some(parent) = state.c100
                            && let Some(doc) = from_c170(r, parent, ctx) {
                                docs.push(doc);
                            }
                }
                "C175" => {
                    if let Some(r) = inner.as_any().downcast_ref::<RegistroC175>()
                        && let Some(parent) = state.c100 {
                            // Lógica similar ao C170
                            if let Some(doc) = from_c175(r, parent, ctx) {
                                docs.push(doc);
                            }
                        }
                }

                // --- BLOCO M (Exemplo de Apuração) ---
                "M100" => {
                    if let Some(r) = inner.as_any().downcast_ref::<RegistroM100>() {
                        state.m100 = Some(r);
                        // Processar M100 (Crédito PIS) -> Gerar Docs de Ajuste?
                        docs.extend(from_m100(r, ctx));
                    }
                }
                // Adicionar lógica para M500, D100, etc. aqui
                _ => {}
            }
        }
    }

    docs
}

// ============================================================================
// Funções de Transformação (Mappers)
// ============================================================================

// Helper para converter Option<Decimal> para Option<f64>
fn dec_to_f64(d: Option<rust_decimal::Decimal>) -> Option<f64> {
    d.and_then(|v| v.to_f64())
}

fn from_c170(reg: &RegistroC170, parent: &RegistroC100, ctx: &SpedContext) -> Option<DocsFiscais> {
    // Lógica de extração de dados do contexto
    let prod_info = reg.cod_item.as_ref().and_then(|c| ctx.produtos.get(c));
    let part_info = parent
        .cod_part
        .as_ref()
        .and_then(|c| ctx.participantes.get(c));

    let (_part_cnpj, _part_cpf, _part_nome) = match part_info {
        Some(p) => (
            p.cnpj.as_deref().unwrap_or_default(),
            p.cpf.as_deref().unwrap_or_default(),
            p.nome.as_deref().unwrap_or_default(),
        ),
        None => ("", "", ""),
    };

    let cst_cofins_u16 = reg.cst_cofins.as_ref().and_then(|s| s.parse::<u16>().ok());

    // Construção do DocsFiscais
    let mut doc = DocsFiscais {
        arquivo_efd: ctx.arquivo_path.to_string_lossy().to_string(),
        num_linha_efd: Some(reg.line_number),

        // Dados do Contexto
        estabelecimento_cnpj: ctx.cnpj_base.clone(), // Simplificação, idealmente pegar do 0140/Filial
        estabelecimento_nome: "TODO: Obter do 0000 ou 0140".to_string(),
        periodo_de_apuracao: ctx.pa,

        // Dados Hierarquia (Pai C100)
        data_emissao: parent.dt_doc,
        data_lancamento: parent.dt_e_s,
        chave_doc: parent.chv_nfe.clone().unwrap_or_default(),
        num_doc: parent.num_doc.as_ref().and_then(|s| s.parse().ok()),

        // Dados do Item (C170)
        num_item: reg.num_item.as_ref().and_then(|s| s.parse().ok()),
        tipo_item: prod_info
            .map(|p| p.tipo_item.clone().unwrap_or_default())
            .unwrap_or_default(),
        descr_item: prod_info
            .map(|p| p.descr_item.clone().unwrap_or_default())
            .unwrap_or_default(),
        cod_ncm: prod_info
            .map(|p| p.cod_ncm.clone().unwrap_or_default())
            .unwrap_or_default(),

        cst: cst_cofins_u16,
        cfop: reg.cfop.as_ref().and_then(|s| s.parse().ok()),

        // Valores (Decimal -> f64)
        valor_item: dec_to_f64(reg.vl_item),
        valor_bc: dec_to_f64(reg.vl_bc_cofins),
        aliq_pis: dec_to_f64(reg.aliq_pis),
        aliq_cofins: dec_to_f64(reg.aliq_cofins),
        valor_pis: dec_to_f64(reg.vl_pis),
        valor_cofins: dec_to_f64(reg.vl_cofins),

        ..Default::default()
    };

    // Preencher campos derivados (Ano, Mês, Trimestre)
    if let Some(date) = doc.periodo_de_apuracao {
        doc.ano = Some(date.year());
        doc.mes = Some(date.month());
        doc.trimestre = Some((date.month() - 1) / 3 + 1);
    }

    // Aplicar formatações finais (CNPJ masks, etc)
    doc.format();

    Some(doc)
}

fn from_c175(reg: &RegistroC175, parent: &RegistroC100, ctx: &SpedContext) -> Option<DocsFiscais> {
    // Implementação similar ao C170, mas mapeando campos do C175 (Agregado)
    // O C175 não tem código de item individual no mesmo nível de detalhe às vezes,
    // depende da estrutura do registro.

    let mut doc = DocsFiscais {
        arquivo_efd: ctx.arquivo_path.to_string_lossy().to_string(),
        num_linha_efd: Some(reg.line_number),
        periodo_de_apuracao: ctx.pa,
        // ... Preencher dados do pai
        chave_doc: parent.chv_nfe.clone().unwrap_or_default(),

        // Valores C175
        valor_item: dec_to_f64(reg.vl_opr), // Campo VL_OPR no C175
        valor_bc: dec_to_f64(reg.vl_bc_cofins),
        aliq_pis: dec_to_f64(reg.aliq_pis),
        aliq_cofins: dec_to_f64(reg.aliq_cofins),
        valor_pis: dec_to_f64(reg.vl_pis),
        valor_cofins: dec_to_f64(reg.vl_cofins),

        cst: reg.cst_cofins.as_ref().and_then(|s| s.parse().ok()),
        cfop: reg.cfop.as_ref().and_then(|s| s.parse().ok()),

        ..Default::default()
    };

    if let Some(date) = doc.periodo_de_apuracao {
        doc.ano = Some(date.year());
        doc.mes = Some(date.month());
    }
    doc.format();

    Some(doc)
}

fn from_m100(reg: &RegistroM100, ctx: &SpedContext) -> Vec<DocsFiscais> {
    // O M100 pode gerar múltiplas linhas de DocsFiscais se houver ajustes
    // Exemplo: 1 linha para o crédito, outras para ajustes se necessário.
    // Aqui segue a lógica simplificada de mapear o crédito apurado.

    let mut doc = DocsFiscais {
        arquivo_efd: ctx.arquivo_path.to_string_lossy().to_string(),
        num_linha_efd: Some(reg.line_number),
        periodo_de_apuracao: ctx.pa,
        registro: "M100".to_string(),

        valor_bc: dec_to_f64(reg.vl_bc_pis), // Base de cálculo PIS
        aliq_pis: dec_to_f64(reg.aliq_pis),
        valor_pis: dec_to_f64(reg.vl_cred), // Valor do crédito

        cod_credito: reg.cod_cred.as_ref().and_then(|s| s.parse().ok()),

        ..Default::default()
    };

    // Preenchimento padrão de datas
    if let Some(date) = doc.periodo_de_apuracao {
        doc.ano = Some(date.year());
        doc.mes = Some(date.month());
    }
    doc.format();

    vec![doc]
}

// Adicione as demais implementações (from_d100, from_m500, etc.) seguindo o padrão.
// O objetivo é substituir cada função 'ler_registro_X' do dispatch_table
// por uma função 'from_X' que retorna Option<DocsFiscais> ou Vec<DocsFiscais>.
