use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};

use crate::{
    EFDResult,
    SpedFile,
    SpedRecord,
    blocos::*,
    obter_grupo_de_contas, // Importa todos os structs de registros (Registro0000, C100, etc.)
};

// ============================================================================
// 1. Contexto Imutável (Dados Globais e Tabelas)
// ============================================================================

/// Mantém as tabelas de referência carregadas do Bloco 0.
/// É construído uma vez e passado como referência imutável para os processadores de blocos.
#[derive(Debug, Default)]
pub struct SpedContext {
    pub path: PathBuf,
    pub messages: Vec<String>, // Mensagens de erro/aviso coletadas na construção

    // --- Metadados Globais (Registro 0000 e afins) ---
    // Registro0000
    pub estabelecimento_cnpj: String,
    pub estabelecimento_cnpj_base: String,
    pub estabelecimento_nome: String,
    pub periodo_de_apuracao: Option<NaiveDate>, // Período de Apuração
    pub dt_ini: NaiveDate,
    pub dt_fin: NaiveDate,

    // Registro0110
    pub ind_apro_cred: Option<String>,

    // Registro0111
    pub rec_bru_ncum_trib_mi: Option<Decimal>,
    pub rec_bru_ncum_nt_mi: Option<Decimal>,
    pub rec_bru_ncum_exp: Option<Decimal>,
    pub rec_bru_cum: Option<Decimal>,
    pub rec_bru_total: Option<Decimal>,

    // --- Tabelas de consulta (Lookups) ---
    pub complementar: HashMap<String, String>,
    pub contabil: HashMap<String, HashMap<String, String>>,
    pub estabelecimentos: HashMap<String, String>,
    pub nat_operacao: HashMap<String, String>,
    pub participantes: BTreeMap<String, HashMap<String, String>>,
    pub produtos: BTreeMap<String, HashMap<String, String>>,
    pub unidade_de_medida: HashMap<String, String>,

    // Cache de Nomes
    pub nome_do_cnpj: BTreeMap<String, String>,
    pub nome_do_cpf: BTreeMap<String, String>,
}

impl SpedContext {
    /// Constrói o contexto lendo apenas o Bloco 0 do arquivo SPED.
    pub fn new(file: &SpedFile, path: &Path) -> EFDResult<Self> {
        let mut ctx = Self {
            path: path.to_path_buf(),
            ..Self::default()
        };

        let bloco_0 = match file.obter_bloco_option('0') {
            Some(recs) => recs,
            None => return Ok(ctx),
        };

        // Itera sobre os registros do Bloco 0 para popular as tabelas
        for sped_record in bloco_0 {
            // Usa o SpedRecord::downcast_ref via helper interno ou match direto se exposto
            // Assumindo a estrutura do model.rs fornecido:
            if let SpedRecord::Generic(inner) = sped_record {
                // Pattern Matching no nome do registro para performance e segurança
                match inner.registro_name() {
                    "0000" => {
                        // Registro 0000: Abertura do Arquivo Digital e Identificação da Pessoa Jurídica
                        if let Ok(r) = sped_record.downcast_ref::<Registro0000>() {
                            ctx.estabelecimento_cnpj = r.cnpj.clone();
                            ctx.estabelecimento_cnpj_base = r.get_cnpj_base();
                            ctx.estabelecimento_nome = r.get_nome().to_string();
                            ctx.periodo_de_apuracao = Some(r.dt_ini);
                            ctx.dt_ini = r.dt_ini;
                            ctx.dt_fin = r.dt_fin;
                        }
                    }
                    "0110" => {
                        // Registro 0110: Regimes de Apuração da Contribuição Social e de Apropriação de Crédito
                        if let Ok(r) = sped_record.downcast_ref::<Registro0110>() {
                            ctx.ind_apro_cred = r.ind_apro_cred.clone();
                        }
                    }
                    "0111" => {
                        // Registro 0111: Tabela de Receita Bruta Mensal Para Fins de Rateio de Créditos Comuns
                        if let Ok(r) = sped_record.downcast_ref::<Registro0111>() {
                            ctx.rec_bru_ncum_trib_mi = r.rec_bru_ncum_trib_mi;
                            ctx.rec_bru_ncum_nt_mi = r.rec_bru_ncum_nt_mi;
                            ctx.rec_bru_ncum_exp = r.rec_bru_ncum_exp;
                            ctx.rec_bru_cum = r.rec_bru_cum;
                            ctx.rec_bru_total = r.rec_bru_total;
                        }
                    }
                    "0140" => {
                        // Registro 0140: Tabela de Cadastro de Estabelecimentos
                        // O Registro 0140 tem por objetivo relacionar e informar os estabelecimentos da pessoa jurídica.
                        if let Ok(r) = sped_record.downcast_ref::<Registro0140>()
                            && let Some(cnpj) = &r.cnpj
                            && let Some(nome) = &r.nome
                            && !cnpj.is_empty()
                            && !nome.is_empty()
                        {
                            ctx.estabelecimentos.insert(cnpj.clone(), nome.clone());
                        }
                    }
                    "0150" => {
                        // Registro 0150: Tabela de Cadastro do Participante
                        // Atribuir NOME ao CNPJ ou ao CPF
                        if let Ok(r) = sped_record.downcast_ref::<Registro0150>() {
                            let mut hash = HashMap::new();

                            if let Some(nome) = &r.nome
                                && !nome.is_empty()
                            {
                                hash.insert("NOME".to_string(), nome.to_string());
                                if let Some(cnpj) = &r.cnpj
                                    && !cnpj.is_empty()
                                {
                                    hash.insert("CNPJ".to_string(), cnpj.to_string());
                                    ctx.nome_do_cnpj.insert(cnpj.clone(), nome.clone());
                                }
                                if let Some(cpf) = &r.cpf
                                    && !cpf.is_empty()
                                {
                                    hash.insert("CPF".to_string(), cpf.to_string());
                                    ctx.nome_do_cpf.insert(cpf.clone(), nome.clone());
                                }
                            }

                            if let Some(cod_part) = &r.cod_part
                                && !cod_part.is_empty()
                            {
                                ctx.participantes.insert(cod_part.clone(), hash);
                            }
                        }
                    }
                    "0190" => {
                        // Registro 0190: Identificação das Unidades de Medida
                        if let Ok(r) = sped_record.downcast_ref::<Registro0190>()
                            && let Some(cod_unidade) = &r.unid
                            && let Some(descricao) = &r.descr
                            && !cod_unidade.is_empty()
                            && !descricao.is_empty()
                        {
                            ctx.unidade_de_medida
                                .insert(cod_unidade.clone(), descricao.clone());
                        }
                    }
                    "0200" => {
                        // Registro 0200: Tabela de Identificação do Item (Produtos e Serviços)
                        if let Ok(r) = sped_record.downcast_ref::<Registro0200>()
                            && let Some(cod_item) = &r.cod_item
                            && !cod_item.is_empty()
                        {
                            let mut item_data = HashMap::with_capacity(5);

                            // Helper closure para inserir se não for vazio
                            let mut insert_if_present = |key: &str, val: &Option<String>| {
                                if let Some(v) = val.as_ref().filter(|s| !s.is_empty()) {
                                    item_data.insert(key.to_string(), v.to_string());
                                }
                            };

                            insert_if_present("DESCR_ITEM", &r.descr_item);
                            insert_if_present("TIPO_ITEM", &r.tipo_item);
                            insert_if_present("COD_NCM", &r.cod_ncm);
                            insert_if_present("COD_GEN", &r.cod_gen);
                            insert_if_present("COD_LST", &r.cod_lst);

                            ctx.produtos.insert(cod_item.clone(), item_data);
                        }
                    }
                    "0400" => {
                        // Registro 0400: Tabela de Natureza da Operação/Prestação
                        if let Ok(r) = sped_record.downcast_ref::<Registro0400>()
                            && let Some(cod_nat) = &r.cod_nat
                            && let Some(descr_nat) = &r.descr_nat
                            && !cod_nat.is_empty()
                            && !descr_nat.is_empty()
                        {
                            ctx.nat_operacao.insert(cod_nat.clone(), descr_nat.clone());
                        }
                    }
                    "0450" => {
                        // Registro 0450: Tabela de Informação Complementar do Documento Fiscal
                        if let Ok(r) = sped_record.downcast_ref::<Registro0450>()
                            && let Some(cod_inf) = &r.cod_inf
                            && let Some(txt) = &r.txt
                            && !cod_inf.is_empty()
                            && !txt.is_empty()
                        {
                            ctx.complementar.insert(cod_inf.clone(), txt.clone());
                        }
                    }
                    "0500" => {
                        // Registro 0500: Plano de Contas Contábeis
                        // Este registro tem o objetivo de identificar as contas contábeis utilizadas pelo contribuinte em sua Escrituração
                        // Contábil, relacionadas às operações representativas de receitas, tributadas ou não, e dos créditos apurados.
                        if let Ok(r) = sped_record.downcast_ref::<Registro0500>()
                            && let Some(cod_conta) = &r.cod_cta
                            && !cod_conta.is_empty()
                        {
                            // 1. Preparação dos dados (Imutável)
                            let nome_da_conta: String =
                                r.nome_cta.as_deref().unwrap_or_default().to_uppercase();

                            // Resolve o grupo e o código num único passo funcional
                            let (cod_nat_cc, grupo_de_contas) = r
                                .cod_nat_cc
                                .as_deref()
                                .map(|cod| (Some(cod.to_string()), obter_grupo_de_contas(cod)))
                                .unwrap_or((None, String::new()));

                            // 2. Lógica de Formatação (Pattern Matching)
                            // Define o nome composto baseando-se se as partes estão vazias ou não
                            let conta_contabil =
                                match (grupo_de_contas.is_empty(), nome_da_conta.is_empty()) {
                                    (true, true) => String::new(),
                                    (true, false) => nome_da_conta,
                                    (false, true) => grupo_de_contas,
                                    (false, false) => {
                                        format!("{}: {}", grupo_de_contas, nome_da_conta)
                                    }
                                };

                            // 3. Construção do Mapa
                            let mut dados = HashMap::with_capacity(2);

                            if !conta_contabil.is_empty() {
                                dados.insert("NOME_CTA".to_string(), conta_contabil);
                            }

                            if let Some(cod) = cod_nat_cc {
                                dados.insert("COD_NAT_CC".to_string(), cod);
                            }

                            ctx.contabil.insert(cod_conta.clone(), dados);
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
        if let Some(c) = cnpj
            && let Some(nome) = self.nome_do_cnpj.get(c)
        {
            return nome.clone();
        }
        // A lógica complexa de "most frequent value" pode ser aplicada aqui se desejar,
        // mas por simplicidade retornamos vazio ou busca exata.
        if let Some(c) = cpf
            && let Some(nome) = self.nome_do_cpf.get(c)
        {
            return nome.clone();
        }
        String::new()
    }
}
