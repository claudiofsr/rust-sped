use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{EFDResult, SpedFile, SpedRecord, StringParser, blocos::*, obter_grupo_de_contas};

// ============================================================================
// 1. Contexto Imutável (Dados Globais e Tabelas)
// ============================================================================

/// Mantém as tabelas de referência carregadas do Bloco 0.
///
/// O uso de `Arc<str>` (Atomic Reference Counting) substitui `String` para garantir
/// que strings repetitivas (como nomes, CNPJs e descrições) sejam alocadas
/// apenas uma vez na Heap e compartilhadas entre milhares de registros `DocsFiscais`
/// através de ponteiros baratos, economizando significativamente a memória RAM.
#[derive(Debug, Default)]
pub struct SpedContext {
    pub path: PathBuf,
    pub messages: Vec<String>, // Mensagens de erro/aviso

    // Cache do nome do arquivo para uso repetitivo
    pub arquivo_efd: Arc<str>,

    // --- Metadados Globais (Registro 0000 e afins) ---
    // Registro0000
    pub estabelecimento_cnpj: Arc<str>,
    pub estabelecimento_cnpj_base: Arc<str>,
    pub estabelecimento_nome: Arc<str>,

    pub periodo_de_apuracao: Option<NaiveDate>, // Período de Apuração
    pub dt_ini: NaiveDate,
    pub dt_fin: NaiveDate,

    // Registro0110
    pub ind_apro_cred: Option<Arc<str>>,

    // Registro0111
    pub rec_bru_ncum_trib_mi: Option<Decimal>,
    pub rec_bru_ncum_nt_mi: Option<Decimal>,
    pub rec_bru_ncum_exp: Option<Decimal>,
    pub rec_bru_cum: Option<Decimal>,
    pub rec_bru_total: Option<Decimal>,

    // --- Tabelas de consulta (Lookups com Arc<str>) ---
    // (Chaves Arc<str> economizam RAM se repetidas, Valores Arc<str> evitam cópia)
    pub complementar: HashMap<Arc<str>, Arc<str>>,
    pub contabil: HashMap<Arc<str>, HashMap<Arc<str>, Arc<str>>>,
    pub estabelecimentos: HashMap<Arc<str>, Arc<str>>,
    pub nat_operacao: HashMap<Arc<str>, Arc<str>>,
    pub participantes: BTreeMap<Arc<str>, HashMap<Arc<str>, Arc<str>>>,
    pub produtos: BTreeMap<Arc<str>, HashMap<Arc<str>, Arc<str>>>,
    pub unidade_de_medida: HashMap<Arc<str>, Arc<str>>,

    // Cache de Nomes para Lookup Rápido
    pub nome_do_cnpj: BTreeMap<Arc<str>, Arc<str>>,
    pub nome_do_cpf: BTreeMap<Arc<str>, Arc<str>>,
}

impl SpedContext {
    /// Constrói o contexto lendo apenas o Bloco 0 do arquivo SPED.
    pub fn new(file: &SpedFile, path: &Path) -> EFDResult<Self> {
        // Inicialização "Lazy" ou default
        let mut ctx = Self {
            path: path.to_path_buf(),
            arquivo_efd: path.to_string_lossy().into(), // Cow -> Arc<str>

            // Inicializa vazios (serão sobrescritos pelo Reg 0000)
            estabelecimento_cnpj: Arc::from(""),
            estabelecimento_cnpj_base: Arc::from(""),
            estabelecimento_nome: Arc::from(""),
            ..Self::default()
        };

        // Fail fast se não houver bloco 0
        let bloco_0 = match file.obter_bloco_option('0') {
            Some(recs) => recs,
            None => return Ok(ctx),
        };

        // Itera sobre os registros do Bloco 0 para popular as tabelas
        for sped_record in bloco_0 {
            // Guard clause para filtrar Generic records
            let SpedRecord::Generic(inner) = sped_record else {
                continue;
            };

            match inner.registro_name() {
                "0000" => {
                    // Registro 0000: Abertura do Arquivo Digital e Identificação da Pessoa Jurídica
                    if let Ok(r) = sped_record.downcast_ref::<Registro0000>() {
                        // Conversão direta de String -> &str -> Arc<str>
                        ctx.estabelecimento_cnpj = Arc::from(r.cnpj.as_str());
                        ctx.estabelecimento_cnpj_base = Arc::from(r.get_cnpj_base().as_str());

                        // to_upper_arc garante normalização e retorna Option<Arc>.
                        // Unwrap_or cria um Arc vazio se None.
                        ctx.estabelecimento_nome =
                            r.nome.to_upper_arc().unwrap_or_else(|| Arc::from(""));

                        ctx.periodo_de_apuracao = Some(r.dt_ini);
                        ctx.dt_ini = r.dt_ini;
                        ctx.dt_fin = r.dt_fin;
                    }
                }
                "0110" => {
                    // Registro 0110: Regimes de Apuração da Contribuição Social e de Apropriação de Crédito
                    if let Ok(r) = sped_record.downcast_ref::<Registro0110>() {
                        // Clone de Arc é barato (apenas incrementa contador)
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
                    if let Ok(r) = sped_record.downcast_ref::<Registro0150>()
                        && let Some(cod_part) = &r.cod_part
                        && !cod_part.is_empty()
                    {
                        let mut hash: HashMap<Arc<str>, Arc<str>> = HashMap::new();

                        if let Some(nome) = r.nome.as_ref().filter(|s| !s.is_empty()) {
                            hash.insert("NOME".into(), nome.clone());

                            if let Some(cnpj) = r.cnpj.as_ref().filter(|s| !s.is_empty()) {
                                hash.insert("CNPJ".into(), cnpj.clone());
                                ctx.nome_do_cnpj.insert(cnpj.clone(), nome.clone());
                            }
                            if let Some(cpf) = r.cpf.as_ref().filter(|s| !s.is_empty()) {
                                hash.insert("CPF".into(), cpf.clone());
                                ctx.nome_do_cpf.insert(cpf.clone(), nome.clone());
                            }
                        }

                        ctx.participantes.insert(cod_part.clone(), hash);
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
                        let mut item_data: HashMap<Arc<str>, Arc<str>> = HashMap::with_capacity(5);

                        // Helper closure para inserir Arc se não for vazio
                        let mut insert_arc = |key: &str, val: &Option<Arc<str>>| {
                            if let Some(v) = val {
                                item_data.insert(key.into(), v.clone());
                            }
                        };

                        insert_arc("DESCR_ITEM", &r.descr_item);
                        insert_arc("TIPO_ITEM", &r.tipo_item);
                        insert_arc("COD_NCM", &r.cod_ncm);
                        insert_arc("COD_GEN", &r.cod_gen);
                        insert_arc("COD_LST", &r.cod_lst);

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
                    if let Ok(r) = sped_record.downcast_ref::<Registro0500>()
                        && let Some(cod_conta) = &r.cod_cta
                        && !cod_conta.is_empty()
                    {
                        let nome_da_conta = r.nome_cta.to_upper_arc();

                        // Resolve o grupo e o código
                        let (cod_nat_cc, grupo_de_contas) = r
                            .cod_nat_cc
                            .as_deref()
                            .map(|cod| (Some(Arc::from(cod)), obter_grupo_de_contas(cod)))
                            .unwrap_or((None, String::new()));

                        // Formatação do nome da conta
                        // Decisão sobre o nome final
                        let conta_contabil: Arc<str> =
                            match (grupo_de_contas.is_empty(), nome_da_conta) {
                                (true, None) => Arc::from(""),
                                (true, Some(nome)) => nome,
                                (false, None) => grupo_de_contas.into(),
                                (false, Some(nome)) => {
                                    // Aqui infelizmente precisamos alocar uma nova String composta
                                    Arc::from(format!("{}: {}", grupo_de_contas, nome))
                                }
                            };

                        let mut dados: HashMap<Arc<str>, Arc<str>> = HashMap::with_capacity(2);

                        if !conta_contabil.is_empty() {
                            dados.insert("NOME_CTA".into(), conta_contabil);
                        }

                        if let Some(cod) = cod_nat_cc {
                            dados.insert("COD_NAT_CC".into(), cod);
                        }

                        ctx.contabil.insert(cod_conta.clone(), dados);
                    }
                }
                _ => {}
            }
        }

        Ok(ctx)
    }

    /// Obtém o nome do participante baseado no CNPJ/CPF se não encontrado pelo código.
    /// Retorna um `Arc<str>` (ponteiro barato) para evitar alocações.
    pub fn obter_nome_participante(&self, cnpj: Option<&str>, cpf: Option<&str>) -> Arc<str> {
        // A lógica complexa de "most frequent value" pode ser aplicada aqui se desejar,
        // mas por simplicidade retornamos vazio ou busca exata.
        cnpj.and_then(|c| self.nome_do_cnpj.get(c))
            .or_else(|| cpf.and_then(|c| self.nome_do_cpf.get(c)))
            .cloned()
            .unwrap_or_default()
    }
}
