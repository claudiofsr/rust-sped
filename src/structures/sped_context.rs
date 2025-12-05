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
    pub contabil: HashMap<Arc<str>, Arc<str>>,
    pub estabelecimentos: HashMap<Arc<str>, Arc<str>>,
    pub nat_operacao: HashMap<Arc<str>, Arc<str>>,
    pub participantes: BTreeMap<Arc<str>, HashMap<Arc<str>, Arc<str>>>,
    pub produtos: BTreeMap<Arc<str>, HashMap<Arc<str>, Arc<str>>>,
    pub unidade_de_medida: HashMap<Arc<str>, Arc<str>>,

    // Cache de Nomes para Lookup Rápido
    // Usamos BTreeMap para manter ordem e facilitar buscas determinísticas se necessário
    pub nome_do_cnpj: BTreeMap<Arc<str>, Arc<str>>,
    pub nome_do_cpf: BTreeMap<Arc<str>, Arc<str>>,
}

impl SpedContext {
    /// Constrói o contexto lendo apenas o Bloco 0.
    pub fn new(file: &SpedFile, path: &Path) -> EFDResult<Self> {
        let mut ctx = Self {
            path: path.to_path_buf(),
            arquivo_efd: path.to_string_lossy().into(),
            ..Self::default()
        };

        // Fail fast usando let-else
        let Some(bloco_0) = file.obter_bloco_option('0') else {
            return Ok(ctx);
        };

        for sped_record in bloco_0 {
            // Garante que é um registro genérico antes de tentar downcast
            let SpedRecord::Generic(inner) = sped_record else {
                continue;
            };

            match inner.registro_name() {
                "0000" => ctx.handle_0000(sped_record),
                "0110" => ctx.handle_0110(sped_record),
                "0111" => ctx.handle_0111(sped_record),
                "0140" => ctx.handle_0140(sped_record),
                "0150" => ctx.handle_0150(sped_record),
                "0190" => ctx.handle_0190(sped_record),
                "0200" => ctx.handle_0200(sped_record),
                "0400" => ctx.handle_0400(sped_record),
                "0450" => ctx.handle_0450(sped_record),
                "0500" => ctx.handle_0500(sped_record),
                _ => {}
            }
        }

        Ok(ctx)
    }

    // --- Handlers Específicos (Inline logic separation) ---

    /// Registro 0000: Abertura do Arquivo Digital e Identificação da Pessoa Jurídica
    fn handle_0000(&mut self, sped_record: &SpedRecord) {
        if let Ok(r) = sped_record.downcast_ref::<Registro0000>() {
            self.estabelecimento_cnpj = r.cnpj.clone();
            self.estabelecimento_cnpj_base = r.get_cnpj_base();
            self.estabelecimento_nome = r.get_nome();
            self.periodo_de_apuracao = Some(r.dt_ini);
            self.dt_ini = r.dt_ini;
            self.dt_fin = r.dt_fin;
        }
    }

    /// Registro 0110: Regimes de Apuração da Contribuição Social e de Apropriação de Crédito
    fn handle_0110(&mut self, sped_record: &SpedRecord) {
        if let Ok(r) = sped_record.downcast_ref::<Registro0110>() {
            // Clone de Arc é barato (apenas incrementa contador)
            self.ind_apro_cred = r.ind_apro_cred.clone();
        }
    }

    /// Registro 0111: Tabela de Receita Bruta Mensal Para Fins de Rateio de Créditos Comuns
    fn handle_0111(&mut self, sped_record: &SpedRecord) {
        if let Ok(r) = sped_record.downcast_ref::<Registro0111>() {
            self.rec_bru_ncum_trib_mi = r.rec_bru_ncum_trib_mi;
            self.rec_bru_ncum_nt_mi = r.rec_bru_ncum_nt_mi;
            self.rec_bru_ncum_exp = r.rec_bru_ncum_exp;
            self.rec_bru_cum = r.rec_bru_cum;
            self.rec_bru_total = r.rec_bru_total;
        }
    }

    /// Registro 0140: Tabela de Cadastro de Estabelecimentos
    fn handle_0140(&mut self, sped_record: &SpedRecord) {
        if let Ok(r) = sped_record.downcast_ref::<Registro0140>()
            && let Some(cnpj) = &r.cnpj
            && let Some(nome) = &r.nome
            && !cnpj.is_empty()
            && !nome.is_empty()
        {
            self.estabelecimentos.insert(cnpj.clone(), nome.clone());
        }
    }

    /// Registro 0150: Tabela de Cadastro do Participante
    fn handle_0150(&mut self, sped_record: &SpedRecord) {
        if let Ok(r) = sped_record.downcast_ref::<Registro0150>()
            && let Some(cod_part) = &r.cod_part
            && let Some(nome) = &r.nome
            && !cod_part.is_empty()
            && !nome.is_empty()
        {
            let mut hash: HashMap<Arc<str>, Arc<str>> = HashMap::with_capacity(3);

            hash.insert("NOME".into(), nome.clone());

            if let Some(cnpj) = r.cnpj.as_ref().filter(|s| !s.is_empty()) {
                hash.insert("CNPJ".into(), cnpj.clone());
                self.nome_do_cnpj.insert(cnpj.clone(), nome.clone());
            }
            if let Some(cpf) = r.cpf.as_ref().filter(|s| !s.is_empty()) {
                hash.insert("CPF".into(), cpf.clone());
                self.nome_do_cpf.insert(cpf.clone(), nome.clone());
            }

            self.participantes.insert(cod_part.clone(), hash);
        }
    }

    /// Registro 0190: Identificação das Unidades de Medida
    fn handle_0190(&mut self, sped_record: &SpedRecord) {
        if let Ok(r) = sped_record.downcast_ref::<Registro0190>()
            && let Some(cod_unidade) = &r.unid
            && let Some(descricao) = &r.descr
            && !cod_unidade.is_empty()
            && !descricao.is_empty()
        {
            self.unidade_de_medida
                .insert(cod_unidade.clone(), descricao.clone());
        }
    }

    /// Registro 0200: Tabela de Identificação do Item (Produtos e Serviços)
    fn handle_0200(&mut self, sped_record: &SpedRecord) {
        if let Ok(r) = sped_record.downcast_ref::<Registro0200>()
            && let Some(cod_item) = &r.cod_item
            && !cod_item.is_empty()
        {
            let mut item_data: HashMap<Arc<str>, Arc<str>> = HashMap::with_capacity(5);

            // Helper local para reduzir boilerplate
            let mut add = |k: &str, v: &Option<Arc<str>>| {
                if let Some(val) = v {
                    item_data.insert(k.into(), val.clone());
                }
            };

            add("DESCR_ITEM", &r.descr_item);
            add("TIPO_ITEM", &r.tipo_item);
            add("COD_NCM", &r.cod_ncm);
            add("COD_GEN", &r.cod_gen);
            add("COD_LST", &r.cod_lst);

            self.produtos.insert(cod_item.clone(), item_data);
        }
    }

    /// Registro 0400: Tabela de Natureza da Operação/Prestação
    fn handle_0400(&mut self, sped_record: &SpedRecord) {
        // Registro 0400: Tabela de Natureza da Operação/Prestação
        if let Ok(r) = sped_record.downcast_ref::<Registro0400>()
            && let Some(cod_nat) = &r.cod_nat
            && let Some(descr_nat) = &r.descr_nat
            && !cod_nat.is_empty()
            && !descr_nat.is_empty()
        {
            self.nat_operacao.insert(cod_nat.clone(), descr_nat.clone());
        }
    }

    /// Registro 0450: Tabela de Informação Complementar do Documento Fiscal
    fn handle_0450(&mut self, sped_record: &SpedRecord) {
        if let Ok(r) = sped_record.downcast_ref::<Registro0450>()
            && let Some(cod_inf) = &r.cod_inf
            && let Some(txt) = &r.txt
            && !cod_inf.is_empty()
            && !txt.is_empty()
        {
            self.complementar.insert(cod_inf.clone(), txt.clone());
        }
    }

    /// Registro 0500: Plano de Contas Contábeis
    fn handle_0500(&mut self, sped_record: &SpedRecord) {
        if let Ok(r) = sped_record.downcast_ref::<Registro0500>()
            && let Some(cod_conta) = &r.cod_cta
            && !cod_conta.is_empty()
        {
            // 1. Obtém o nome da conta normalizado (Option<Arc>)
            let nome_da_conta = r.nome_cta.to_upper_arc();

            // 2. Resolve o grupo de contas (se existir)
            //    Filtra strings vazias para transformar Some("") em None
            let grupo_de_contas = r
                .cod_nat_cc
                .as_deref()
                .map(obter_grupo_de_contas)
                .filter(|s| !s.is_empty());

            // 3. Match funcional para decidir o formato final
            //    Evita alocações (format!) a menos que necessário
            let conta_contabil: Arc<str> = match (grupo_de_contas, nome_da_conta) {
                (Some(grupo), Some(nome)) => {
                    // Caso ambos existam: Alocação necessária
                    Arc::from(format!("{}: {}", grupo, nome))
                }
                (Some(grupo), None) => {
                    // Apenas grupo: Alocação String -> Arc
                    Arc::from(grupo)
                }
                (None, Some(nome)) => {
                    // Apenas nome: Zero Copy (move o Arc existente)
                    nome
                }
                (None, None) => {
                    // Fallback
                    Arc::from("")
                }
            };

            self.contabil.insert(cod_conta.clone(), conta_contabil);
        }
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

    /// Obtém o nome mais frequente para cada Base de CNPJ (8 primeiros dígitos).
    ///
    /// Processa o mapa `nome_do_cnpj` (que contém CNPJ completo -> Nome) e retorna
    /// um mapa de `CNPJBase -> Nome`.
    ///
    /// Esta função utiliza uma abordagem funcional eficiente (`fold` + `entry`),
    /// mantendo a contagem e preservando o `Arc<str>` original para economizar memória.
    pub fn obter_nomes_por_cnpj_base(&self) -> HashMap<String, Arc<str>> {
        self.nome_do_cnpj
            .iter()
            // 1. Filtra nomes vazios
            .filter(|(_cnpj, name)| !name.trim().is_empty())
            // 2. Agrupa por CNPJ Base e conta frequências
            .fold(
                // Acumulador: Map<CNPJBase, Map<NomeLowerCase, (Contagem, NomeOriginalArc)>>
                HashMap::new(),
                |mut acc: HashMap<String, HashMap<String, (u32, Arc<str>)>>, (cnpj, name)| {
                    // Garante que temos ao menos 8 dígitos para a base
                    if cnpj.len() >= 8 {
                        let cnpj_base = cnpj[0..8].to_string();

                        acc.entry(cnpj_base)
                            .or_default()
                            .entry(name.to_lowercase()) // Chave normalizada para contagem agnóstica de case
                            .and_modify(|(count, _)| *count += 1)
                            .or_insert_with(|| (1, name.clone())); // Clona o Arc (barato), não a string
                    }
                    acc
                },
            )
            // 3. Transforma o acumulador no resultado final
            .into_iter()
            .filter_map(|(cnpj_base, counts_map)| {
                // Para cada base, pega o nome com maior contagem
                counts_map
                    .into_values()
                    // Em caso de empate, max_by_key escolhe o último (ou é arbitrário em HashMap),
                    // mas para contagem de nomes isso geralmente é suficiente.
                    .max_by_key(|(count, _)| *count)
                    .map(|(_, original_arc_name)| (cnpj_base, original_arc_name))
            })
            .collect()
    }
}
