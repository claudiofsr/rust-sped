use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use crate::{EFDResult, GrupoDeContas, SpedFile, SpedRecord, blocos::*};

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

    // Cache do nome do arquivo para uso repetitivo
    pub arquivo_efd: Arc<str>,

    // --- Metadados Globais (Registro 0000 e afins) ---
    // Registro0000
    pub matriz_estabelecimento_cnpj: Arc<str>,
    pub matriz_estabelecimento_nome: Arc<str>,

    // Estabelecimento das Filiais
    pub estabelecimento_cnpj: Arc<str>,
    pub estabelecimento_nome: Arc<str>,

    // Matriz e filiais devem possuir o mesmo CNPJ Base
    pub estabelecimento_cnpj_base: Arc<str>,

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
    pub participantes: BTreeMap<Arc<str>, Registro0150>,
    pub produtos: BTreeMap<Arc<str>, Registro0200>,
    pub unidade_de_medida: HashMap<Arc<str>, Arc<str>>,

    // Cache de Nomes para Lookup Rápido
    // Usamos BTreeMap para manter ordem e facilitar buscas determinísticas se necessário
    pub nome_do_cnpj: BTreeMap<Arc<str>, Arc<str>>,
    pub nome_do_cpf: BTreeMap<Arc<str>, Arc<str>>,

    // --- Cache de Nomes Mais Frequentes (Calculado em new) ---
    // Chave: String (Base do CNPJ ou CPF) -> Valor: Arc<str> (Nome original compartilhado)
    pub cache_nomes_cnpj_base: HashMap<String, Arc<str>>,
    pub cache_nomes_cpf_base: HashMap<String, Arc<str>>,
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

        // --- Geração única das tabelas de frequência ---
        // CNPJ Base: 8 primeiros dígitos
        ctx.cache_nomes_cnpj_base = Self::consolidar_nomes_mais_frequentes(&ctx.nome_do_cnpj, 8);

        // CPF Base: 9 primeiros dígitos (Identificação da Pessoa Física antes do dígito verificador)
        // ou 11 para CPF completo se preferir não agrupar. Usaremos 11 para exatidão ou 9 para base.
        // Geralmente CPF não varia de dono como CNPJ de filial, mas corrige erros de digitação.
        ctx.cache_nomes_cpf_base = Self::consolidar_nomes_mais_frequentes(&ctx.nome_do_cpf, 9);

        Ok(ctx)
    }

    // --- Handlers Específicos (Inline logic separation) ---

    /// Registro 0000: Abertura do Arquivo Digital e Identificação da Pessoa Jurídica
    fn handle_0000(&mut self, sped_record: &SpedRecord) {
        if let Ok(r) = sped_record.downcast_ref::<Registro0000>() {
            self.matriz_estabelecimento_cnpj = r.cnpj.clone();
            self.matriz_estabelecimento_nome = r.get_nome();
            self.estabelecimento_cnpj_base = r.get_cnpj_base();
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
            // Popula os caches de lookup rápido
            if let Some(cnpj) = r.cnpj.as_ref().filter(|s| !s.is_empty()) {
                self.nome_do_cnpj.insert(cnpj.clone(), nome.clone());
            }
            if let Some(cpf) = r.cpf.as_ref().filter(|s| !s.is_empty()) {
                self.nome_do_cpf.insert(cpf.clone(), nome.clone());
            }

            // Insere o registro inteiro. Muito mais limpo e rápido.
            self.participantes.insert(cod_part.clone(), r.clone());
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
            // Clone aqui é barato: copia u8/Decimal e incrementa Arcs.
            // Não aloca novas strings na Heap.
            self.produtos.insert(cod_item.clone(), r.clone());
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
    /// Ver utilização nos registros A110, C110, C500, D100, D500
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
            // 1. Resolve o grupo de contas usando o Enum GrupoDeContas
            // - as_deref(): Option<Arc<str>> -> Option<&str>
            // - and_then(): Tenta parsear. Se der erro (código inválido), vira None.
            // - map(): Se válido, formata a string "01 - Contas de Ativo"
            let grupo_de_contas: Option<String> = r
                .cod_nat_cc
                .as_deref()
                .and_then(|s| GrupoDeContas::from_str(s).ok())
                .map(|g| g.descricao_com_codigo());

            // 2. Match funcional para decidir o formato final
            //    Evita alocações (format!) a menos que necessário
            let conta_contabil: Arc<str> = match (grupo_de_contas, &r.nome_cta) {
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
                    nome.clone()
                }
                (None, None) => {
                    // A forma mais idiomática e garantidamente "grátis"
                    // de criar um Arc<str> vazio é usar Default:
                    Arc::default()
                }
            };

            self.contabil.insert(cod_conta.clone(), conta_contabil);
        }
    }

    /// Obter CNPJ do estabelecimento. Prioridade: Filiais -> Matriz
    pub fn obter_cnpj_do_estabelecimento(&self, current_cnpj: Option<Arc<str>>) -> Arc<str> {
        // Zero-copy se CNPJ não mudar
        current_cnpj.unwrap_or_else(|| self.matriz_estabelecimento_cnpj.clone())
    }

    /// Obter nome do estabelecimento. Prioridade: Filiais -> Matriz
    pub fn obter_nome_do_estabelecimento(&self, estabelecimento_cnpj: &Arc<str>) -> Arc<str> {
        self.estabelecimentos
            .get(estabelecimento_cnpj)
            // Clona o Arc<str> (apenas incrementa contador, nanosegundos)
            .cloned()
            // Se não achar, usa o fallback (também Arc clone barato)
            .unwrap_or_else(|| self.matriz_estabelecimento_nome.clone())
    }

    /// Consolidar nomes mais frequentes (Estilo Funcional).
    ///
    /// Identifica o nome mais comum para um determinado prefixo (Base do CNPJ ou CPF).
    /// Retorna um HashMap onde a chave é a Base (String) e o valor é o Nome Original (Arc<str>).
    fn consolidar_nomes_mais_frequentes(
        source: &BTreeMap<Arc<str>, Arc<str>>,
        slice_len: usize,
    ) -> HashMap<String, Arc<str>> {
        source
            .iter()
            // 1. Filtragem inicial: descarta chaves curtas ou nomes vazios
            //    Isso é o "Fail Fast" no pipeline.
            .filter(|(doc, name)| doc.len() >= slice_len && !name.trim().is_empty())
            // 2. Agregação (Reduce/Fold): Constrói o mapa de frequências
            //    Map<Base, Map<NomeNormalizado, (Contagem, NomeOriginal)>>
            .fold(
                HashMap::new(),
                |mut acc: HashMap<String, BTreeMap<String, (u32, Arc<str>)>>, (doc, name)| {
                    // Alocação necessária: A chave do mapa deve ser dona dos seus dados (String),
                    // pois não podemos referenciar um slice de um Arc<str> armazenado em outro lugar.
                    let base = doc[0..slice_len].to_string();

                    // Normalização para agrupar "EMPRESA X" e "Empresa X"
                    let nome_norm = name.trim().to_uppercase();

                    acc.entry(base)
                        .or_default() // Cria um BTreeMap vazio se não existir
                        .entry(nome_norm) // BTreeMap ordena por esta chave (String)
                        .and_modify(|(count, _)| *count += 1)
                        .or_insert_with(|| (1, name.clone())); // Clone barato do Arc (ponteiro)

                    acc
                },
            )
            // 3. Seleção do Vencedor: Transforma o acumulador no resultado final
            .into_iter()
            .filter_map(|(base, counts_map)| {
                // counts_map.into_values() consome o mapa interno retornando os valores
                // Para cada base, pega o nome com maior contagem
                counts_map
                    .into_values()
                    // Encontra a tupla com maior contagem (retorna Option)
                    // Determinismo garantido:
                    // Como a iteração do BTreeMap é ordenada (alfabética),
                    // em caso de empate de contagem, o max_by_key pegará consistentemente
                    // o último em ordem alfabética.
                    .max_by_key(|(count, _)| *count)
                    // Se encontrou (Some), mapeia para o formato final (Base, NomeOriginal)
                    // Se não encontrou (None), o filter_map descarta esta entrada silenciosamente.
                    .map(|(_, original_arc_name)| (base, original_arc_name))
            })
            .collect()
    }

    /// Tenta obter o nome pelo CNPJ completo, ou cai para o fallback do CNPJ Base mais frequente.
    pub fn obter_nome_por_cnpj(&self, cnpj: &str) -> Option<Arc<str>> {
        // 1. Tenta match exato (O(log n) no BTreeMap)
        if let Some(nome) = self.nome_do_cnpj.get(cnpj) {
            return Some(nome.clone());
        }

        // 2. Tenta match pela base (O(1) no HashMap)
        if cnpj.len() >= 8 {
            return self.cache_nomes_cnpj_base.get(&cnpj[0..8]).cloned();
        }

        None
    }

    /// Tenta obter o nome pelo CPF completo.
    pub fn obter_nome_por_cpf(&self, cpf: &str) -> Option<Arc<str>> {
        // 1. Tenta match exato
        if let Some(nome) = self.nome_do_cpf.get(cpf) {
            return Some(nome.clone());
        }

        // 2. Tenta match pela base (se configurou slice_len=11, isso é redundante, mas seguro)
        // Se configurou slice_len=9, isso ajuda a achar o nome mesmo com digito verificador errado.
        let len_base = 9; // Deve bater com o configurado em `new`
        if cpf.len() >= len_base {
            return self.cache_nomes_cpf_base.get(&cpf[0..len_base]).cloned();
        }

        None
    }

    // Helper unificado usado pelo Builder
    pub fn obter_nome_participante_inteligente(&self, codigo: &str) -> Option<Arc<str>> {
        match codigo.len() {
            14 => self.obter_nome_por_cnpj(codigo),
            11 => self.obter_nome_por_cpf(codigo),
            _ => None,
        }
    }
}
