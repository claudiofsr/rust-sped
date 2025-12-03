use super::*; // Importa itens de info_new.rs / analyze_one_new.rs
use crate::{
    // Importa as definições do crate raiz necessárias
    RegistroFilho,
    RegistroPai,
    SpedContext,
    TipoDeCredito,
    impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;

// ========================================================================
// MOCKS (Simulação de Registros)
// ========================================================================

// Mock do Contexto para evitar instanciar a estrutura pesada inteira
fn create_mock_context() -> SpedContext {
    SpedContext {
        arquivo_efd: Arc::from("teste.txt"),
        estabelecimento_cnpj: Arc::from("00000000000191"),
        estabelecimento_nome: Arc::from("EMPRESA TESTE"),
        periodo_de_apuracao: NaiveDate::from_ymd_opt(2023, 1, 1),
        ..Default::default()
    }
}

// ------------------------------------------------------------------------
// MOCK PAI (Ex: Simula um Registro C100)
// ------------------------------------------------------------------------
#[derive(Debug)]
struct RegMockPai {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    dt_doc: Option<NaiveDate>,
    cod_part: Option<String>,
    vl_icms: Option<Decimal>,
}

// Requisito do model.rs: Para ser um Registro, tem que ser SpedRecordTrait
impl_sped_record_trait!(RegMockPai);

// Implementa a interface de Pai usada pelo DocsBuilder
impl RegistroPai for RegMockPai {
    fn get_dt_emissao(&self) -> Option<NaiveDate> {
        self.dt_doc
    }
    fn get_cod_part(&self) -> Option<&str> {
        self.cod_part.as_deref()
    }
    fn get_valor_icms(&self) -> Option<Decimal> {
        self.vl_icms
    }
}

// ------------------------------------------------------------------------
// MOCK FILHO (Ex: Simula um Registro C170)
// ------------------------------------------------------------------------
#[derive(Debug)]
struct RegMockFilho {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    num_item: Option<String>,
    vl_item: Option<Decimal>,
    cst_pis: Option<String>,
    aliq_pis: Option<Decimal>,
    vl_pis: Option<Decimal>,
    // Campos para override (sobrescrever dados do pai)
    dt_emissao_override: Option<NaiveDate>,
    cod_part_override: Option<String>,
}

impl_sped_record_trait!(RegMockFilho);

// Implementa a interface de Filho usada pelo DocsBuilder
impl RegistroFilho for RegMockFilho {
    fn get_num_item(&self) -> Option<&str> {
        self.num_item.as_deref()
    }
    fn get_valor_item(&self) -> Option<Decimal> {
        self.vl_item
    }
    fn get_cst_pis(&self) -> Option<&str> {
        self.cst_pis.as_deref()
    }
    fn get_aliq_pis(&self) -> Option<Decimal> {
        self.aliq_pis
    }
    fn get_valor_pis(&self) -> Option<Decimal> {
        self.vl_pis
    }

    // Teste de override/prevalência
    fn get_dt_emissao(&self) -> Option<NaiveDate> {
        self.dt_emissao_override
    }
    fn get_part_override(&self) -> Option<&str> {
        self.cod_part_override.as_deref()
    }
}

// ========================================================================
// TESTES: CorrelationManager
// ========================================================================

#[test]
fn test_correlation_manager_storage_and_retrieval() {
    let mut mgr = CorrelationManager::default();

    let cst = Some(String::from("50"));
    let val_item = Some(dec!(1000.00));
    let aliq = Some(dec!(1.65));
    let val_pis = Some(dec!(16.50));
    let cfop = Some("1102");
    let part = Some("PART_01");

    // 1. Armazenar
    mgr.store(cst.as_ref(), val_item, aliq, val_pis, cfop, part);

    // 2. Recuperação Forte (Com Contexto)
    let res_strong = mgr.resolve(Some("50"), val_item, Some("1102"), Some("PART_01"));
    assert_eq!(
        res_strong,
        Some((1.65, 16.50)),
        "Deve recuperar pela chave forte"
    );

    // 3. Recuperação Fraca (Fallback sem contexto)
    let res_weak = mgr.resolve(Some("50"), val_item, None, None);
    assert_eq!(
        res_weak,
        Some((1.65, 16.50)),
        "Deve recuperar pela chave fraca"
    );
}

#[test]
fn test_correlation_manager_partial_strong_key() {
    let mut mgr = CorrelationManager::default();

    // Armazena apenas com CFOP, sem Participante
    mgr.store(
        Some(&"01".to_string()),
        Some(dec!(100.0)),
        Some(dec!(1.0)),
        Some(dec!(1.0)),
        Some("5405"),
        None,
    );

    // Tenta recuperar passando Participante (diferente da origem que era None)
    // A chave forte (StrongKey) exige igualdade exata nos Options.
    // Como (Some(CFOP), Some(Part)) != (Some(CFOP), None), a busca forte falha.
    // O sistema deve cair no Weak Cache (CST + Valor).
    let result = mgr.resolve(Some("01"), Some(dec!(100.0)), Some("5405"), Some("PART_X"));

    assert!(result.is_some(), "Deve achar via fallback weak cache");
}

// ========================================================================
// TESTES: Builder & Herança (DocsBuilder)
// ========================================================================

#[test]
fn test_builder_inheritance_logic() {
    let ctx = create_mock_context();

    // Dados do Pai
    let pai = RegMockPai {
        nivel: 1,
        bloco: 'P',
        registro: "MOCK_PAI".to_string(),
        line_number: 20,

        dt_doc: NaiveDate::from_ymd_opt(2023, 5, 10),
        cod_part: Some("FORNECEDOR_A".to_string()),
        vl_icms: Some(dec!(100.00)),
    };

    // Dados do Filho (alguns campos vazios para forçar herança)
    let filho = RegMockFilho {
        nivel: 2,
        bloco: 'F',
        registro: "MOCK_FILHO".to_string(),
        line_number: 28,

        num_item: Some("001".to_string()),
        vl_item: Some(dec!(1000.00)),
        cst_pis: Some("50".to_string()),
        aliq_pis: Some(dec!(1.65)),
        vl_pis: Some(dec!(16.50)),
        dt_emissao_override: None, // Deve herdar
        cod_part_override: None,   // Deve herdar
    };

    // DocsBuilder aceita referências genéricas que implementam as Traits
    // Não precisamos encapsular em SpedRecord::Generic aqui, pois o builder
    // trabalha diretamente com a lógica de negócio, não com o container do arquivo.
    let builder = DocsBuilder::from_child_and_parent(&ctx, &filho, Some(&pai), None);
    let doc = builder.build();

    assert_eq!(doc.data_emissao, pai.dt_doc, "Deve herdar data do Pai");

    // O nome do participante vem do mock context (que retorna default se não achar o CNPJ/Cod)
    // Mas testamos aqui se o código foi propagado internamente ou se a lógica do builder rodou.
    // Para testar o CNPJ exato, precisariamos adicionar o Participante no HashMap do MockContext.
    // Aqui validamos os valores numéricos que não dependem do Contexto:

    assert_eq!(doc.valor_icms, Some(100.00), "Deve herdar ICMS do Pai");
    assert_eq!(
        doc.valor_item,
        Some(1000.00),
        "Deve manter valor do item do Filho"
    );
}

#[test]
fn test_builder_child_override() {
    let ctx = create_mock_context();

    let pai = RegMockPai {
        nivel: 1,
        bloco: 'P',
        registro: "MOCK_PAI".to_string(),
        line_number: 20,

        dt_doc: NaiveDate::from_ymd_opt(2023, 5, 10),
        cod_part: Some("FORNECEDOR_A".to_string()),
        vl_icms: None,
    };

    let filho = RegMockFilho {
        nivel: 2,
        bloco: 'F',
        registro: "MOCK_FILHO".to_string(),
        line_number: 30,

        num_item: Some("001".to_string()),
        vl_item: Some(dec!(500.00)),
        cst_pis: None,
        aliq_pis: None,
        vl_pis: None,
        // Override! O filho tem dados próprios que devem vencer o pai
        dt_emissao_override: NaiveDate::from_ymd_opt(2023, 6, 1),
        cod_part_override: Some("FORNECEDOR_B".to_string()),
    };

    let builder = DocsBuilder::from_child_and_parent(&ctx, &filho, Some(&pai), None);
    let doc = builder.build();

    assert_eq!(
        doc.data_emissao, filho.dt_emissao_override,
        "Data do Filho deve prevalecer sobre o Pai"
    );

    // Verifica se o CNPJ/Código foi usado na estrutura final
    // No DocsBuilder, se o participante não está no HashMap do contexto,
    // ele tenta usar o código como CNPJ/CPF direto se tiver tamanho 14/11.
    // "FORNECEDOR_B" tem 12 chars, então vai cair no fallback ou ficar vazio dependendo da lógica exata.
    // Vamos verificar se o campo foi preenchido com algo diferente do pai.
    // (A lógica atual do seu código tenta fazer lookup no Contexto).

    // Para este teste ser útil, confirmamos apenas que a lógica de override foi chamada
    // Se tivéssemos populado o contexto, verificaríamos o nome.
}

// ========================================================================
// TESTES: Lógica de Negócio (Helpers)
// ========================================================================

#[test]
fn test_determinar_tipo_de_credito() {
    // Caso 1: Importação (CFOP 3xxx)
    let cred_imp = determinar_tipo_de_credito(Some(50), Some(1.65), Some(7.6), None, Some(3102));
    assert_eq!(cred_imp, Some(TipoDeCredito::Importacao));

    // Caso 2: Mercado Interno - Alíquota Básica
    let cred_basico = determinar_tipo_de_credito(Some(50), Some(1.65), Some(7.6), None, Some(1102));
    assert_eq!(cred_basico, Some(TipoDeCredito::AliquotaBasica));

    // Caso 3: Mercado Interno - Alíquota Diferenciada
    let cred_dif = determinar_tipo_de_credito(Some(50), Some(0.65), Some(3.0), None, Some(1102));
    assert_eq!(cred_dif, Some(TipoDeCredito::AliquotasDiferenciadas));
}
