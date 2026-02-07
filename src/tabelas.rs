use std::{
    collections::{HashMap, HashSet},
    fmt,
    path::Path,
    str::FromStr,
    sync::LazyLock,
};

use rust_decimal::{Decimal, prelude::FromPrimitive};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize, Serializer};

use crate::{
    ALIQ_BASICA_COF, ALIQ_BASICA_PIS, DECIMAL_ALIQ, EFDError, EFDResult, RegistroM500,
    RegistroM505, ResultExt,
};

// ============================================================================
// Tributo: PIS e COFINS
// ============================================================================

/**
Representa os tributos federais analisados na EFD Contribuições.

Tributos (Contribuições):

- PIS/PASEP
- COFINS
*/
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash, Serialize, Deserialize)]
pub enum Tributo {
    #[serde(rename = "PIS/PASEP")]
    Pis,
    #[serde(rename = "COFINS")]
    Cofins,
}

// https://docs.rs/serde/latest/serde/ser/trait.Serializer.html#method.collect_str
impl fmt::Display for Tributo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.serialize(f)
    }
}

// ============================================================================
// Meses do Ano
// ============================================================================

#[repr(u8)] // Opcional: Garante que o enum caiba em um u8 na memória
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum MesesDoAno {
    #[serde(rename = "Janeiro")]
    Janeiro = 1,

    #[serde(rename = "Fevereiro")]
    Fevereiro = 2,

    #[serde(rename = "Março")]
    Marco = 3,

    #[serde(rename = "Abril")]
    Abril = 4,

    #[serde(rename = "Maio")]
    Maio = 5,

    #[serde(rename = "Junho")]
    Junho = 6,

    #[serde(rename = "Julho")]
    Julho = 7,

    #[serde(rename = "Agosto")]
    Agosto = 8,

    #[serde(rename = "Setembro")]
    Setembro = 9,

    #[serde(rename = "Outubro")]
    Outubro = 10,

    #[serde(rename = "Novembro")]
    Novembro = 11,

    #[serde(rename = "Dezembro")]
    Dezembro = 12,
}

impl FromStr for MesesDoAno {
    type Err = EFDError;

    fn from_str(s: &str) -> EFDResult<Self> {
        match s.trim() {
            "1" | "01" => Ok(Self::Janeiro),
            "2" | "02" => Ok(Self::Fevereiro),
            "3" | "03" => Ok(Self::Marco),
            "4" | "04" => Ok(Self::Abril),
            "5" | "05" => Ok(Self::Maio),
            "6" | "06" => Ok(Self::Junho),
            "7" | "07" => Ok(Self::Julho),
            "8" | "08" => Ok(Self::Agosto),
            "9" | "09" => Ok(Self::Setembro),
            "10" => Ok(Self::Outubro),
            "11" => Ok(Self::Novembro),
            "12" => Ok(Self::Dezembro),
            _ => Err(EFDError::InvalidDate).loc(),
        }
    }
}

impl TryFrom<u32> for MesesDoAno {
    type Error = EFDError; // Você pode criar um erro customizado se preferir

    fn try_from(v: u32) -> EFDResult<Self> {
        match v {
            1 => Ok(Self::Janeiro),
            2 => Ok(Self::Fevereiro),
            3 => Ok(Self::Marco),
            4 => Ok(Self::Abril),
            5 => Ok(Self::Maio),
            6 => Ok(Self::Junho),
            7 => Ok(Self::Julho),
            8 => Ok(Self::Agosto),
            9 => Ok(Self::Setembro),
            10 => Ok(Self::Outubro),
            11 => Ok(Self::Novembro),
            12 => Ok(Self::Dezembro),
            _ => Err(EFDError::InvalidDate).loc(), // Retorna erro se o mês não for 1-13
        }
    }
}

// Permite usar .to_string() ou println!("{}", enum) usando a string definida no serde
impl fmt::Display for MesesDoAno {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Usa a implementação de Serialize para escrever a string (rename) no formatter
        self.serialize(f)
    }
}

// ============================================================================
// Indicador de Origem
// ============================================================================

// MercadoInterno = 0, o valor = 0 pode ser omitido, mas em sistemas fiscais/contábeis,
// é boa prática manter para indicar que aqueles números são regras imutáveis.

#[repr(u8)] // Opcional: Garante que o enum caiba em um u8 na memória
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum IndicadorDeOrigem {
    // Valores explícitos para garantir que o Enum
    // esteja sempre alinhado com a documentação do SPED
    #[serde(rename = "Operação no Mercado Interno")]
    MercadoInterno = 0,

    #[serde(rename = "Operação de Importação")]
    Importacao = 1,
}

impl FromStr for IndicadorDeOrigem {
    type Err = EFDError;

    fn from_str(s: &str) -> EFDResult<Self> {
        // .trim() remove espaços em branco acidentais que possam vir do arquivo
        match s.trim() {
            "0" => Ok(Self::MercadoInterno),
            "1" => Ok(Self::Importacao),
            _ => Err(EFDError::KeyNotFound(s.to_string())).loc(),
        }
    }
}

impl TryFrom<u16> for IndicadorDeOrigem {
    type Error = EFDError;

    /// Converte u16 para o IndicadorDeOrigem de forma segura
    fn try_from(cod: u16) -> EFDResult<Self> {
        match cod {
            0 => Ok(Self::MercadoInterno),
            1 => Ok(Self::Importacao),
            _ => Err(EFDError::KeyNotFound(cod.to_string())).loc(),
        }
    }
}

impl fmt::Display for IndicadorDeOrigem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.serialize(f)
    }
}

// ============================================================================
// Tipo de Operação
// ============================================================================

/// Define o tipo de operação para fins de apuração e ajustes.
#[repr(u8)] // Opcional: Garante que o enum caiba em um u8 na memória
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
// Note: Não derivamos Serialize automaticamente aqui para ter controle
// sobre o agrupamento de nomes (ex: AjusteAcrescimo -> "Ajuste")
pub enum TipoDeOperacao {
    Entrada = 1,
    Saida = 2,
    AjusteAcrescimo = 3,   // "Ajuste de Acréscimo"
    AjusteReducao = 4,     // "Ajuste de Redução"
    DescontoNoPeriodo = 5, // "Desconto da Contribuição Apurada no Próprio Período"
    DescontoPosterior = 6, // "Desconto Efetuado em Período Posterior"
    Detalhamento = 7,
}

impl Serialize for TipoDeOperacao {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Define a String de saída para cada variante
        let nome = match self {
            Self::Entrada => "Entrada",
            Self::Saida => "Saída",
            Self::AjusteAcrescimo | Self::AjusteReducao => "Ajuste",
            Self::DescontoNoPeriodo | Self::DescontoPosterior => "Desconto",
            Self::Detalhamento => "Detalhamento",
        };
        serializer.serialize_str(nome)
    }
}

// Implementação de Deserialize manual caso precise ler de volta o nome "Ajuste"
// para um ID específico (geralmente assume-se o padrão, ex: Acréscimo)
// Se não for ler arquivos CSV gerados de volta para Struct, Deserialize pode ser omitido ou simplificado.

impl fmt::Display for TipoDeOperacao {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.serialize(f)
    }
}

impl TipoDeOperacao {
    /// Operaçoes de Ajuste de Acréscimo ou de Redução
    pub fn is_ajuste(&self) -> bool {
        matches!(self, Self::AjusteAcrescimo | Self::AjusteReducao)
    }

    /// Operaçoes de Desconto no Próprio Período ou em Período Posterior
    pub fn is_desconto(&self) -> bool {
        matches!(self, Self::DescontoNoPeriodo | Self::DescontoPosterior)
    }

    /// Operações de Entrada ou de Saída
    pub fn is_entrada_ou_saida(&self) -> bool {
        matches!(self, Self::Entrada | Self::Saida)
    }
}

// ============================================================================
// Tipo de Rateio (Dígito da Centena)
// ============================================================================

#[repr(u16)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum TipoDeRateio {
    #[serde(rename = "Receita Bruta Não Cumulativa: Tributada no Mercado Interno")]
    RecBrutaNCumTribMercInterno = 1,

    #[serde(rename = "Receita Bruta Não Cumulativa: Não Tributada no Mercado Interno")]
    RecBrutaNCumNTribMercInterno = 2,

    #[serde(rename = "Receita Bruta Não Cumulativa: de Exportação")]
    RecBrutaNCumDeExportacao = 3,

    #[serde(rename = "Receita Bruta Cumulativa")]
    RecBrutaCumulativa = 4,
}

impl TipoDeRateio {
    /// Converte u16 para o TipoDeRateio de forma segura
    pub const fn from_u16(cod: u16) -> Option<Self> {
        match cod {
            1 => Some(Self::RecBrutaNCumTribMercInterno),
            2 => Some(Self::RecBrutaNCumNTribMercInterno),
            3 => Some(Self::RecBrutaNCumDeExportacao),
            4 => Some(Self::RecBrutaCumulativa),
            _ => None,
        }
    }

    /// Converte o código de crédito (ex: 101, 205) para o Enum correspondente.
    /// Baseado na regra: cod / 100.
    pub fn from_codigo_credito(cod: u16) -> Option<Self> {
        Self::from_u16(cod / 100)
    }
}

impl fmt::Display for TipoDeRateio {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.serialize(f)
    }
}

// ============================================================================
// Tipo de Crédito (Dezenas e Unidades)
// ============================================================================

#[repr(u16)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum TipoDeCredito {
    #[serde(rename = "Alíquota Básica")]
    AliquotaBasica = 1,

    #[serde(rename = "Alíquotas Diferenciadas")]
    AliquotasDiferenciadas = 2,

    #[serde(rename = "Alíquota por Unidade de Produto")]
    AliquotaPorUnidadeProduto = 3,

    #[serde(rename = "Estoque de Abertura")]
    EstoqueAbertura = 4,

    #[serde(rename = "Aquisição Embalagens para Revenda")]
    AquisicaoEmbalagens = 5,

    #[serde(rename = "Presumido da Agroindústria")]
    PresumidoAgroindustria = 6,

    #[serde(rename = "Outros Créditos Presumidos")]
    OutrosCreditosPresumidos = 7,

    #[serde(rename = "Importação")]
    Importacao = 8,

    #[serde(rename = "Atividade Imobiliária")]
    AtividadeImobiliaria = 9,

    #[serde(rename = "Outros")]
    Outros = 99,
}

impl TipoDeCredito {
    /// Retorna o valor numérico do TipoDeCredito (ex: 1, ..., 9, 99).
    pub const fn code(self) -> u16 {
        self as u16
    }

    /// Converte u16 para o TipoDeCredito de forma segura
    pub const fn from_u16(cod: u16) -> Option<Self> {
        match cod {
            1 => Some(Self::AliquotaBasica),
            2 => Some(Self::AliquotasDiferenciadas),
            3 => Some(Self::AliquotaPorUnidadeProduto),
            4 => Some(Self::EstoqueAbertura),
            5 => Some(Self::AquisicaoEmbalagens),
            6 => Some(Self::PresumidoAgroindustria),
            7 => Some(Self::OutrosCreditosPresumidos),
            8 => Some(Self::Importacao),
            9 => Some(Self::AtividadeImobiliaria),
            99 => Some(Self::Outros),
            _ => None,
        }
    }

    #[rustfmt::skip]
    /// Mapeia o Tipo de Crédito para sua respectiva Natureza de Soma (Base de Cálculo)
    pub fn para_natureza_soma(&self) -> Option<NaturezaBaseCalculo> {
        match self {
            Self::AliquotaBasica => Some(NaturezaBaseCalculo::BaseSomaAliquotaBasica),
            Self::AliquotasDiferenciadas => Some(NaturezaBaseCalculo::BaseSomaAliquotasDiferenciadas),
            Self::AliquotaPorUnidadeProduto => Some(NaturezaBaseCalculo::BaseSomaAliquotaUnidade),
            Self::EstoqueAbertura => Some(NaturezaBaseCalculo::BaseSomaEstoqueAbertura),
            Self::AquisicaoEmbalagens => Some(NaturezaBaseCalculo::BaseSomaAquisicaoEmbalagens),
            Self::PresumidoAgroindustria => Some(NaturezaBaseCalculo::BaseSomaPresumidoAgroindustria),
            Self::OutrosCreditosPresumidos => Some(NaturezaBaseCalculo::BaseSomaOutrosCreditosPresumidos),
            Self::Importacao => Some(NaturezaBaseCalculo::BaseSomaImportacao),
            Self::AtividadeImobiliaria => Some(NaturezaBaseCalculo::BaseSomaAtividadeImobiliaria),
            Self::Outros => Some(NaturezaBaseCalculo::BaseSomaOutros),
        }
    }

    /// Converte o código de crédito (ex: 101, 205) para o Enum correspondente.
    /// Baseado na regra: cod % 100.
    pub fn from_codigo_credito(cod: u16) -> Option<Self> {
        Self::from_u16(cod % 100)
    }

    /// Retorna a descrição formatada com o código (ex: "01 - Alíquota Básica")
    pub fn descricao_com_codigo(&self) -> String {
        // Como 'self' é o próprio enum, ele sempre tem valor.
        // O cast (*self as u16) funciona por causa do #[repr(u16)]
        // O uso de 'self' na string funciona por causa do impl Display
        format!("{:02} - {}", *self as u16, self)
    }
}

// Implementação do Display usando o Serializer do Serde para exibir a descrição
impl fmt::Display for TipoDeCredito {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.serialize(f)
    }
}

// ============================================================================
// Código do Crédito
// ============================================================================

/// Representa o Código do Crédito (XYY) informado nos Blocos M e 1.
/// X (centena) = Tipo de Rateio (1 a 4).
/// YY (resto)  = Tipo de Crédito (01 a 99).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct CodigoDoCredito {
    pub rateio: TipoDeRateio,
    pub credito: TipoDeCredito,
}

impl CodigoDoCredito {
    /// Tenta criar um CodigoDoCredito a partir de um u16 bruto.
    ///
    /// Valida se a centena corresponde a um TipoDeRateio válido
    /// e se as dezenas/unidades correspondem a um TipoDeCredito válido.
    pub fn new(val: u16, arquivo: &Path, linha_num: usize, campo_nome: &str) -> EFDResult<Self> {
        let x = val / 100;
        let yy = val % 100;

        // Se from_u16 retornar None, geramos o erro rico imediatamente
        let rateio = TipoDeRateio::from_u16(x).map_loc(|_| EFDError::InvalidField {
            arquivo: arquivo.to_path_buf(),
            linha_num,
            campo: campo_nome.to_string(),
            valor: val.to_string(),
            detalhe: Some(format!(
                "Dígito da centena (Rateio) '{}' inválido (esperado 1-4)",
                x
            )),
        })?;

        let credito = TipoDeCredito::from_u16(yy).map_loc(|_| EFDError::InvalidField {
            arquivo: arquivo.to_path_buf(),
            linha_num,
            campo: campo_nome.to_string(),
            valor: val.to_string(),
            detalhe: Some(format!(
                "2 Dígitos finais (Tipo de Crédito) '{:02}' inválidos",
                yy
            )),
        })?;

        Ok(Self { rateio, credito })
    }

    /// Converte o código de volta para a representação numérica XYY.
    pub const fn to_u16(&self) -> u16 {
        (self.rateio as u16 * 100) + (self.credito as u16)
    }

    /// Verifica se o código refere-se a uma operação de Importação (YY = 08).
    pub fn eh_importacao(&self) -> bool {
        matches!(self.credito, TipoDeCredito::Importacao)
    }
}

impl fmt::Display for CodigoDoCredito {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:03}", self.to_u16())
    }
}

// ============================================================================
// Tipo do Item
// ============================================================================

/// 4.3.1 - Tabela Tipo do Item.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TipoDoItem {
    #[serde(rename = "Mercadoria para Revenda")]
    MercadoriaParaRevenda = 0,

    #[serde(rename = "Matéria-Prima")]
    MateriaPrima = 1,

    #[serde(rename = "Embalagem")]
    Embalagem = 2,

    #[serde(rename = "Produto em Processo")]
    ProdutoEmProcesso = 3,

    #[serde(rename = "Produto Acabado")]
    ProdutoAcabado = 4,

    #[serde(rename = "Subproduto")]
    Subproduto = 5,

    #[serde(rename = "Produto Intermediário")]
    ProdutoIntermediario = 6,

    #[serde(rename = "Material de Uso e Consumo")]
    MaterialDeUsoEConsumo = 7,

    #[serde(rename = "Ativo Imobilizado")]
    AtivoImobilizado = 8,

    #[serde(rename = "Serviços")]
    Servicos = 9,

    #[serde(rename = "Outros insumos")]
    OutrosInsumos = 10,

    #[serde(rename = "Outras")]
    Outras = 99,
}

impl TipoDoItem {
    /// Converte u8 para o TipoDoItem de forma segura.
    pub const fn from_u8(cod: u8) -> Option<Self> {
        match cod {
            0 => Some(Self::MercadoriaParaRevenda),
            1 => Some(Self::MateriaPrima),
            2 => Some(Self::Embalagem),
            3 => Some(Self::ProdutoEmProcesso),
            4 => Some(Self::ProdutoAcabado),
            5 => Some(Self::Subproduto),
            6 => Some(Self::ProdutoIntermediario),
            7 => Some(Self::MaterialDeUsoEConsumo),
            8 => Some(Self::AtivoImobilizado),
            9 => Some(Self::Servicos),
            10 => Some(Self::OutrosInsumos),
            99 => Some(Self::Outras),
            _ => None,
        }
    }

    /// Retorna a descrição formatada com o código (ex: "09 - Serviços")
    pub fn descricao_com_codigo(&self) -> String {
        // Como 'self' é o próprio enum, ele sempre tem valor.
        // O cast (*self as u8) funciona por causa do #[repr(u8)]
        // O uso de 'self' na string funciona por causa do impl Display
        format!("{:02} - {}", *self as u8, self)
    }
}

// Implementação do Display usando o Serializer do Serde para exibir a descrição
impl fmt::Display for TipoDoItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.serialize(f)
    }
}

// ============================================================================
// Grupo de Contas
// ============================================================================

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GrupoDeContas {
    #[serde(rename = "Contas de Ativo")]
    ContasDeAtivo = 1,

    #[serde(rename = "Contas de Passivo")]
    ContasDePassivo = 2,

    #[serde(rename = "Patrimônio Líquido")]
    PatrimonioLiquido = 3,

    #[serde(rename = "Contas de Resultado")]
    ContasDeResultado = 4,

    #[serde(rename = "Contas de Compensação")]
    ContasDeCompensacao = 5,

    #[serde(rename = "Outras")]
    Outras = 9,
}

impl FromStr for GrupoDeContas {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // .trim() remove espaços em branco (ex: " 01 ")
        match s.trim() {
            "01" | "1" => Ok(Self::ContasDeAtivo),
            "02" | "2" => Ok(Self::ContasDePassivo),
            "03" | "3" => Ok(Self::PatrimonioLiquido),
            "04" | "4" => Ok(Self::ContasDeResultado),
            "05" | "5" => Ok(Self::ContasDeCompensacao),
            "09" | "9" => Ok(Self::Outras),
            _ => Err(format!("Código de Grupo de Contas inválido: {}", s)),
        }
    }
}

impl GrupoDeContas {
    /// Tenta criar um CodigoDoCredito a partir de um u16 bruto.
    ///
    /// Valida se a centena corresponde a um TipoDeRateio válido
    /// e se as dezenas/unidades correspondem a um TipoDeCredito válido.
    pub fn new(cod: u8, arquivo: &Path, linha_num: usize, campo_nome: &str) -> EFDResult<Self> {
        // Se from_u8 retornar None, geramos o erro rico imediatamente
        GrupoDeContas::from_u8(cod).map_loc(|_| EFDError::InvalidField {
            arquivo: arquivo.to_path_buf(),
            linha_num,
            campo: campo_nome.to_string(),
            valor: cod.to_string(),
            detalhe: Some(format!(
                "Código do Grupo de Contas '{}' inválido (esperado 1-9)",
                cod
            )),
        })
    }

    /// Converte u8 para o GrupoDeContas de forma segura.
    pub const fn from_u8(cod: u8) -> Option<Self> {
        match cod {
            1 => Some(Self::ContasDeAtivo),
            2 => Some(Self::ContasDePassivo),
            3 => Some(Self::PatrimonioLiquido),
            4 => Some(Self::ContasDeResultado),
            5 => Some(Self::ContasDeCompensacao),
            9 => Some(Self::Outras),
            _ => None,
        }
    }

    /// Retorna o valor numérico (u8)
    pub const fn code(self) -> u8 {
        self as u8
    }

    /// Retorna a descrição formatada com o código (ex: "01 - Contas de Ativo")
    pub fn descricao_com_codigo(&self) -> String {
        // *self as u8 obtém o valor numérico (1, 2, 9...)
        // self (Display) obtém a descrição do #[serde(rename)]
        format!("{:02} - {}", *self as u8, self)
    }
}

// Implementação do Display usando o Serializer do Serde para exibir a descrição
impl fmt::Display for GrupoDeContas {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Isso utiliza o texto definido em #[serde(rename = "...")]
        self.serialize(f)
    }
}

// ============================================================================
// TABELA ESTÁTICA DE CORRELAÇÃO: Aliq COFINS --> Aliq PIS/PASEP
// ============================================================================

/// Tabela de correlação COFINS -> PIS (Lazy Initialization)
static CORRELACAO_COFINS_PIS: LazyLock<HashMap<Decimal, Decimal>> = LazyLock::new(|| {
    // Aliq COFINS --> Aliq PIS/PASEP
    [
        // (Aliq COFINS, Aliq PIS/PASEP)
        (0.00, 1.6500), // Alíquotas Diferenciadas
        (0.76, 0.1650),
        (1.52, 0.3300),
        (2.66, 0.5775),
        (3.80, 0.8250),
        (4.56, 0.9900),
        (5.70, 1.2375), // PIS: 1,2375 % (1,65% x 75%) e. – COFINS: 5,70% (7,60% x 75%). Subcontratação Serviços de Transporte
        (6.08, 1.3200),
        (7.00, 1.6500),
        (7.60, 1.6500),
        (8.54, 1.8600), // venda pelo atacadista ao varejista ou ao consumidor final
        (9.65, 2.1000),
        (10.68, 2.3200),
        (10.80, 2.3000),
        (14.37, 2.1000),
    ]
    .into_iter()
    .filter_map(|(cof, pis)| Some((Decimal::from_f64(cof)?, Decimal::from_f64(pis)?)))
    .collect()
});

/// Tenta obter a alíquota de PIS baseada na de COFINS usando a tabela estática.
pub fn obter_pis_da_tabela_estatica(pai: &RegistroM500, filho: &RegistroM505) -> Option<Decimal> {
    let aliq_cofins = pai.aliq_cofins?;
    match CORRELACAO_COFINS_PIS.get(&aliq_cofins) {
        Some(aliq_pis) => Some(*aliq_pis),
        None => {
            eprintln!("tabelas.rs");
            eprintln!("fn obter_aliquota_correlacionada_de_pis()");
            eprintln!(
                "Não foi possível obter por correlações a alíquota de PIS/PASEP relacionada à alíquota de COFINS: {aliq_cofins}"
            );
            eprintln!(
                "A correlação entre as alíquotas de PIS/PASEP e COFINS será obtida consultando a tabela CORRELACAO_COFINS_PIS"
            );
            eprintln!("{pai:?}");
            eprintln!("{filho:?}\n");
            None
        }
    }
}

// ============================================================================
// Tabela Modelos de Documentos Fiscais
// ============================================================================

/// 4.1.1- Tabela Modelos de Documentos Fiscais
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModeloDocFiscal {
    #[serde(rename = "Nota Fiscal")]
    NotaFiscal, // 01

    #[serde(rename = "Nota Fiscal Avulsa")]
    NotaFiscalAvulsa, // 1B

    #[serde(rename = "Nota Fiscal de Venda a Consumidor")]
    NotaFiscalVendaConsumidor, // 02

    #[serde(rename = "Cupom Fiscal emitido por ECF")]
    CupomFiscalECF, // 2D

    #[serde(rename = "Bilhete de Passagem emitido por ECF")]
    BilhetePassagemECF, // 2E

    #[serde(rename = "Nota Fiscal de Produtor")]
    NotaFiscalProdutor, // 04

    #[serde(rename = "Nota Fiscal / Conta de Energia Elétrica")]
    EnergiaEletrica, // 06

    #[serde(rename = "Nota Fiscal de Serviço de Transporte")]
    ServicoTransporte, // 07

    #[serde(rename = "Conhecimento de Transporte Rodoviário de Cargas")]
    ConhecimentoRodoviario, // 08

    #[serde(rename = "Conhecimento de Transporte de Cargas Avulso")]
    ConhecimentoAvulso, // 8B

    #[serde(rename = "Conhecimento de Transporte Aquaviário de Cargas")]
    ConhecimentoAquaviario, // 09

    #[serde(rename = "Conhecimento Aéreo")]
    ConhecimentoAereo, // 10

    #[serde(rename = "Conhecimento de Transporte Ferroviário de Cargas")]
    ConhecimentoFerroviario, // 11

    #[serde(rename = "Bilhete de Passagem Rodoviário")]
    BilheteRodoviario, // 13

    #[serde(rename = "Bilhete de Passagem Aquaviário")]
    BilheteAquaviario, // 14

    #[serde(rename = "Bilhete de Passagem e Nota de Bagagem")]
    BilheteNotaBagagem, // 15

    #[serde(rename = "Bilhete de Passagem Ferroviário")]
    BilheteFerroviario, // 16

    #[serde(rename = "Despacho de Transporte")]
    DespachoTransporte, // 17

    #[serde(rename = "Resumo de Movimento Diário")]
    ResumoMovimentoDiario, // 18

    #[serde(rename = "Ordem de Coleta de Cargas")]
    OrdemColetaCargas, // 20

    #[serde(rename = "Nota Fiscal de Serviço de Comunicação")]
    Comunicacao, // 21

    #[serde(rename = "Nota Fiscal de Serviço de Telecomunicação")]
    Telecomunicacao, // 22

    #[serde(rename = "GNRE")]
    GNRE, // 23

    #[serde(rename = "Autorização de Carregamento e Transporte")]
    AutorizacaoCarregamento, // 24

    #[serde(rename = "Manifesto de Carga")]
    ManifestoCarga, // 25

    #[serde(rename = "Conhecimento de Transporte Multimodal de Cargas")]
    ConhecimentoMultimodal, // 26

    #[serde(rename = "Nota Fiscal de Transporte Ferroviário de Cargas")]
    NotaFiscalFerroviario, // 27

    #[serde(rename = "Nota Fiscal / Conta de Fornecimento de Gás Canalizado")]
    GasCanalizado, // 28

    #[serde(rename = "Nota Fiscal / Conta de Fornecimento de Água Canalizada")]
    AguaCanalizada, // 29

    #[serde(rename = "Bilhete / Recibo do Passageiro")]
    BilhetePassageiro, // 30

    #[serde(rename = "Nota Fiscal Eletrônica: NF-e")]
    NFe, // 55

    #[serde(rename = "Conhecimento de Transporte Eletrônico: CT-e")]
    CTe, // 57

    #[serde(rename = "Cupom Fiscal Eletrônico: CF-e (CF-e-SAT)")]
    CFeSAT, // 59

    #[serde(rename = "Cupom Fiscal Eletrônico: CF-e-ECF")]
    CFeECF, // 60

    #[serde(rename = "Bilhete de Passagem Eletrônico: BP-e")]
    BPe, // 63

    #[serde(rename = "Nota Fiscal Eletrônica ao Consumidor Final: NFC-e")]
    NFCe, // 65

    #[serde(rename = "Nota Fiscal de Energia Elétrica Eletrônica: NF3e")]
    NF3e, // 66

    #[serde(rename = "Conhecimento de Transporte Eletrônico para Outros Serviços: CT-e OS")]
    CTeOS, // 67
}

impl ModeloDocFiscal {
    /// Retorna o código oficial (string) associado ao modelo.
    /// Necessário pois existem códigos alfanuméricos (ex: "1B").
    pub const fn codigo(&self) -> &'static str {
        match self {
            Self::NotaFiscal => "01",
            Self::NotaFiscalAvulsa => "1B",
            Self::NotaFiscalVendaConsumidor => "02",
            Self::CupomFiscalECF => "2D",
            Self::BilhetePassagemECF => "2E",
            Self::NotaFiscalProdutor => "04",
            Self::EnergiaEletrica => "06",
            Self::ServicoTransporte => "07",
            Self::ConhecimentoRodoviario => "08",
            Self::ConhecimentoAvulso => "8B",
            Self::ConhecimentoAquaviario => "09",
            Self::ConhecimentoAereo => "10",
            Self::ConhecimentoFerroviario => "11",
            Self::BilheteRodoviario => "13",
            Self::BilheteAquaviario => "14",
            Self::BilheteNotaBagagem => "15",
            Self::BilheteFerroviario => "16",
            Self::DespachoTransporte => "17",
            Self::ResumoMovimentoDiario => "18",
            Self::OrdemColetaCargas => "20",
            Self::Comunicacao => "21",
            Self::Telecomunicacao => "22",
            Self::GNRE => "23",
            Self::AutorizacaoCarregamento => "24",
            Self::ManifestoCarga => "25",
            Self::ConhecimentoMultimodal => "26",
            Self::NotaFiscalFerroviario => "27",
            Self::GasCanalizado => "28",
            Self::AguaCanalizada => "29",
            Self::BilhetePassageiro => "30",
            Self::NFe => "55",
            Self::CTe => "57",
            Self::CFeSAT => "59",
            Self::CFeECF => "60",
            Self::BPe => "63",
            Self::NFCe => "65",
            Self::NF3e => "66",
            Self::CTeOS => "67",
        }
    }

    /// Retorna a descrição formatada com o código (ex: "55 - Nota Fiscal Eletrônica: NF-e")
    pub fn descricao_com_codigo(&self) -> String {
        format!("{} - {}", self.codigo(), self)
    }
}

impl FromStr for ModeloDocFiscal {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "01" => Ok(Self::NotaFiscal),
            "1B" => Ok(Self::NotaFiscalAvulsa),
            "02" => Ok(Self::NotaFiscalVendaConsumidor),
            "2D" => Ok(Self::CupomFiscalECF),
            "2E" => Ok(Self::BilhetePassagemECF),
            "04" => Ok(Self::NotaFiscalProdutor),
            "06" => Ok(Self::EnergiaEletrica),
            "07" => Ok(Self::ServicoTransporte),
            "08" => Ok(Self::ConhecimentoRodoviario),
            "8B" => Ok(Self::ConhecimentoAvulso),
            "09" => Ok(Self::ConhecimentoAquaviario),
            "10" => Ok(Self::ConhecimentoAereo),
            "11" => Ok(Self::ConhecimentoFerroviario),
            "13" => Ok(Self::BilheteRodoviario),
            "14" => Ok(Self::BilheteAquaviario),
            "15" => Ok(Self::BilheteNotaBagagem),
            "16" => Ok(Self::BilheteFerroviario),
            "17" => Ok(Self::DespachoTransporte),
            "18" => Ok(Self::ResumoMovimentoDiario),
            "20" => Ok(Self::OrdemColetaCargas),
            "21" => Ok(Self::Comunicacao),
            "22" => Ok(Self::Telecomunicacao),
            "23" => Ok(Self::GNRE),
            "24" => Ok(Self::AutorizacaoCarregamento),
            "25" => Ok(Self::ManifestoCarga),
            "26" => Ok(Self::ConhecimentoMultimodal),
            "27" => Ok(Self::NotaFiscalFerroviario),
            "28" => Ok(Self::GasCanalizado),
            "29" => Ok(Self::AguaCanalizada),
            "30" => Ok(Self::BilhetePassageiro),
            "55" => Ok(Self::NFe),
            "57" => Ok(Self::CTe),
            "59" => Ok(Self::CFeSAT),
            "60" => Ok(Self::CFeECF),
            "63" => Ok(Self::BPe),
            "65" => Ok(Self::NFCe),
            "66" => Ok(Self::NF3e),
            "67" => Ok(Self::CTeOS),
            _ => Err(format!(
                "Código de Modelo de Documento Fiscal inválido: {}",
                s
            )),
        }
    }
}

// Implementação do Display usando o Serializer do Serde para exibir a descrição
impl fmt::Display for ModeloDocFiscal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.serialize(f)
    }
}

// ============================================================================
// Código da Natureza da Base de Cálculo dos Créditos
// ============================================================================

static CFOP_PARA_NATUREZA_BC: LazyLock<HashMap<u16, NaturezaBaseCalculo>> = LazyLock::new(|| {
    let mut info = HashMap::new();

    // Natureza 01 - Aquisição de Bens para Revenda
    let n01 = [
        1102, 1113, 1117, 1118, 1121, 1159, 1251, 1403, 1652, 2102, 2113, 2117, 2118, 2121, 2159,
        2251, 2403, 2652, 3102, 3251, 3652,
    ];

    // Natureza 02 - Aquisição de Bens Utilizados como Insumo
    let n02 = [
        1101, 1111, 1116, 1120, 1122, 1126, 1128, 1401, 1407, 1556, 1651, 1653, 2101, 2111, 2116,
        2120, 2122, 2126, 2128, 2401, 2407, 2556, 2651, 2653, 3101, 3126, 3128, 3556, 3651, 3653,
        1135, 2135, 1132, 2132, 1456, 2456,
    ];

    // Natureza 03 - Aquisição de Serviços Utilizados como Insumo
    let n03 = [1124, 1125, 1933, 2124, 2125, 2933];

    // Natureza 12 - Devolução de Vendas Sujeitas à Incidência Não-Cumulativa
    let n12 = [
        1201, 1202, 1203, 1204, 1410, 1411, 1660, 1661, 1662, 2201, 2202, 2410, 2411, 2660, 2661,
        2662, 1206, 2206, 1207, 2207, 1215, 1216, 2215, 2216,
    ];

    // Natureza 13 - Outras Operações com Direito a Crédito
    let n13 = [1922, 2922];

    for cfop in n01 {
        info.insert(cfop, NaturezaBaseCalculo::AquisicaoBensRevenda);
    }
    for cfop in n02 {
        info.insert(cfop, NaturezaBaseCalculo::AquisicaoBensInsumo);
    }
    for cfop in n03 {
        info.insert(cfop, NaturezaBaseCalculo::AquisicaoServicosInsumo);
    }
    for cfop in n12 {
        info.insert(cfop, NaturezaBaseCalculo::DevolucaoVendasNaoCumulativa);
    }
    for cfop in n13 {
        info.insert(cfop, NaturezaBaseCalculo::OutrasOperacoesComDireitoCredito);
    }

    info
});

/// Obtém a Natureza da BC baseada no CFOP e CST.
///
/// desde que:
///
/// 50 <= CST <= 56 ou 60 <= CST <= 66
///
/// e
///
/// CFOP seja um código de Operações com direito a créditos,
///
/// conforme Tabela “CFOP – Operações Geradoras de Créditos”.
pub fn obter_natureza_da_bc(
    cfop_opt: Option<u16>,
    cst_opt: Option<CodigoSituacaoTributaria>,
) -> Option<NaturezaBaseCalculo> {
    // 1. Verifica se o CST é de crédito (50..56 ou 60..66)
    if !cst_opt.is_some_and(|cst| cst.eh_base_de_credito()) {
        return None;
    }

    // 2. Busca a natureza no mapa através do CFOP
    cfop_opt.and_then(|cfop| CFOP_PARA_NATUREZA_BC.get(&cfop).copied())
}

// ============================================================================
// Natureza da Base de Cálculo dos Créditos
// ============================================================================

#[repr(u16)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum NaturezaBaseCalculo {
    #[serde(rename = "Aquisição de Bens para Revenda")]
    AquisicaoBensRevenda = 1,

    #[serde(rename = "Aquisição de Bens Utilizados como Insumo")]
    AquisicaoBensInsumo = 2,

    #[serde(rename = "Aquisição de Serviços Utilizados como Insumo")]
    AquisicaoServicosInsumo = 3,

    #[serde(rename = "Energia Elétrica e Térmica, Inclusive sob a Forma de Vapor")]
    EnergiaEletricaTermica = 4,

    #[serde(rename = "Aluguéis de Prédios")]
    AlugueisPredios = 5,

    #[serde(rename = "Aluguéis de Máquinas e Equipamentos")]
    AlugueisMaquinasEquipamentos = 6,

    #[serde(rename = "Armazenagem de Mercadoria e Frete na Operação de Venda")]
    ArmazenagemFreteVenda = 7,

    #[serde(rename = "Contraprestações de Arrendamento Mercantil")]
    ArrendamentoMercantil = 8,

    #[serde(rename = "Máquinas, Equipamentos ... (Crédito sobre Encargos de Depreciação)")]
    MaquinasEquipamentosDepreciacao = 9,

    #[serde(rename = "Máquinas, Equipamentos ... (Crédito com Base no Valor de Aquisição)")]
    MaquinasEquipamentosAquisicao = 10,

    #[serde(rename = "Amortizacao e Depreciação de Edificações e Benfeitorias em Imóveis")]
    AmortizacaoDepreciacaoEdificacoes = 11,

    #[serde(rename = "Devolução de Vendas Sujeitas à Incidência Não-Cumulativa")]
    DevolucaoVendasNaoCumulativa = 12,

    #[serde(rename = "Outras Operações com Direito a Crédito")]
    OutrasOperacoesComDireitoCredito = 13,

    #[serde(rename = "Atividade de Transporte de Cargas - Subcontratação")]
    TransporteCargasSubcontratacao = 14,

    #[serde(rename = "Atividade Imobiliária - Custo Incorrido de Unidade Imobiliária")]
    AtividadeImobiliariaCustoIncorrido = 15,

    #[serde(rename = "Atividade Imobiliária - Custo Orçado de Unidade não Concluída")]
    AtividadeImobiliariaCustoOrcado = 16,

    #[serde(rename = "Atividade de Prestação de Serviços de Limpeza, Conservação e Manutenção")]
    ServicosLimpezaConservacao = 17,

    #[serde(rename = "Estoque de Abertura de Bens")]
    EstoqueAberturaBens = 18,

    // Ajustes
    #[serde(rename = "Ajuste de Acréscimo (PIS/PASEP)")]
    AjusteAcrescimoPis = 31,

    #[serde(rename = "Ajuste de Acréscimo (COFINS)")]
    AjusteAcrescimoCofins = 35,

    #[serde(rename = "Ajuste de Redução (PIS/PASEP)")]
    AjusteReducaoPis = 41,

    #[serde(rename = "Ajuste de Redução (COFINS)")]
    AjusteReducaoCofins = 45,

    // Descontos
    #[serde(rename = "Desconto da Contribuição Apurada no Próprio Período (PIS/PASEP)")]
    DescontoProprioPeriodoPis = 51,

    #[serde(rename = "Desconto da Contribuição Apurada no Próprio Período (COFINS)")]
    DescontoProprioPeriodoCofins = 55,

    #[serde(rename = "Desconto Efetuado em Período Posterior (PIS/PASEP)")]
    DescontoPeriodoPosteriorPis = 61,

    #[serde(rename = "Desconto Efetuado em Período Posterior (COFINS)")]
    DescontoPeriodoPosteriorCofins = 65,

    // Agrupadores / Soma
    // Base de Cálculo dos Créditos
    #[serde(rename = "Base de Cálculo dos Créditos - Alíquota Básica (Soma)")]
    BaseSomaAliquotaBasica = 101,

    #[serde(rename = "Base de Cálculo dos Créditos - Alíquotas Diferenciadas (Soma)")]
    BaseSomaAliquotasDiferenciadas = 102,

    #[serde(rename = "Base de Cálculo dos Créditos - Alíquota por Unidade de Produto (Soma)")]
    BaseSomaAliquotaUnidade = 103,

    #[serde(rename = "Base de Cálculo dos Créditos - Estoque de Abertura (Soma)")]
    BaseSomaEstoqueAbertura = 104,

    #[serde(rename = "Base de Cálculo dos Créditos - Aquisição Embalagens para Revenda (Soma)")]
    BaseSomaAquisicaoEmbalagens = 105,

    #[serde(rename = "Base de Cálculo dos Créditos - Presumido da Agroindústria (Soma)")]
    BaseSomaPresumidoAgroindustria = 106,

    #[serde(rename = "Base de Cálculo dos Créditos - Outros Créditos Presumidos (Soma)")]
    BaseSomaOutrosCreditosPresumidos = 107,

    #[serde(rename = "Base de Cálculo dos Créditos - Importação (Soma)")]
    BaseSomaImportacao = 108,

    #[serde(rename = "Base de Cálculo dos Créditos - Atividade Imobiliária (Soma)")]
    BaseSomaAtividadeImobiliaria = 109,

    #[serde(rename = "Base de Cálculo dos Créditos - Outros (Soma)")]
    BaseSomaOutros = 199,

    #[serde(rename = "Crédito Apurado no Período (PIS/PASEP)")]
    CreditoApuradoPis = 201,

    #[serde(rename = "Crédito Apurado no Período (COFINS)")]
    CreditoApuradoCofins = 205,

    #[serde(rename = "Crédito Disponível após Ajustes (PIS/PASEP)")]
    CreditoAposAjustesPis = 211,

    #[serde(rename = "Crédito Disponível após Ajustes (COFINS)")]
    CreditoAposAjustesCofins = 215,

    #[serde(rename = "Crédito Disponível após Descontos (PIS/PASEP)")]
    CreditoAposDescontosPis = 221,

    #[serde(rename = "Crédito Disponível após Descontos (COFINS)")]
    CreditoAposDescontosCofins = 225,

    #[serde(rename = "Base de Cálculo dos Créditos - Valor Total (Soma)")]
    BaseSomaValorTotal = 300,

    #[serde(rename = "Saldo de Crédito Passível de Desconto ou Ressarcimento (PIS/PASEP)")]
    SaldoDisponivelPis = 301,

    #[serde(rename = "Saldo de Crédito Passível de Desconto ou Ressarcimento (COFINS)")]
    SaldoDisponivelCofins = 305,
}

impl NaturezaBaseCalculo {
    /// Converte u16 para o NaturezaBaseCalculo de forma segura.
    pub const fn from_u16(cod: u16) -> Option<Self> {
        match cod {
            1 => Some(Self::AquisicaoBensRevenda),
            2 => Some(Self::AquisicaoBensInsumo),
            3 => Some(Self::AquisicaoServicosInsumo),
            4 => Some(Self::EnergiaEletricaTermica),
            5 => Some(Self::AlugueisPredios),
            6 => Some(Self::AlugueisMaquinasEquipamentos),
            7 => Some(Self::ArmazenagemFreteVenda),
            8 => Some(Self::ArrendamentoMercantil),
            9 => Some(Self::MaquinasEquipamentosDepreciacao),
            10 => Some(Self::MaquinasEquipamentosAquisicao),
            11 => Some(Self::AmortizacaoDepreciacaoEdificacoes),
            12 => Some(Self::DevolucaoVendasNaoCumulativa),
            13 => Some(Self::OutrasOperacoesComDireitoCredito),
            14 => Some(Self::TransporteCargasSubcontratacao),
            15 => Some(Self::AtividadeImobiliariaCustoIncorrido),
            16 => Some(Self::AtividadeImobiliariaCustoOrcado),
            17 => Some(Self::ServicosLimpezaConservacao),
            18 => Some(Self::EstoqueAberturaBens),
            31 => Some(Self::AjusteAcrescimoPis),
            35 => Some(Self::AjusteAcrescimoCofins),
            41 => Some(Self::AjusteReducaoPis),
            45 => Some(Self::AjusteReducaoCofins),
            51 => Some(Self::DescontoProprioPeriodoPis),
            55 => Some(Self::DescontoProprioPeriodoCofins),
            61 => Some(Self::DescontoPeriodoPosteriorPis),
            65 => Some(Self::DescontoPeriodoPosteriorCofins),
            101 => Some(Self::BaseSomaAliquotaBasica),
            102 => Some(Self::BaseSomaAliquotasDiferenciadas),
            103 => Some(Self::BaseSomaAliquotaUnidade),
            104 => Some(Self::BaseSomaEstoqueAbertura),
            105 => Some(Self::BaseSomaAquisicaoEmbalagens),
            106 => Some(Self::BaseSomaPresumidoAgroindustria),
            107 => Some(Self::BaseSomaOutrosCreditosPresumidos),
            108 => Some(Self::BaseSomaImportacao),
            109 => Some(Self::BaseSomaAtividadeImobiliaria),
            199 => Some(Self::BaseSomaOutros),
            201 => Some(Self::CreditoApuradoPis),
            205 => Some(Self::CreditoApuradoCofins),
            211 => Some(Self::CreditoAposAjustesPis),
            215 => Some(Self::CreditoAposAjustesCofins),
            221 => Some(Self::CreditoAposDescontosPis),
            225 => Some(Self::CreditoAposDescontosCofins),
            300 => Some(Self::BaseSomaValorTotal),
            301 => Some(Self::SaldoDisponivelPis),
            305 => Some(Self::SaldoDisponivelCofins),
            _ => None,
        }
    }

    /// Determina a natureza de ajuste baseada no tipo de operação e tributo
    pub fn from_ajustes(tipo: TipoDeOperacao, tributo: Tributo) -> Option<Self> {
        match (tipo, tributo) {
            (TipoDeOperacao::AjusteAcrescimo, Tributo::Pis) => Some(Self::AjusteAcrescimoPis),
            (TipoDeOperacao::AjusteAcrescimo, Tributo::Cofins) => Some(Self::AjusteAcrescimoCofins),
            (TipoDeOperacao::AjusteReducao, Tributo::Pis) => Some(Self::AjusteReducaoPis),
            (TipoDeOperacao::AjusteReducao, Tributo::Cofins) => Some(Self::AjusteReducaoCofins),
            _ => None,
        }
    }

    /// Determina a natureza de desconto baseada no tipo de operação e tributo
    pub fn from_descontos(tipo: TipoDeOperacao, tributo: Tributo) -> Option<Self> {
        match (tipo, tributo) {
            (TipoDeOperacao::DescontoNoPeriodo, Tributo::Pis) => {
                Some(Self::DescontoProprioPeriodoPis)
            }
            (TipoDeOperacao::DescontoNoPeriodo, Tributo::Cofins) => {
                Some(Self::DescontoProprioPeriodoCofins)
            }
            (TipoDeOperacao::DescontoPosterior, Tributo::Pis) => {
                Some(Self::DescontoPeriodoPosteriorPis)
            }
            (TipoDeOperacao::DescontoPosterior, Tributo::Cofins) => {
                Some(Self::DescontoPeriodoPosteriorCofins)
            }
            _ => None,
        }
    }

    /// Determina a Natureza da BC para Descontos com base no tipo de operação e registro SPED.
    pub fn from_tipo_de_operacao(tipo: TipoDeOperacao, registro: &str) -> Option<Self> {
        match (tipo, registro) {
            // PIS (Bloco M100 / 1100)
            (TipoDeOperacao::DescontoNoPeriodo, "M100" | "1100") => {
                Some(Self::DescontoProprioPeriodoPis)
            }
            (TipoDeOperacao::DescontoPosterior, "M100" | "1100") => {
                Some(Self::DescontoPeriodoPosteriorPis)
            }

            // COFINS (Bloco M500 / 1500)
            (TipoDeOperacao::DescontoNoPeriodo, "M500" | "1500") => {
                Some(Self::DescontoProprioPeriodoCofins)
            }
            (TipoDeOperacao::DescontoPosterior, "M500" | "1500") => {
                Some(Self::DescontoPeriodoPosteriorCofins)
            }

            _ => None,
        }
    }

    /// Retorna o valor numérico (u16)
    pub const fn code(self) -> u16 {
        self as u16
    }

    /// Verifica se o código está entre 01 e 18 (Operações geradoras de crédito)
    pub const fn eh_geradora_de_credito(&self) -> bool {
        // matches! com range é extremamente rápido (compila para uma ou duas instruções assembly)
        matches!(self.code(), 1..=18)
    }

    /// Verifica se o código está entre 101 e 199 (Operações de Soma de Base de Cálculo)
    pub const fn eh_soma_de_bc(&self) -> bool {
        matches!(self.code(), 101..=199)
    }

    /// Retorna a descrição formatada.
    /// Mantém a lógica original: códigos <= 18 usam padding "02", outros usam a string pura.
    pub fn descricao_com_codigo(&self) -> String {
        let c = self.code();
        if c <= 18 {
            format!("{:02} - {}", c, self)
        } else {
            self.to_string()
        }
    }

    /// Crédito Apurado ou Ajuste de PIS
    pub fn eh_ajuste_de_pis(&self) -> bool {
        matches!(
            self,
            Self::CreditoApuradoPis | Self::AjusteAcrescimoPis | Self::AjusteReducaoPis
        )
    }

    /// Crédito Apurado ou Ajuste de COFINS
    pub fn eh_ajuste_de_cofins(&self) -> bool {
        matches!(
            self,
            Self::CreditoApuradoCofins | Self::AjusteAcrescimoCofins | Self::AjusteReducaoCofins
        )
    }

    /// Crédito Após Ajuste e Descontos de PIS
    pub fn eh_desconto_de_pis(&self) -> bool {
        matches!(
            self,
            Self::CreditoAposAjustesPis
                | Self::DescontoProprioPeriodoPis
                | Self::DescontoPeriodoPosteriorPis
        )
    }

    /// Crédito Após Ajuste e Descontos de COFINS
    pub fn eh_desconto_de_cofins(&self) -> bool {
        matches!(
            self,
            Self::CreditoAposAjustesCofins
                | Self::DescontoProprioPeriodoCofins
                | Self::DescontoPeriodoPosteriorCofins
        )
    }
}

impl fmt::Display for NaturezaBaseCalculo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.serialize(f)
    }
}

impl FromStr for NaturezaBaseCalculo {
    type Err = EFDError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 1. Converte e localiza o erro de parsing
        let val = s
            .trim()
            .parse::<u16>()
            .map_loc(|e| EFDError::ParseIntError(e, s.to_string()))?;

        // 2. Converte u16 para o Enum
        Self::from_u16(val)
            .map_loc(|_| EFDError::KeyNotFound(format!("Natureza da BC não encontrada: {val}")))
    }
}

pub trait NatBCOption {
    /// Retorna o valor numérico (u16)
    fn code(&self) -> Option<u16>;

    /// Retorna a descrição formatada.
    fn descricao(&self) -> String;

    /// Verifica se o código está entre 101 e 199 (Operações de Soma de Base de Cálculo)
    fn eh_soma_de_bc(&self) -> bool;
}

impl NatBCOption for Option<NaturezaBaseCalculo> {
    fn code(&self) -> Option<u16> {
        self.map(|c| c as u16)
    }

    fn descricao(&self) -> String {
        self.map(|c| c.descricao_com_codigo()).unwrap_or_default()
    }

    fn eh_soma_de_bc(&self) -> bool {
        self.is_some_and(|n| n.eh_soma_de_bc())
    }
}

// ============================================================================
// Crédito Presumido
// ============================================================================

// Derive(Copy, Clone) permite passar por valor (zero custo de clone).
// Decimal implementa Eq e Hash normalizando a escala (1.00 == 1.0),
// mas o arredondamento prévio na struct garante o "corte" na casa desejada.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct AliquotasKey {
    pis: Decimal,
    cof: Decimal,
}

impl AliquotasKey {
    /// Construtor que normaliza a precisão.
    ///
    /// #[inline] sugere ao compilador otimizar chamadas, útil pois é chamado em loop/filter.
    #[inline]
    fn new(pis: Decimal, cof: Decimal) -> Self {
        Self {
            pis: pis.round_dp(DECIMAL_ALIQ as u32),
            cof: cof.round_dp(DECIMAL_ALIQ as u32),
        }
    }
}

static ALIQUOTAS_DE_CRED_PRESUMIDO: LazyLock<HashSet<AliquotasKey>> = LazyLock::new(|| {
    [
        dec!(0.10), // Lei 12.599, Art. 5o, § 1o  # pis = 0.1650 ; confins = 0.7600 --> crédito presumido - exportação de café, produtos com ncm 0901.1
        dec!(0.12), // 3/25
        dec!(0.20), // Lei 10.925, Art. 8o, § 3o, inciso V.    # pis = 0.3300 ; confins = 1.5200
        dec!(0.35), // Lei 10.925, Art. 8o, § 3o, inciso III.  # pis = 0.5775 ; confins = 2.6600
        dec!(0.50), // Lei 10.925, Art. 8o, § 3o, inciso IV.   # pis = 0.8250 ; confins = 3.8000
        dec!(0.60), // Lei 10.925, Art. 8o, § 3o, inciso I.    # pis = 0.9900 ; confins = 4.5600
        dec!(0.80), // Lei 12.599, Art. 6o, § 2o  # pis = 1.3200 ; confins = 6.0800 --> crédito presumido - industrialização do café, aquisição dos produtos com ncm 0901.1 utilizados na elaboração dos produtos com 0901.2 e 2101.1
        dec!(1.00), // Adição de Alíquota Básica. Alguns Contribuintes usaram esta alíquota como Cred Presumido.
    ]
    .iter() // Itera sobre referências (&Decimal)
    .map(|&p| AliquotasKey::new(p * ALIQ_BASICA_PIS, p * ALIQ_BASICA_COF)) // Transforma
    .collect() // Consome em um HashSet (Zero mutabilidade explícita)
});

/// Verifica se o par de alíquotas corresponde a crédito presumido.
pub fn cred_presumido(aliq_pis: Option<Decimal>, aliq_cof: Option<Decimal>) -> bool {
    aliq_pis
        .zip(aliq_cof) // Combina Option<A> e Option<B> em Option<(A, B)>
        .is_some_and(|(pis, cof)| {
            // Cria a chave "on-the-fly" e verifica no HashSet estático
            let key = AliquotasKey::new(pis, cof);
            ALIQUOTAS_DE_CRED_PRESUMIDO.contains(&key)
        })
}

// ============================================================================
// Código da Situação Tributária (CST)
// ============================================================================

/// 4.3.4 - Tabela Código da Situação Tributária (CST) para PIS/COFINS.
///
/// - CSTs oficiais vão de 1 a 99.
///
/// - CSTs fictícios vão de 490 a 980.
///
/// **Mnemônicos Adotados:**
/// - `Oper` -> Operação | `Trib` -> Tributável | `Aliq` -> Alíquota
/// - `Rec` -> Receita | `Cred` -> Crédito | `MI` -> Mercado Interno
/// - `Exp` -> Exportação | `Aq` -> Aquisição | `ST` -> Subst. Tributária
#[repr(u16)]
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum CodigoSituacaoTributaria {
    // --- Saídas (01-49) ---
    /// Operação Tributável com Alíquota Básica
    #[serde(rename = "Operação Tributável com Alíquota Básica")]
    OperTribAliqBasica = 1,

    /// Operação Tributável com Alíquota Diferenciada
    #[serde(rename = "Operação Tributável com Alíquota Diferenciada")]
    OperTribAliqDif = 2,

    /// Operação Tributável com Alíquota por Unidade de Medida de Produto
    #[serde(rename = "Operação Tributável com Alíquota por Unidade de Medida de Produto")]
    OperTribAliqUnidade = 3,

    /// Operação Tributável Monofásica - Revenda a Alíquota Zero
    #[serde(rename = "Operação Tributável Monofásica - Revenda a Alíquota Zero")]
    OperTribMonofasicaRevendaAliqZero = 4,

    /// Operação Tributável por Substituição Tributária
    #[serde(rename = "Operação Tributável por Substituição Tributária")]
    OperTribST = 5,

    /// Operação Tributável a Alíquota Zero
    #[serde(rename = "Operação Tributável a Alíquota Zero")]
    OperTribAliqZero = 6,

    /// Operação Isenta da Contribuição
    #[serde(rename = "Operação Isenta da Contribuição")]
    OperIsenta = 7,

    /// Operação sem Incidência da Contribuição
    #[serde(rename = "Operação sem Incidência da Contribuição")]
    OperSemIncidencia = 8,

    /// Operação com Suspensão da Contribuição
    #[serde(rename = "Operação com Suspensão da Contribuição")]
    OperSuspensao = 9,

    /// Outras Operações de Saída
    #[serde(rename = "Outras Operações de Saída")]
    OutrasOperSaida = 49,

    // --- Entradas com Direito a Crédito (50-56) ---
    /// Operação com Direito a Crédito - Vinculada Exclusivamente a Receita Tributada no Mercado Interno
    #[serde(
        rename = "Operação com Direito a Crédito - Vinculada Exclusivamente a Receita Tributada no Mercado Interno"
    )]
    CredVincExclRecTribMI = 50,

    /// Operação com Direito a Crédito - Vinculada Exclusivamente a Receita Não-Tributada no Mercado Interno
    #[serde(
        rename = "Operação com Direito a Crédito - Vinculada Exclusivamente a Receita Não-Tributada no Mercado Interno"
    )]
    CredVincExclRecNTribMI = 51,

    /// Operação com Direito a Crédito - Vinculada Exclusivamente a Receita de Exportação
    #[serde(
        rename = "Operação com Direito a Crédito - Vinculada Exclusivamente a Receita de Exportação"
    )]
    CredVincExclRecExp = 52,

    /// Operação com Direito a Crédito - Vinculada a Receitas Tributadas e Não-Tributadas no Mercado Interno
    #[serde(
        rename = "Operação com Direito a Crédito - Vinculada a Receitas Tributadas e Não-Tributadas no Mercado Interno"
    )]
    CredVincRecTribENTribMI = 53,

    /// Operação com Direito a Crédito - Vinculada a Receitas Tributadas no Mercado Interno e de Exportação
    #[serde(
        rename = "Operação com Direito a Crédito - Vinculada a Receitas Tributadas no Mercado Interno e de Exportação"
    )]
    CredVincRecTribMIExp = 54,

    /// Operação com Direito a Crédito - Vinculada a Receitas Não Tributadas no Mercado Interno e de Exportação
    #[serde(
        rename = "Operação com Direito a Crédito - Vinculada a Receitas Não Tributadas no Mercado Interno e de Exportação"
    )]
    CredVincRecNTribMIExp = 55,

    /// Operação com Direito a Crédito - Vinculada a Receitas Tributadas e Não-Tributadas no Mercado Interno e de Exportação
    #[serde(
        rename = "Operação com Direito a Crédito - Vinculada a Receitas Tributadas e Não-Tributadas no Mercado Interno e de Exportação"
    )]
    CredVincRecTribENTribMIExp = 56,

    // --- Crédito Presumido (60-67) ---
    /// Crédito Presumido - Operação de Aquisição Vinculada Exclusivamente a Receita Tributada no Mercado Interno
    #[serde(
        rename = "Crédito Presumido - Operação de Aquisição Vinculada Exclusivamente a Receita Tributada no Mercado Interno"
    )]
    CredPresAqExclRecTribMI = 60,

    /// Crédito Presumido - Operação de Aquisição Vinculada Exclusivamente a Receita Não-Tributada no Mercado Interno
    #[serde(
        rename = "Crédito Presumido - Operação de Aquisição Vinculada Exclusivamente a Receita Não-Tributada no Mercado Interno"
    )]
    CredPresAqExclRecNTribMI = 61,

    /// Crédito Presumido - Operação de Aquisição Vinculada Exclusivamente a Receita de Exportação
    #[serde(
        rename = "Crédito Presumido - Operação de Aquisição Vinculada Exclusivamente a Receita de Exportação"
    )]
    CredPresAqExclRecExp = 62,

    /// Crédito Presumido - Operação de Aquisição Vinculada a Receitas Tributadas e Não-Tributadas no Mercado Interno
    #[serde(
        rename = "Crédito Presumido - Operação de Aquisição Vinculada a Receitas Tributadas e Não-Tributadas no Mercado Interno"
    )]
    CredPresAqRecTribENTribMI = 63,

    /// Crédito Presumido - Operação de Aquisição Vinculada a Receitas Tributadas no Mercado Interno e de Exportação
    #[serde(
        rename = "Crédito Presumido - Operação de Aquisição Vinculada a Receitas Tributadas no Mercado Interno e de Exportação"
    )]
    CredPresAqRecTribMIExp = 64,

    /// Crédito Presumido - Operação de Aquisição Vinculada a Receitas Não-Tributadas no Mercado Interno e de Exportação
    #[serde(
        rename = "Crédito Presumido - Operação de Aquisição Vinculada a Receitas Não-Tributadas no Mercado Interno e de Exportação"
    )]
    CredPresAqRecNTribMIExp = 65,

    /// Crédito Presumido - Operação de Aquisição Vinculada a Receitas Tributadas e Não-Tributadas no Mercado Interno e de Exportação
    #[serde(
        rename = "Crédito Presumido - Operação de Aquisição Vinculada a Receitas Tributadas e Não-Tributadas no Mercado Interno e de Exportação"
    )]
    CredPresAqRecTribENTribMIExp = 66,

    /// Crédito Presumido - Outras Operações
    #[serde(rename = "Crédito Presumido - Outras Operações")]
    CredPresOutrasOper = 67,

    // --- Entradas sem Direito a Crédito (70-75) ---
    /// Operação de Aquisição sem Direito a Crédito
    #[serde(rename = "Operação de Aquisição sem Direito a Crédito")]
    AqSemCred = 70,

    /// Operação de Aquisição com Isenção
    #[serde(rename = "Operação de Aquisição com Isenção")]
    AqIsencao = 71,

    /// Operação de Aquisição com Suspensão
    #[serde(rename = "Operação de Aquisição com Suspensão")]
    AqSuspensao = 72,

    /// Operação de Aquisição a Alíquota Zero
    #[serde(rename = "Operação de Aquisição a Alíquota Zero")]
    AqAliqZero = 73,

    /// Operação de Aquisição sem Incidência da Contribuição
    #[serde(rename = "Operação de Aquisição sem Incidência da Contribuição")]
    AqSemIncidencia = 74,

    /// Operação de Aquisição por Substituição Tributária
    #[serde(rename = "Operação de Aquisição por Substituição Tributária")]
    AqSubstiTrib = 75,

    // --- Outros (98-99) ---
    /// Outras Operações de Entrada
    #[serde(rename = "Outras Operações de Entrada")]
    OutrasOperEntrada = 98,

    /// Outras Operações
    #[serde(rename = "Outras Operações")]
    OutrasOper = 99,

    // --- CSTs Fictícios (490-980) ---
    /// Total Receitas/Saídas
    #[serde(rename = "Total Receitas/Saídas")]
    TotalReceitasSaidas = 490,

    /// Agrupador da Base de Cálculo
    #[serde(rename = "Agrupador da Base de Cálculo")]
    AgrupadorSomaBC = 900,

    /// Soma Parcial da Base de Cálculo
    #[serde(rename = "Soma Parcial da Base de Cálculo")]
    SomaParcialDaBaseCalculo = 910,

    /// Crédito Apurado no Período (PIS/PASEP)
    #[serde(rename = "Crédito Apurado no Período (PIS/PASEP)")]
    CSTApuradoPIS = 920,

    /// Crédito Apurado no Período (COFINS)
    #[serde(rename = "Crédito Apurado no Período (COFINS)")]
    CSTApuradoCofins = 930,

    /// Agrupador de Saldo Disponível
    #[serde(rename = "Agrupador de Saldo Disponível")]
    AgrupadorSaldoDisp = 950,

    /// Total Aquisições/Custos/Despesas
    #[serde(rename = "Total Aquisições/Custos/Despesas")]
    TotalAquisicoes = 980,
}

impl CodigoSituacaoTributaria {
    /// Tenta converter um `u16` para `CodigoSituacaoTributaria`.
    ///
    /// Retorna `Some(CodigoSituacaoTributaria)` se o valor for válido,
    /// caso contrário, retorna `None`.
    pub const fn from_u16(cod: u16) -> Option<Self> {
        match cod {
            // Saídas (01-49)
            1 => Some(Self::OperTribAliqBasica),
            2 => Some(Self::OperTribAliqDif),
            3 => Some(Self::OperTribAliqUnidade),
            4 => Some(Self::OperTribMonofasicaRevendaAliqZero),
            5 => Some(Self::OperTribST),
            6 => Some(Self::OperTribAliqZero),
            7 => Some(Self::OperIsenta),
            8 => Some(Self::OperSemIncidencia),
            9 => Some(Self::OperSuspensao),
            49 => Some(Self::OutrasOperSaida),

            // Entradas com Crédito (50-56)
            50 => Some(Self::CredVincExclRecTribMI),
            51 => Some(Self::CredVincExclRecNTribMI),
            52 => Some(Self::CredVincExclRecExp),
            53 => Some(Self::CredVincRecTribENTribMI),
            54 => Some(Self::CredVincRecTribMIExp),
            55 => Some(Self::CredVincRecNTribMIExp),
            56 => Some(Self::CredVincRecTribENTribMIExp),

            // Crédito Presumido (60-67)
            60 => Some(Self::CredPresAqExclRecTribMI),
            61 => Some(Self::CredPresAqExclRecNTribMI),
            62 => Some(Self::CredPresAqExclRecExp),
            63 => Some(Self::CredPresAqRecTribENTribMI),
            64 => Some(Self::CredPresAqRecTribMIExp),
            65 => Some(Self::CredPresAqRecNTribMIExp),
            66 => Some(Self::CredPresAqRecTribENTribMIExp),
            67 => Some(Self::CredPresOutrasOper),

            // Entradas sem Crédito (70-75)
            70 => Some(Self::AqSemCred),
            71 => Some(Self::AqIsencao),
            72 => Some(Self::AqSuspensao),
            73 => Some(Self::AqAliqZero),
            74 => Some(Self::AqSemIncidencia),
            75 => Some(Self::AqSubstiTrib),
            98 => Some(Self::OutrasOperEntrada),
            99 => Some(Self::OutrasOper),

            // Fictícios
            490 => Some(Self::TotalReceitasSaidas),
            900 => Some(Self::AgrupadorSomaBC),
            910 => Some(Self::SomaParcialDaBaseCalculo),
            920 => Some(Self::CSTApuradoPIS),
            930 => Some(Self::CSTApuradoCofins),
            950 => Some(Self::AgrupadorSaldoDisp),
            980 => Some(Self::TotalAquisicoes),
            _ => None,
        }
    }

    /// Retorna o valor numérico do CST (ex: 1, 50, 99).
    pub const fn code(self) -> u16 {
        self as u16
    }

    /// Indica se o CST é uma construção interna para fins de relatório
    pub fn eh_ficticio(&self) -> bool {
        self.code() >= 490
    }

    /// Indica se o CST deve ser ocultado/limpo após a ordenação
    /// (Ex: Agrupadores técnicos vs Totais que talvez devam aparecer)
    pub fn deve_limpar_cst(&self) -> bool {
        self.code() >= 900
    }

    /// Define a ordem de apresentação da Consolidação de CST.
    pub fn get_ordem(self) -> u16 {
        match self.code() {
            1..=49 => 1,  // Saídas:   1  <= cst <= 49
            490 => 2,     // TotalReceitasSaidas
            50..=98 => 3, // Entradas: 50 <= cst <= 98 (Créditos e Aquisições)
            980 => 4,     // TotalAquisicoes
            99 => 5,      // Outras
            _ => 0,       // Fallback
        }
    }

    /// Verifica se o CST gera direito a crédito.
    ///
    /// Faixas consideradas:
    /// - 50 a 56: Operações com Direito a Crédito
    /// - 60 a 66: Crédito Presumido
    pub const fn eh_base_de_credito(&self) -> bool {
        // O cast (*self as u16) é seguro e sem custo (zero-cost) devido ao #[repr(u16)]
        matches!(self.code(), 50..=56 | 60..=66)
    }

    /// Verifica se é um CST de Receita Bruta (excluindo ou incluindo o 49 conforme flag).
    pub fn eh_receita_bruta(self, excluir_49: bool) -> bool {
        let codigo = self.code();
        if excluir_49 && codigo == 49 {
            return false;
        }
        matches!(codigo, 1..=9 | 49)
    }

    /// Retorna a descrição formatada com o código (ex: "01 - Operação Tributável com Alíquota Básica")
    pub fn descricao_com_codigo(&self) -> String {
        // *self as u16 obtém o valor numérico (1, 50, 99...)
        // self (Display) obtém a descrição do #[serde(rename)]
        format!("{:02} - {}", self.code(), self)
    }
}

pub trait CSTOption {
    fn descricao(&self) -> String;
    fn code(&self) -> Option<u16>;
    fn eh_base_de_credito(&self) -> bool;
    fn eh_receita_bruta(&self, excluir_49: bool) -> bool;

    /// Deduz o Tipo de Operação baseado no CST.
    /// Remove os "Magic Numbers" do método build.
    fn obter_tipo_operacao(&self) -> Option<TipoDeOperacao>;
}

impl CSTOption for Option<CodigoSituacaoTributaria> {
    fn descricao(&self) -> String {
        self.map(|c| c.descricao_com_codigo()).unwrap_or_default()
    }

    fn code(&self) -> Option<u16> {
        self.map(|c| c as u16)
    }

    fn eh_base_de_credito(&self) -> bool {
        self.is_some_and(|c| c.eh_base_de_credito())
    }

    fn eh_receita_bruta(&self, excluir_49: bool) -> bool {
        self.is_some_and(|c| c.eh_receita_bruta(excluir_49))
    }

    fn obter_tipo_operacao(&self) -> Option<TipoDeOperacao> {
        match self.code() {
            Some(1..=49) => Some(TipoDeOperacao::Saida),
            Some(50..=99) => Some(TipoDeOperacao::Entrada),
            _ => None,
        }
    }
}

// Implementação do Display usando o Serializer do Serde para exibir a descrição
impl fmt::Display for CodigoSituacaoTributaria {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Isso utiliza o texto definido em #[serde(rename = "...")]
        self.serialize(f)
    }
}

// ============================================================================
// CFOP: Descrição Resumida
// ============================================================================

static CFOP_DESCRICAO_RESUMIDA: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    // array of tuples [T; length]
    // Tabela Código CFOP: Descrição Resumida
    // https://www.confaz.fazenda.gov.br/legislacao/ajustes/sinief/cfop_cvsn_70_vigente
    // ^(\d{4}) - (.*)$       |        $cfop_descricao_resumida{'$1'} = '$2';
    [
            (1000, "ENTRADAS OU AQUISIÇÕES DE SERVIÇOS DO ESTADO"),
            (
                1100,
                "COMPRAS PARA INDUSTRIALIZAÇÃO, PRODUÇÃO RURAL, COMERCIALIZAÇÃO OU PRESTAÇÃO DE SERVIÇOS",
            ),
            (1101, "Compra para industrialização ou produção rural"),
            (1102, "Compra para comercialização"),
            (
                1111,
                "Compra para industrialização de mercadoria recebida anteriormente em consignação industrial",
            ),
            (
                1113,
                "Compra para comercialização, de mercadoria recebida anteriormente em consignação mercantil",
            ),
            (
                1116,
                "Compra para industrialização ou produção rural originada de encomenda para recebimento futuro",
            ),
            (
                1117,
                "Compra para comercialização originada de encomenda para recebimento futuro",
            ),
            (
                1118,
                "Compra de mercadoria para comercialização pelo adquirente originário, entregue pelo vendedor remetente ao destinatário, em venda à ordem",
            ),
            (
                1120,
                "Compra para industrialização, em venda à ordem, já recebida do vendedor remetente",
            ),
            (
                1121,
                "Compra para comercialização, em venda à ordem, já recebida do vendedor remetente",
            ),
            (
                1122,
                "Compra para industrialização em que a mercadoria foi remetida pelo fornecedor ao industrializador sem transitar pelo estabelecimento adquirente",
            ),
            (1124, "Industrialização efetuada por outra empresa"),
            (
                1125,
                "Industrialização efetuada por outra empresa quando a mercadoria remetida para utilização no processo de industrialização não transitou pelo estabelecimento adquirente da mercadoria",
            ),
            (
                1126,
                "Compra para utilização na prestação de serviço sujeita ao ICMS",
            ),
            (
                1128,
                "Compra para utilização na prestação de serviço sujeita ao ISSQN",
            ),
            (
                1131,
                "Entrada de mercadoria com previsão de posterior ajuste ou fixação de preço, decorrente de operação de ato cooperativo",
            ),
            (
                1132,
                "Fixação de preço de produção do estabelecimento produtor, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço, em ato cooperativo, para comercialização",
            ),
            (
                1135,
                "Fixação de preço de produção do estabelecimento produtor, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço, em ato cooperativo, para industrialização",
            ),
            (
                1150,
                "TRANSFERÊNCIAS PARA INDUSTRIALIZAÇÃO, PRODUÇÃO RURAL, COMERCIALIZAÇÃO OU PRESTAÇÃO DE SERVIÇOS",
            ),
            (
                1151,
                "Transferência para industrialização ou produção rural",
            ),
            (1152, "Transferência para comercialização"),
            (1153, "Transferência de energia elétrica para distribuição"),
            (
                1154,
                "Transferência para utilização na prestação de serviço",
            ),
            (
                1159,
                "Entrada decorrente do fornecimento de produto ou mercadoria de ato cooperativo",
            ),
            (
                1200,
                "DEVOLUÇÕES DE VENDAS DE PRODUÇÃO PRÓPRIA, DE TERCEIROS OU ANULAÇÕES DE VALORES",
            ),
            (1201, "Devolução de venda de produção do estabelecimento"),
            (
                1202,
                "Devolução de venda de mercadoria adquirida ou recebida de terceiros",
            ),
            (
                1203,
                "Devolução de venda de produção do estabelecimento, destinada à Zona Franca de Manaus ou Áreas de Livre Comércio",
            ),
            (
                1204,
                "Devolução de venda de mercadoria adquirida ou recebida de terceiros, destinada à Zona Franca de Manaus ou Áreas de Livre Comércio",
            ),
            (
                1205,
                "Anulação de valor relativo à prestação de serviço de comunicação",
            ),
            (
                1206,
                "Anulação de valor relativo à prestação de serviço de transporte",
            ),
            (
                1207,
                "Anulação de valor relativo à venda de energia elétrica",
            ),
            (
                1208,
                "Devolução de produção do estabelecimento, remetida em transferência",
            ),
            (
                1209,
                "Devolução de mercadoria adquirida ou recebida de terceiros, remetida em transferência",
            ),
            (
                1212,
                "Devolução de venda no mercado interno de mercadoria industrializada e insumo importado sob o Regime Aduaneiro Especial de Entreposto Industrial sob Controle Informatizado do Sistema Público de Escrituração Digital (Recof-Sped)",
            ),
            (
                1213,
                "Devolução de remessa de produção do estabelecimento com previsão de posterior ajuste ou fixação de preço, em ato cooperativo",
            ),
            (
                1214,
                "Devolução de fixação de preço de produção do estabelecimento produtor, de ato cooperativo",
            ),
            (1250, "COMPRAS DE ENERGIA ELÉTRICA"),
            (
                1251,
                "Compra de energia elétrica para distribuição ou comercialização",
            ),
            (
                1252,
                "Compra de energia elétrica por estabelecimento industrial",
            ),
            (
                1253,
                "Compra de energia elétrica por estabelecimento comercial",
            ),
            (
                1254,
                "Compra de energia elétrica por estabelecimento prestador de serviço de transporte",
            ),
            (
                1255,
                "Compra de energia elétrica por estabelecimento prestador de serviço de comunicação",
            ),
            (
                1256,
                "Compra de energia elétrica por estabelecimento de produtor rural",
            ),
            (
                1257,
                "Compra de energia elétrica para consumo por demanda contratada",
            ),
            (1300, "AQUISIÇÕES DE SERVIÇOS DE COMUNICAÇÃO"),
            (
                1301,
                "Aquisição de serviço de comunicação para execução de serviço da mesma natureza",
            ),
            (
                1302,
                "Aquisição de serviço de comunicação por estabelecimento industrial",
            ),
            (
                1303,
                "Aquisição de serviço de comunicação por estabelecimento comercial",
            ),
            (
                1304,
                "Aquisição de serviço de comunicação por estabelecimento de prestador de serviço de transporte",
            ),
            (
                1305,
                "Aquisição de serviço de comunicação por estabelecimento de geradora ou de distribuidora de energia elétrica",
            ),
            (
                1306,
                "Aquisição de serviço de comunicação por estabelecimento de produtor rural",
            ),
            (1350, "AQUISIÇÕES DE SERVIÇOS DE TRANSPORTE"),
            (
                1351,
                "Aquisição de serviço de transporte para execução de serviço da mesma natureza",
            ),
            (
                1352,
                "Aquisição de serviço de transporte por estabelecimento industrial",
            ),
            (
                1353,
                "Aquisição de serviço de transporte por estabelecimento comercial",
            ),
            (
                1354,
                "Aquisição de serviço de transporte por estabelecimento de prestador de serviço de comunicação",
            ),
            (
                1355,
                "Aquisição de serviço de transporte por estabelecimento de geradora ou de distribuidora de energia elétrica",
            ),
            (
                1356,
                "Aquisição de serviço de transporte por estabelecimento de produtor rural",
            ),
            (
                1360,
                "Aquisição de serviço de transporte por contribuinte substituto em relação ao serviço de transporte",
            ),
            (
                1400,
                "ENTRADAS DE MERCADORIAS SUJEITAS AO REGIME DE SUBSTITUIÇÃO TRIBUTÁRIA",
            ),
            (
                1401,
                "Compra para industrialização ou produção rural em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                1403,
                "Compra para comercialização em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                1406,
                "Compra de bem para o ativo imobilizado cuja mercadoria está sujeita ao regime de substituição tributária",
            ),
            (
                1407,
                "Compra de mercadoria para uso ou consumo cuja mercadoria está sujeita ao regime de substituição tributária",
            ),
            (
                1408,
                "Transferência para industrialização ou produção rural em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                1409,
                "Transferência para comercialização em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                1410,
                "Devolução de venda de produção do estabelecimento em operação com produto sujeito ao regime de substituição tributária",
            ),
            (
                1411,
                "Devolução de venda de mercadoria adquirida ou recebida de terceiros em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                1414,
                "Retorno de produção do estabelecimento, remetida para venda fora do estabelecimento em operação com produto sujeito ao regime de substituição tributária",
            ),
            (
                1415,
                "Retorno de mercadoria adquirida ou recebida de terceiros, remetida para venda fora do estabelecimento em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (1450, "SISTEMAS DE INTEGRAÇÃO"),
            (1451, "Retorno de animal do estabelecimento produtor"),
            (1452, "Retorno de insumo não utilizado na produção"),
            (
                1500,
                "ENTRADAS DE MERCADORIAS REMETIDAS PARA FORMAÇÃO DE LOTE OU COM FIM ESPECÍFICO DE EXPORTAÇÃO E EVENTUAIS DEVOLUÇÕES",
            ),
            (
                1501,
                "Entrada de mercadoria recebida com fim específico de exportação",
            ),
            (
                1503,
                "Entrada decorrente de devolução de produto remetido com fim específico de exportação, de produção do estabelecimento",
            ),
            (
                1504,
                "Entrada decorrente de devolução de mercadoria remetida com fim específico de exportação, adquirida ou recebida de terceiros",
            ),
            (
                1505,
                "Entrada decorrente de devolução de mercadorias remetidas para formação de lote de exportação, de produtos industrializados ou produzidos pelo próprio estabelecimento",
            ),
            (
                1506,
                "Entrada decorrente de devolução de mercadorias, adquiridas ou recebidas de terceiros, remetidas para formação de lote de exportação",
            ),
            (
                1550,
                "OPERAÇÕES COM BENS DE ATIVO IMOBILIZADO E MATERIAIS PARA USO OU CONSUMO",
            ),
            (1551, "Compra de bem para o ativo imobilizado"),
            (1552, "Transferência de bem do ativo imobilizado"),
            (1553, "Devolução de venda de bem do ativo imobilizado"),
            (
                1554,
                "Retorno de bem do ativo imobilizado remetido para uso fora do estabelecimento",
            ),
            (
                1555,
                "Entrada de bem do ativo imobilizado de terceiro, remetido para uso no estabelecimento",
            ),
            (1556, "Compra de material para uso ou consumo"),
            (1557, "Transferência de material para uso ou consumo"),
            (1600, "CRÉDITOS E RESSARCIMENTOS DE ICMS"),
            (1601, "Recebimento, por transferência, de crédito de ICMS"),
            (
                1602,
                "Recebimento, por transferência, de saldo credor de ICMS de outro estabelecimento da mesma empresa, para compensação de saldo devedor de ICMS",
            ),
            (
                1603,
                "Ressarcimento de ICMS retido por substituição tributária",
            ),
            (
                1604,
                "Lançamento do crédito relativo à compra de bem para o ativo imobilizado",
            ),
            (
                1605,
                "Recebimento, por transferência, de saldo devedor de ICMS de outro estabelecimento da mesma empresa",
            ),
            (
                1650,
                "ENTRADAS DE COMBUSTÍVEIS, DERIVADOS OU NÃO DE PETRÓLEO E LUBRIFICANTES",
            ),
            (
                1651,
                "Compra de combustível ou lubrificante para industrialização subseqüente",
            ),
            (
                1652,
                "Compra de combustível ou lubrificante para comercialização",
            ),
            (
                1653,
                "Compra de combustível ou lubrificante por consumidor ou usuário final",
            ),
            (
                1658,
                "Transferência de combustível e lubrificante para industrialização",
            ),
            (
                1659,
                "Transferência de combustível e lubrificante para comercialização",
            ),
            (
                1660,
                "Devolução de venda de combustível ou lubrificante destinado à industrialização subseqüente",
            ),
            (
                1661,
                "Devolução de venda de combustível ou lubrificante destinado à comercialização",
            ),
            (
                1662,
                "Devolução de venda de combustível ou lubrificante destinado a consumidor ou usuário final",
            ),
            (
                1663,
                "Entrada de combustível ou lubrificante para armazenagem",
            ),
            (
                1664,
                "Retorno de combustível ou lubrificante remetido para armazenagem",
            ),
            (
                1900,
                "OUTRAS ENTRADAS DE MERCADORIAS OU AQUISIÇÕES DE SERVIÇOS",
            ),
            (1901, "Entrada para industrialização por encomenda"),
            (
                1902,
                "Retorno de mercadoria remetida para industrialização por encomenda",
            ),
            (
                1903,
                "Entrada de mercadoria remetida para industrialização e não aplicada no referido processo",
            ),
            (
                1904,
                "Retorno de remessa para venda fora do estabelecimento",
            ),
            (
                1905,
                "Entrada de mercadoria recebida para depósito em depósito fechado ou armazém geral",
            ),
            (
                1906,
                "Retorno de mercadoria remetida para depósito fechado ou armazém geral",
            ),
            (
                1907,
                "Retorno simbólico de mercadoria remetida para depósito fechado ou armazém geral",
            ),
            (1908, "Entrada de bem por conta de contrato de comodato"),
            (
                1909,
                "Retorno de bem remetido por conta de contrato de comodato",
            ),
            (1910, "Entrada de bonificação, doação ou brinde"),
            (1911, "Entrada de amostra grátis"),
            (
                1912,
                "Entrada de mercadoria ou bem recebido para demonstração ou mostruário",
            ),
            (
                1913,
                "Retorno de mercadoria ou bem remetido para demonstração, mostruário ou treinamento",
            ),
            (
                1914,
                "Retorno de mercadoria ou bem remetido para exposição ou feira",
            ),
            (
                1915,
                "Entrada de mercadoria ou bem recebido para conserto ou reparo",
            ),
            (
                1916,
                "Retorno de mercadoria ou bem remetido para conserto ou reparo",
            ),
            (
                1917,
                "Entrada de mercadoria recebida em consignação mercantil ou industrial",
            ),
            (
                1918,
                "Devolução de mercadoria remetida em consignação mercantil ou industrial",
            ),
            (
                1919,
                "Devolução simbólica de mercadoria vendida ou utilizada em processo industrial, remetida anteriormente em consignação mercantil ou industrial",
            ),
            (1920, "Entrada de vasilhame ou sacaria"),
            (1921, "Retorno de vasilhame ou sacaria"),
            (
                1922,
                "Lançamento efetuado a título de simples faturamento decorrente de compra para recebimento futuro",
            ),
            (
                1923,
                "Entrada de mercadoria recebida do vendedor remetente, em venda à ordem",
            ),
            (
                1924,
                "Entrada para industrialização por conta e ordem do adquirente da mercadoria, quando esta não transitar pelo estabelecimento do adquirente",
            ),
            (
                1925,
                "Retorno de mercadoria remetida para industrialização por conta e ordem do adquirente da mercadoria, quando esta não transitar pelo estabelecimento do adquirente",
            ),
            (
                1926,
                "Lançamento efetuado a título de reclassificação de mercadoria decorrente de formação de kit ou de sua desagregação",
            ),
            (
                1931,
                "Lançamento efetuado pelo tomador do serviço de transporte quando a responsabilidade de retenção do imposto for atribuída ao remetente ou alienante da mercadoria, pelo serviço de transporte realizado por transportador autônomo ou por transportador não inscrito na unidade da Federação onde iniciado o serviço",
            ),
            (
                1932,
                "Aquisição de serviço de transporte iniciado em unidade da Federação diversa daquela onde inscrito o prestador",
            ),
            (1933, "Aquisição de serviço tributado pelo ISSQN"),
            (
                1934,
                "Entrada simbólica de mercadoria recebida para depósito fechado ou armazém geral",
            ),
            (
                1949,
                "Outra entrada de mercadoria ou prestação de serviço não especificada",
            ),
            (2000, "ENTRADAS OU AQUISIÇÕES DE SERVIÇOS DE OUTROS ESTADOS"),
            (
                2100,
                "COMPRAS PARA INDUSTRIALIZAÇÃO, PRODUÇÃO RURAL, COMERCIALIZAÇÃO OU PRESTAÇÃO DE SERVIÇOS",
            ),
            (2101, "Compra para industrialização ou produção rural"),
            (2102, "Compra para comercialização"),
            (
                2111,
                "Compra para industrialização de mercadoria recebida anteriormente em consignação industrial",
            ),
            (
                2113,
                "Compra para comercialização, de mercadoria recebida anteriormente em consignação mercantil",
            ),
            (
                2116,
                "Compra para industrialização ou produção rural originada de encomenda para recebimento futuro",
            ),
            (
                2117,
                "Compra para comercialização originada de encomenda para recebimento futuro",
            ),
            (
                2118,
                "Compra de mercadoria para comercialização pelo adquirente originário, entregue pelo vendedor remetente ao destinatário, em venda à ordem",
            ),
            (
                2120,
                "Compra para industrialização, em venda à ordem, já recebida do vendedor remetente",
            ),
            (
                2121,
                "Compra para comercialização, em venda à ordem, já recebida do vendedor remetente",
            ),
            (
                2122,
                "Compra para industrialização em que a mercadoria foi remetida pelo fornecedor ao industrializador sem transitar pelo estabelecimento adquirente",
            ),
            (2124, "Industrialização efetuada por outra empresa"),
            (
                2125,
                "Industrialização efetuada por outra empresa quando a mercadoria remetida para utilização no processo de industrialização não transitou pelo estabelecimento adquirente da mercadoria",
            ),
            (
                2126,
                "Compra para utilização na prestação de serviço sujeita ao ICMS",
            ),
            (
                2128,
                "Compra para utilização na prestação de serviço sujeita ao ISSQN",
            ),
            (
                2131,
                "Entrada de mercadoria com previsão de posterior ajuste ou fixação de preço, decorrente de operação de ato cooperativo",
            ),
            (
                2132,
                "Fixação de preço de produção do estabelecimento produtor, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço, em ato cooperativo, para comercialização",
            ),
            (
                2135,
                "Fixação de preço de produção do estabelecimento produtor, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço, em ato cooperativo, para industrialização",
            ),
            (
                2150,
                "TRANSFERÊNCIAS PARA INDUSTRIALIZAÇÃO, PRODUÇÃO RURAL, COMERCIALIZAÇÃO OU PRESTAÇÃO DE SERVIÇOS",
            ),
            (
                2151,
                "Transferência para industrialização ou produção rural",
            ),
            (2152, "Transferência para comercialização"),
            (2153, "Transferência de energia elétrica para distribuição"),
            (
                2154,
                "Transferência para utilização na prestação de serviço",
            ),
            (
                2159,
                "Entrada decorrente do fornecimento de produto ou mercadoria de ato cooperativo",
            ),
            (
                2200,
                "DEVOLUÇÕES DE VENDAS DE PRODUÇÃO PRÓPRIA, DE TERCEIROS OU ANULAÇÕES DE VALORES",
            ),
            (2201, "Devolução de venda de produção do estabelecimento"),
            (
                2202,
                "Devolução de venda de mercadoria adquirida ou recebida de terceiros",
            ),
            (
                2203,
                "Devolução de venda de produção do estabelecimento, destinada à Zona Franca de Manaus ou Áreas de Livre Comércio",
            ),
            (
                2204,
                "Devolução de venda de mercadoria adquirida ou recebida de terceiros, destinada à Zona Franca de Manaus ou Áreas de Livre Comércio",
            ),
            (
                2205,
                "Anulação de valor relativo à prestação de serviço de comunicação",
            ),
            (
                2206,
                "Anulação de valor relativo à prestação de serviço de transporte",
            ),
            (
                2207,
                "Anulação de valor relativo à venda de energia elétrica",
            ),
            (
                2208,
                "Devolução de produção do estabelecimento, remetida em transferência",
            ),
            (
                2209,
                "Devolução de mercadoria adquirida ou recebida de terceiros, remetida em transferência",
            ),
            (
                2212,
                "Devolução de venda no mercado interno de mercadoria industrializada e insumo importado sob o Regime Aduaneiro Especial de Entreposto Industrial sob Controle Informatizado do Sistema Público de Escrituração Digital (Recof-Sped)",
            ),
            (
                2213,
                "Devolução de remessa de produção do estabelecimento com previsão de posterior ajuste ou fixação de preço, em ato cooperativo",
            ),
            (
                2214,
                "Devolução de fixação de preço de produção do estabelecimento produtor, de ato cooperativo",
            ),
            (2250, "COMPRAS DE ENERGIA ELÉTRICA"),
            (
                2251,
                "Compra de energia elétrica para distribuição ou comercialização",
            ),
            (
                2252,
                "Compra de energia elétrica por estabelecimento industrial",
            ),
            (
                2253,
                "Compra de energia elétrica por estabelecimento comercial",
            ),
            (
                2254,
                "Compra de energia elétrica por estabelecimento prestador de serviço de transporte",
            ),
            (
                2255,
                "Compra de energia elétrica por estabelecimento prestador de serviço de comunicação",
            ),
            (
                2256,
                "Compra de energia elétrica por estabelecimento de produtor rural",
            ),
            (
                2257,
                "Compra de energia elétrica para consumo por demanda contratada",
            ),
            (2300, "AQUISIÇÕES DE SERVIÇOS DE COMUNICAÇÃO"),
            (
                2301,
                "Aquisição de serviço de comunicação para execução de serviço da mesma natureza",
            ),
            (
                2302,
                "Aquisição de serviço de comunicação por estabelecimento industrial",
            ),
            (
                2303,
                "Aquisição de serviço de comunicação por estabelecimento comercial",
            ),
            (
                2304,
                "Aquisição de serviço de comunicação por estabelecimento de prestador de serviço de transporte",
            ),
            (
                2305,
                "Aquisição de serviço de comunicação por estabelecimento de geradora ou de distribuidora de energia elétrica",
            ),
            (
                2306,
                "Aquisição de serviço de comunicação por estabelecimento de produtor rural",
            ),
            (2350, "AQUISIÇÕES DE SERVIÇOS DE TRANSPORTE"),
            (
                2351,
                "Aquisição de serviço de transporte para execução de serviço da mesma natureza",
            ),
            (
                2352,
                "Aquisição de serviço de transporte por estabelecimento industrial",
            ),
            (
                2353,
                "Aquisição de serviço de transporte por estabelecimento comercial",
            ),
            (
                2354,
                "Aquisição de serviço de transporte por estabelecimento de prestador de serviço de comunicação",
            ),
            (
                2355,
                "Aquisição de serviço de transporte por estabelecimento de geradora ou de distribuidora de energia elétrica",
            ),
            (
                2356,
                "Aquisição de serviço de transporte por estabelecimento de produtor rural",
            ),
            (
                2400,
                "ENTRADAS DE MERCADORIAS SUJEITAS AO REGIME DE SUBSTITUIÇÃO TRIBUTÁRIA",
            ),
            (
                2401,
                "Compra para industrialização ou produção rural em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                2403,
                "Compra para comercialização em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                2406,
                "Compra de bem para o ativo imobilizado cuja mercadoria está sujeita ao regime de substituição tributária",
            ),
            (
                2407,
                "Compra de mercadoria para uso ou consumo cuja mercadoria está sujeita ao regime de substituição tributária",
            ),
            (
                2408,
                "Transferência para industrialização ou produção rural em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                2409,
                "Transferência para comercialização em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                2410,
                "Devolução de venda de produção do estabelecimento em operação com produto sujeito ao regime de substituição tributária",
            ),
            (
                2411,
                "Devolução de venda de mercadoria adquirida ou recebida de terceiros em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                2414,
                "Retorno de produção do estabelecimento, remetida para venda fora do estabelecimento em operação com produto sujeito ao regime de substituição tributária",
            ),
            (
                2415,
                "Retorno de mercadoria adquirida ou recebida de terceiros, remetida para venda fora do estabelecimento em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                2500,
                "ENTRADAS DE MERCADORIAS REMETIDAS PARA FORMAÇÃO DE LOTE OU COM FIM ESPECÍFICO DE EXPORTAÇÃO E EVENTUAIS DEVOLUÇÕES",
            ),
            (
                2501,
                "Entrada de mercadoria recebida com fim específico de exportação",
            ),
            (
                2503,
                "Entrada decorrente de devolução de produto remetido com fim específico de exportação, de produção do estabelecimento",
            ),
            (
                2504,
                "Entrada decorrente de devolução de mercadoria remetida com fim específico de exportação, adquirida ou recebida de terceiros",
            ),
            (
                2505,
                "Entrada decorrente de devolução de mercadorias remetidas para formação de lote de exportação, de produtos industrializados ou produzidos pelo próprio estabelecimento",
            ),
            (
                2506,
                "Entrada decorrente de devolução de mercadorias, adquiridas ou recebidas de terceiros, remetidas para formação de lote de exportação",
            ),
            (
                2550,
                "OPERAÇÕES COM BENS DE ATIVO IMOBILIZADO E MATERIAIS PARA USO OU CONSUMO",
            ),
            (2551, "Compra de bem para o ativo imobilizado"),
            (2552, "Transferência de bem do ativo imobilizado"),
            (2553, "Devolução de venda de bem do ativo imobilizado"),
            (
                2554,
                "Retorno de bem do ativo imobilizado remetido para uso fora do estabelecimento",
            ),
            (
                2555,
                "Entrada de bem do ativo imobilizado de terceiro, remetido para uso no estabelecimento",
            ),
            (2556, "Compra de material para uso ou consumo"),
            (2557, "Transferência de material para uso ou consumo"),
            (2600, "CRÉDITOS E RESSARCIMENTOS DE ICMS"),
            (
                2603,
                "Ressarcimento de ICMS retido por substituição tributária",
            ),
            (
                2650,
                "ENTRADAS DE COMBUSTÍVEIS, DERIVADOS OU NÃO DE PETRÓLEO E LUBRIFICANTES",
            ),
            (
                2651,
                "Compra de combustível ou lubrificante para industrialização subseqüente",
            ),
            (
                2652,
                "Compra de combustível ou lubrificante para comercialização",
            ),
            (
                2653,
                "Compra de combustível ou lubrificante por consumidor ou usuário final",
            ),
            (
                2658,
                "Transferência de combustível e lubrificante para industrialização",
            ),
            (
                2659,
                "Transferência de combustível e lubrificante para comercialização",
            ),
            (
                2660,
                "Devolução de venda de combustível ou lubrificante destinado à industrialização subseqüente",
            ),
            (
                2661,
                "Devolução de venda de combustível ou lubrificante destinado à comercialização",
            ),
            (
                2662,
                "Devolução de venda de combustível ou lubrificante destinado a consumidor ou usuário final",
            ),
            (
                2663,
                "Entrada de combustível ou lubrificante para armazenagem",
            ),
            (
                2664,
                "Retorno de combustível ou lubrificante remetido para armazenagem",
            ),
            (
                2900,
                "OUTRAS ENTRADAS DE MERCADORIAS OU AQUISIÇÕES DE SERVIÇOS",
            ),
            (2901, "Entrada para industrialização por encomenda"),
            (
                2902,
                "Retorno de mercadoria remetida para industrialização por encomenda",
            ),
            (
                2903,
                "Entrada de mercadoria remetida para industrialização e não aplicada no referido processo",
            ),
            (
                2904,
                "Retorno de remessa para venda fora do estabelecimento",
            ),
            (
                2905,
                "Entrada de mercadoria recebida para depósito em depósito fechado ou armazém geral",
            ),
            (
                2906,
                "Retorno de mercadoria remetida para depósito fechado ou armazém geral",
            ),
            (
                2907,
                "Retorno simbólico de mercadoria remetida para depósito fechado ou armazém geral",
            ),
            (2908, "Entrada de bem por conta de contrato de comodato"),
            (
                2909,
                "Retorno de bem remetido por conta de contrato de comodato",
            ),
            (2910, "Entrada de bonificação, doação ou brinde"),
            (2911, "Entrada de amostra grátis"),
            (
                2912,
                "Entrada de mercadoria ou bem recebido para demonstração ou mostruário",
            ),
            (
                2913,
                "Retorno de mercadoria ou bem remetido para demonstração, mostruário ou treinamento",
            ),
            (
                2914,
                "Retorno de mercadoria ou bem remetido para exposição ou feira",
            ),
            (
                2915,
                "Entrada de mercadoria ou bem recebido para conserto ou reparo",
            ),
            (
                2916,
                "Retorno de mercadoria ou bem remetido para conserto ou reparo",
            ),
            (
                2917,
                "Entrada de mercadoria recebida em consignação mercantil ou industrial",
            ),
            (
                2918,
                "Devolução de mercadoria remetida em consignação mercantil ou industrial",
            ),
            (
                2919,
                "Devolução simbólica de mercadoria vendida ou utilizada em processo industrial, remetida anteriormente em consignação mercantil ou industrial",
            ),
            (2920, "Entrada de vasilhame ou sacaria"),
            (2921, "Retorno de vasilhame ou sacaria"),
            (
                2922,
                "Lançamento efetuado a título de simples faturamento decorrente de compra para recebimento futuro",
            ),
            (
                2923,
                "Entrada de mercadoria recebida do vendedor remetente, em venda à ordem",
            ),
            (
                2924,
                "Entrada para industrialização por conta e ordem do adquirente da mercadoria, quando esta não transitar pelo estabelecimento do adquirente",
            ),
            (
                2925,
                "Retorno de mercadoria remetida para industrialização por conta e ordem do adquirente da mercadoria, quando esta não transitar pelo estabelecimento do adquirente",
            ),
            (
                2931,
                "Lançamento efetuado pelo tomador do serviço de transporte quando a responsabilidade de retenção do imposto for atribuída ao remetente ou alienante da mercadoria, pelo serviço de transporte realizado por transportador autônomo ou por transportador não inscrito na unidade da Federação onde iniciado o serviço",
            ),
            (
                2932,
                "Aquisição de serviço de transporte iniciado em unidade da Federação diversa daquela onde inscrito o prestador",
            ),
            (2933, "Aquisição de serviço tributado pelo ISSQN"),
            (
                2934,
                "Entrada simbólica de mercadoria recebida para depósito fechado ou armazém geral",
            ),
            (
                2949,
                "Outra entrada de mercadoria ou prestação de serviço não especificado",
            ),
            (3000, "ENTRADAS OU AQUISIÇÕES DE SERVIÇOS DO EXTERIOR"),
            (
                3100,
                "COMPRAS PARA INDUSTRIALIZAÇÃO, PRODUÇÃO RURAL, COMERCIALIZAÇÃO OU PRESTAÇÃO DE SERVIÇOS",
            ),
            (3101, "Compra para industrialização ou produção rural"),
            (3102, "Compra para comercialização"),
            (
                3126,
                "Compra para utilização na prestação de serviço sujeita ao ICMS",
            ),
            (
                3127,
                "Compra para industrialização sob o regime de 'drawback'",
            ),
            (
                3128,
                "Compra para utilização na prestação de serviço sujeita ao ISSQN",
            ),
            (
                3129,
                "Compra para industrialização sob o Regime Aduaneiro Especial de Entreposto Industrial sob Controle Informatizado do Sistema Público de Escrituração Digital (Recof-Sped)",
            ),
            (
                3200,
                "DEVOLUÇÕES DE VENDAS DE PRODUÇÃO PRÓPRIA, DE TERCEIROS OU ANULAÇÕES DE VALORES",
            ),
            (3201, "Devolução de venda de produção do estabelecimento"),
            (
                3202,
                "Devolução de venda de mercadoria adquirida ou recebida de terceiros",
            ),
            (
                3205,
                "Anulação de valor relativo à prestação de serviço de comunicação",
            ),
            (
                3206,
                "Anulação de valor relativo à prestação de serviço de transporte",
            ),
            (
                3207,
                "Anulação de valor relativo à venda de energia elétrica",
            ),
            (
                3211,
                "Devolução de venda de produção do estabelecimento sob o regime de 'drawback'",
            ),
            (
                3212,
                "Devolução de venda no mercado externo de mercadoria industrializada sob o Regime Aduaneiro Especial de Entreposto Industrial sob Controle Informatizado do Sistema Público de Escrituração Digital (Recof-Sped)",
            ),
            (3250, "COMPRAS DE ENERGIA ELÉTRICA"),
            (
                3251,
                "Compra de energia elétrica para distribuição ou comercialização",
            ),
            (3300, "AQUISIÇÕES DE SERVIÇOS DE COMUNICAÇÃO"),
            (
                3301,
                "Aquisição de serviço de comunicação para execução de serviço da mesma natureza",
            ),
            (3350, "AQUISIÇÕES DE SERVIÇOS DE TRANSPORTE"),
            (
                3351,
                "Aquisição de serviço de transporte para execução de serviço da mesma natureza",
            ),
            (
                3352,
                "Aquisição de serviço de transporte por estabelecimento industrial",
            ),
            (
                3353,
                "Aquisição de serviço de transporte por estabelecimento comercial",
            ),
            (
                3354,
                "Aquisição de serviço de transporte por estabelecimento de prestador de serviço de comunicação",
            ),
            (
                3355,
                "Aquisição de serviço de transporte por estabelecimento de geradora ou de distribuidora de energia elétrica",
            ),
            (
                3356,
                "Aquisição de serviço de transporte por estabelecimento de produtor rural",
            ),
            (
                3500,
                "ENTRADAS DE MERCADORIAS REMETIDAS COM FIM ESPECÍFICO DE EXPORTAÇÃO E EVENTUAIS DEVOLUÇÕES",
            ),
            (
                3503,
                "Devolução de mercadoria exportada que tenha sido recebida com fim específico de exportação",
            ),
            (
                3550,
                "OPERAÇÕES COM BENS DE ATIVO IMOBILIZADO E MATERIAIS PARA USO OU CONSUMO",
            ),
            (3551, "Compra de bem para o ativo imobilizado"),
            (3553, "Devolução de venda de bem do ativo imobilizado"),
            (3556, "Compra de material para uso ou consumo"),
            (
                3650,
                "ENTRADAS DE COMBUSTÍVEIS, DERIVADOS OU NÃO DE PETRÓLEO E LUBRIFICANTES",
            ),
            (
                3651,
                "Compra de combustível ou lubrificante para industrialização subseqüente",
            ),
            (
                3652,
                "Compra de combustível ou lubrificante para comercialização",
            ),
            (
                3653,
                "Compra de combustível ou lubrificante por consumidor ou usuário final",
            ),
            (
                3900,
                "OUTRAS ENTRADAS DE MERCADORIAS OU AQUISIÇÕES DE SERVIÇOS",
            ),
            (
                3930,
                "Lançamento efetuado a título de entrada de bem sob amparo de regime especial aduaneiro de admissão temporária",
            ),
            (
                3949,
                "Outra entrada de mercadoria ou prestação de serviço não especificado",
            ),
            (5000, "SAÍDAS OU PRESTAÇÕES DE SERVIÇOS PARA O ESTADO"),
            (5100, "VENDAS DE PRODUÇÃO PRÓPRIA OU DE TERCEIROS"),
            (5101, "Venda de produção do estabelecimento"),
            (
                5102,
                "Venda de mercadoria adquirida ou recebida de terceiros",
            ),
            (
                5103,
                "Venda de produção do estabelecimento, efetuada fora do estabelecimento",
            ),
            (
                5104,
                "Venda de mercadoria adquirida ou recebida de terceiros, efetuada fora do estabelecimento",
            ),
            (
                5105,
                "Venda de produção do estabelecimento que não deva por ele transitar",
            ),
            (
                5106,
                "Venda de mercadoria adquirida ou recebida de terceiros, que não deva por ele transitar",
            ),
            (
                5109,
                "Venda de produção do estabelecimento, destinada à Zona Franca de Manaus ou Áreas de Livre Comércio",
            ),
            (
                5110,
                "Venda de mercadoria adquirida ou recebida de terceiros, destinada à Zona Franca de Manaus ou Áreas de Livre Comércio",
            ),
            (
                5111,
                "Venda de produção do estabelecimento remetida anteriormente em consignação industrial",
            ),
            (
                5112,
                "Venda de mercadoria adquirida ou recebida de terceiros remetida anteriormente em consignação industrial",
            ),
            (
                5113,
                "Venda de produção do estabelecimento remetida anteriormente em consignação mercantil",
            ),
            (
                5114,
                "Venda de mercadoria adquirida ou recebida de terceiros remetida anteriormente em consignação mercantil",
            ),
            (
                5115,
                "Venda de mercadoria adquirida ou recebida de terceiros, recebida anteriormente em consignação mercantil",
            ),
            (
                5116,
                "Venda de produção do estabelecimento originada de encomenda para entrega futura",
            ),
            (
                5117,
                "Venda de mercadoria adquirida ou recebida de terceiros, originada de encomenda para entrega futura",
            ),
            (
                5118,
                "Venda de produção do estabelecimento entregue ao destinatário por conta e ordem do adquirente originário, em venda à ordem",
            ),
            (
                5119,
                "Venda de mercadoria adquirida ou recebida de terceiros entregue ao destinatário por conta e ordem do adquirente originário, em venda à ordem",
            ),
            (
                5120,
                "Venda de mercadoria adquirida ou recebida de terceiros entregue ao destinatário pelo vendedor remetente, em venda à ordem",
            ),
            (
                5122,
                "Venda de produção do estabelecimento remetida para industrialização, por conta e ordem do adquirente, sem transitar pelo estabelecimento do adquirente",
            ),
            (
                5123,
                "Venda de mercadoria adquirida ou recebida de terceiros remetida para industrialização, por conta e ordem do adquirente, sem transitar pelo estabelecimento do adquirente",
            ),
            (5124, "Industrialização efetuada para outra empresa"),
            (
                5125,
                "Industrialização efetuada para outra empresa quando a mercadoria recebida para utilização no processo de industrialização não transitar pelo estabelecimento adquirente da mercadoria",
            ),
            (
                5129,
                "Venda de insumo importado e de mercadoria industrializada sob o amparo do Regime Aduaneiro Especial de Entreposto Industrial sob Controle Informatizado do Sistema Público de Escrituração Digital (Recof-Sped)",
            ),
            (
                5131,
                "Remessa de produção do estabelecimento, com previsão de posterior ajuste ou fixação de preço, de ato cooperativo",
            ),
            (
                5132,
                "Fixação de preço de produção do estabelecimento, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço de ato cooperativo",
            ),
            (5150, "TRANSFERÊNCIAS DE PRODUÇÃO PRÓPRIA OU DE TERCEIROS"),
            (5151, "Transferência de produção do estabelecimento"),
            (
                5152,
                "Transferência de mercadoria adquirida ou recebida de terceiros",
            ),
            (5153, "Transferência de energia elétrica"),
            (
                5155,
                "Transferência de produção do estabelecimento, que não deva por ele transitar",
            ),
            (
                5156,
                "Transferência de mercadoria adquirida ou recebida de terceiros, que não deva por ele transitar",
            ),
            (
                5159,
                "Fornecimento de produção do estabelecimento de ato cooperativo",
            ),
            (
                5160,
                "Fornecimento de mercadoria adquirida ou recebida de terceiros de ato cooperativo",
            ),
            (
                5200,
                "DEVOLUÇÕES DE COMPRAS PARA INDUSTRIALIZAÇÃO, PRODUÇÃO RURAL, COMERCIALIZAÇÃO OU ANULAÇÕES DE VALORES",
            ),
            (
                5201,
                "Devolução de compra para industrialização ou produção rural",
            ),
            (5202, "Devolução de compra para comercialização"),
            (
                5205,
                "Anulação de valor relativo a aquisição de serviço de comunicação",
            ),
            (
                5206,
                "Anulação de valor relativo a aquisição de serviço de transporte",
            ),
            (
                5207,
                "Anulação de valor relativo à compra de energia elétrica",
            ),
            (
                5208,
                "Devolução de mercadoria recebida em transferência para industrialização ou produção rural",
            ),
            (
                5209,
                "Devolução de mercadoria recebida em transferência para comercialização",
            ),
            (
                5210,
                "Devolução de compra para utilização na prestação de serviço",
            ),
            (
                5213,
                "Devolução de entrada de mercadoria com previsão de posterior ajuste ou fixação de preço, em ato cooperativo",
            ),
            (
                5214,
                "Devolução de fixação de preço de produção do estabelecimento produtor, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço, de ato cooperativo, para comercialização",
            ),
            (
                5215,
                "Devolução de fixação de preço de produção do estabelecimento produtor, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço, de ato cooperativo, para industrialização",
            ),
            (5250, "VENDAS DE ENERGIA ELÉTRICA"),
            (
                5251,
                "Venda de energia elétrica para distribuição ou comercialização",
            ),
            (
                5252,
                "Venda de energia elétrica para estabelecimento industrial",
            ),
            (
                5253,
                "Venda de energia elétrica para estabelecimento comercial",
            ),
            (
                5254,
                "Venda de energia elétrica para estabelecimento prestador de serviço de transporte",
            ),
            (
                5255,
                "Venda de energia elétrica para estabelecimento prestador de serviço de comunicação",
            ),
            (
                5256,
                "Venda de energia elétrica para estabelecimento de produtor rural",
            ),
            (
                5257,
                "Venda de energia elétrica para consumo por demanda contratada",
            ),
            (5258, "Venda de energia elétrica a não contribuinte"),
            (5300, "PRESTAÇÕES DE SERVIÇOS DE COMUNICAÇÃO"),
            (
                5301,
                "Prestação de serviço de comunicação para execução de serviço da mesma natureza",
            ),
            (
                5302,
                "Prestação de serviço de comunicação a estabelecimento industrial",
            ),
            (
                5303,
                "Prestação de serviço de comunicação a estabelecimento comercial",
            ),
            (
                5304,
                "Prestação de serviço de comunicação a estabelecimento de prestador de serviço de transporte",
            ),
            (
                5305,
                "Prestação de serviço de comunicação a estabelecimento de geradora ou de distribuidora de energia elétrica",
            ),
            (
                5306,
                "Prestação de serviço de comunicação a estabelecimento de produtor rural",
            ),
            (
                5307,
                "Prestação de serviço de comunicação a não contribuinte",
            ),
            (5350, "PRESTAÇÕES DE SERVIÇOS DE TRANSPORTE"),
            (
                5351,
                "Prestação de serviço de transporte para execução de serviço da mesma natureza",
            ),
            (
                5352,
                "Prestação de serviço de transporte a estabelecimento industrial",
            ),
            (
                5353,
                "Prestação de serviço de transporte a estabelecimento comercial",
            ),
            (
                5354,
                "Prestação de serviço de transporte a estabelecimento de prestador de serviço de comunicação",
            ),
            (
                5355,
                "Prestação de serviço de transporte a estabelecimento de geradora ou de distribuidora de energia elétrica",
            ),
            (
                5356,
                "Prestação de serviço de transporte a estabelecimento de produtor rural",
            ),
            (
                5357,
                "Prestação de serviço de transporte a não contribuinte",
            ),
            (
                5359,
                "Prestação de serviço de transporte a contribuinte ou a não contribuinte quando a mercadoria transportada está dispensada de emissão de nota fiscal",
            ),
            (
                5360,
                "Prestação de serviço de transporte a contribuinte substituto em relação ao serviço de transporte",
            ),
            (
                5400,
                "SAÍDAS DE MERCADORIAS SUJEITAS AO REGIME DE SUBSTITUIÇÃO TRIBUTÁRIA",
            ),
            (
                5401,
                "Venda de produção do estabelecimento em operação com produto sujeito ao regime de substituição tributária, na condição de contribuinte substituto",
            ),
            (
                5402,
                "Venda de produção do estabelecimento de produto sujeito ao regime de substituição tributária, em operação entre contribuintes substitutos do mesmo produto",
            ),
            (
                5403,
                "Venda de mercadoria adquirida ou recebida de terceiros em operação com mercadoria sujeita ao regime de substituição tributária, na condição de contribuinte substituto",
            ),
            (
                5405,
                "Venda de mercadoria adquirida ou recebida de terceiros em operação com mercadoria sujeita ao regime de substituição tributária, na condição de contribuinte substituído",
            ),
            (
                5408,
                "Transferência de produção do estabelecimento em operação com produto sujeito ao regime de substituição tributária",
            ),
            (
                5409,
                "Transferência de mercadoria adquirida ou recebida de terceiros em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                5410,
                "Devolução de compra para industrialização ou produção rural em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                5411,
                "Devolução de compra para comercialização em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                5412,
                "Devolução de bem do ativo imobilizado, em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                5413,
                "Devolução de mercadoria destinada ao uso ou consumo, em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                5414,
                "Remessa de produção do estabelecimento para venda fora do estabelecimento em operação com produto sujeito ao regime de substituição tributária",
            ),
            (
                5415,
                "Remessa de mercadoria adquirida ou recebida de terceiros para venda fora do estabelecimento, em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (5450, "SISTEMAS DE INTEGRAÇÃO"),
            (
                5451,
                "Remessa de animal e de insumo para estabelecimento produtor",
            ),
            (
                5500,
                "REMESSAS PARA FORMAÇÃO DE LOTE E COM FIM ESPECÍFICO DE EXPORTAÇÃO E EVENTUAIS DEVOLUÇÕES",
            ),
            (
                5501,
                "Remessa de produção do estabelecimento, com fim específico de exportação",
            ),
            (
                5502,
                "Remessa de mercadoria adquirida ou recebida de terceiros, com fim específico de exportação",
            ),
            (
                5503,
                "Devolução de mercadoria recebida com fim específico de exportação",
            ),
            (
                5504,
                "Remessa de mercadorias para formação de lote de exportação, de produtos industrializados ou produzidos pelo próprio estabelecimento",
            ),
            (
                5505,
                "Remessa de mercadorias, adquiridas ou recebidas de terceiros, para formação de lote de exportação",
            ),
            (
                5550,
                "OPERAÇÕES COM BENS DE ATIVO IMOBILIZADO E MATERIAIS PARA USO OU CONSUMO",
            ),
            (5551, "Venda de bem do ativo imobilizado"),
            (5552, "Transferência de bem do ativo imobilizado"),
            (5553, "Devolução de compra de bem para o ativo imobilizado"),
            (
                5554,
                "Remessa de bem do ativo imobilizado para uso fora do estabelecimento",
            ),
            (
                5555,
                "Devolução de bem do ativo imobilizado de terceiro, recebido para uso no estabelecimento",
            ),
            (5556, "Devolução de compra de material de uso ou consumo"),
            (5557, "Transferência de material de uso ou consumo"),
            (5600, "CRÉDITOS E RESSARCIMENTOS DE ICMS"),
            (5601, "Transferência de crédito de ICMS acumulado"),
            (
                5602,
                "Transferência de saldo credor de ICMS para outro estabelecimento da mesma empresa, destinado à compensação de saldo devedor de ICMS",
            ),
            (
                5603,
                "Ressarcimento de ICMS retido por substituição tributária",
            ),
            (
                5605,
                "Transferência de saldo devedor de ICMS de outro estabelecimento da mesma empresa",
            ),
            (
                5606,
                "Utilização de saldo credor de ICMS para extinção por compensação de débitos fiscais",
            ),
            (
                5650,
                "SAÍDAS DE COMBUSTÍVEIS, DERIVADOS OU NÃO DE PETRÓLEO E LUBRIFICANTES",
            ),
            (
                5651,
                "Venda de combustível ou lubrificante de produção do estabelecimento destinado à industrialização subseqüente",
            ),
            (
                5652,
                "Venda de combustível ou lubrificante de produção do estabelecimento destinado à comercialização",
            ),
            (
                5653,
                "Venda de combustível ou lubrificante de produção do estabelecimento destinado a consumidor ou usuário final",
            ),
            (
                5654,
                "Venda de combustível ou lubrificante adquirido ou recebido de terceiros destinado à industrialização subseqüente",
            ),
            (
                5655,
                "Venda de combustível ou lubrificante adquirido ou recebido de terceiros destinado à comercialização",
            ),
            (
                5656,
                "Venda de combustível ou lubrificante adquirido ou recebido de terceiros destinado a consumidor ou usuário final",
            ),
            (
                5657,
                "Remessa de combustível ou lubrificante adquirido ou recebido de terceiros para venda fora do estabelecimento",
            ),
            (
                5658,
                "Transferência de combustível ou lubrificante de produção do estabelecimento",
            ),
            (
                5659,
                "Transferência de combustível ou lubrificante adquirido ou recebido de terceiro",
            ),
            (
                5660,
                "Devolução de compra de combustível ou lubrificante adquirido para industrialização subseqüente",
            ),
            (
                5661,
                "Devolução de compra de combustível ou lubrificante adquirido para comercialização",
            ),
            (
                5662,
                "Devolução de compra de combustível ou lubrificante adquirido por consumidor ou usuário final",
            ),
            (
                5663,
                "Remessa para armazenagem de combustível ou lubrificante",
            ),
            (
                5664,
                "Retorno de combustível ou lubrificante recebido para armazenagem",
            ),
            (
                5665,
                "Retorno simbólico de combustível ou lubrificante recebido para armazenagem",
            ),
            (
                5666,
                "Remessa por conta e ordem de terceiros de combustível ou lubrificante recebido para armazenagem",
            ),
            (
                5667,
                "Venda de combustível ou lubrificante a consumidor ou usuário final estabelecido em outra unidade da Federação",
            ),
            (
                5900,
                "OUTRAS SAÍDAS DE MERCADORIAS OU PRESTAÇÕES DE SERVIÇOS",
            ),
            (5901, "Remessa para industrialização por encomenda"),
            (
                5902,
                "Retorno de mercadoria utilizada na industrialização por encomenda",
            ),
            (
                5903,
                "Retorno de mercadoria recebida para industrialização e não aplicada no referido processo",
            ),
            (5904, "Remessa para venda fora do estabelecimento"),
            (5905, "Remessa para depósito fechado ou armazém geral"),
            (
                5906,
                "Retorno de mercadoria depositada em depósito fechado ou armazém geral",
            ),
            (
                5907,
                "Retorno simbólico de mercadoria depositada em depósito fechado ou armazém geral",
            ),
            (5908, "Remessa de bem por conta de contrato de comodato"),
            (
                5909,
                "Retorno de bem recebido por conta de contrato de comodato",
            ),
            (5910, "Remessa em bonificação, doação ou brinde"),
            (5911, "Remessa de amostra grátis"),
            (
                5912,
                "Remessa de mercadoria ou bem para demonstração, mostruário ou treinamento",
            ),
            (
                5913,
                "Retorno de mercadoria ou bem recebido para demonstração ou mostruário",
            ),
            (5914, "Remessa de mercadoria ou bem para exposição ou feira"),
            (5915, "Remessa de mercadoria ou bem para conserto ou reparo"),
            (
                5916,
                "Retorno de mercadoria ou bem recebido para conserto ou reparo",
            ),
            (
                5917,
                "Remessa de mercadoria em consignação mercantil ou industrial",
            ),
            (
                5918,
                "Devolução de mercadoria recebida em consignação mercantil ou industrial",
            ),
            (
                5919,
                "Devolução simbólica de mercadoria vendida ou utilizada em processo industrial, recebida anteriormente em consignação mercantil ou industrial",
            ),
            (5920, "Remessa de vasilhame ou sacaria"),
            (5921, "Devolução de vasilhame ou sacaria"),
            (
                5922,
                "Lançamento efetuado a título de simples faturamento decorrente de venda para entrega futura",
            ),
            (
                5923,
                "Remessa de mercadoria por conta e ordem de terceiros, em venda à ordem ou em operações com armazém geral ou depósito fechado",
            ),
            (
                5924,
                "Remessa para industrialização por conta e ordem do adquirente da mercadoria, quando esta não transitar pelo estabelecimento do adquirente",
            ),
            (
                5925,
                "Retorno de mercadoria recebida para industrialização por conta e ordem do adquirente da mercadoria, quando aquela não transitar pelo estabelecimento do adquirente",
            ),
            (
                5926,
                "Lançamento efetuado a título de reclassificação de mercadoria decorrente de formação de kit ou de sua desagregação",
            ),
            (
                5927,
                "Lançamento efetuado a título de baixa de estoque decorrente de perda, roubo ou deterioração",
            ),
            (
                5928,
                "Lançamento efetuado a título de baixa de estoque decorrente do encerramento da atividade da empresa",
            ),
            (
                5929,
                "Lançamento efetuado em decorrência de emissão de documento fiscal relativo a operação ou prestação também registradaem equipamento Emissorde Cupom Fiscal - ECF",
            ),
            (
                5931,
                "Lançamento efetuado em decorrência da responsabilidade de retenção do imposto por substituição tributária, atribuída ao remetente ou alienante da mercadoria, pelo serviço de transporte realizado por transportador autônomo ou por transportador não inscrito na unidade da Federação onde iniciado o serviço",
            ),
            (
                5932,
                "Prestação de serviço de transporte iniciada em unidade da Federação diversa daquela onde inscrito o prestador",
            ),
            (5933, "Prestação de serviço tributado pelo ISSQN"),
            (
                5934,
                "Remessa simbólica de mercadoria depositada em armazém geral ou depósito fechado",
            ),
            (
                5949,
                "Outra saída de mercadoria ou prestação de serviço não especificado",
            ),
            (6000, "SAÍDAS OU PRESTAÇÕES DE SERVIÇOS PARA OUTROS ESTADOS"),
            (6100, "VENDAS DE PRODUÇÃO PRÓPRIA OU DE TERCEIROS"),
            (6101, "Venda de produção do estabelecimento"),
            (
                6102,
                "Venda de mercadoria adquirida ou recebida de terceiros",
            ),
            (
                6103,
                "Venda de produção do estabelecimento, efetuada fora do estabelecimento",
            ),
            (
                6104,
                "Venda de mercadoria adquirida ou recebida de terceiros, efetuada fora do estabelecimento",
            ),
            (
                6105,
                "Venda de produção do estabelecimento que não deva por ele transitar",
            ),
            (
                6106,
                "Venda de mercadoria adquirida ou recebida de terceiros, que não deva por ele transitar",
            ),
            (
                6107,
                "Venda de produção do estabelecimento, destinada a não contribuinte",
            ),
            (
                6108,
                "Venda de mercadoria adquirida ou recebida de terceiros, destinada a não contribuinte",
            ),
            (
                6109,
                "Venda de produção do estabelecimento, destinada à Zona Franca de Manaus ou Áreas de Livre Comércio",
            ),
            (
                6110,
                "Venda de mercadoria adquirida ou recebida de terceiros, destinada à Zona Franca de Manaus ou Áreas de Livre Comércio",
            ),
            (
                6111,
                "Venda de produção do estabelecimento remetida anteriormente em consignação industrial",
            ),
            (
                6112,
                "Venda de mercadoria adquirida ou recebida de Terceiros remetida anteriormente em consignação industrial",
            ),
            (
                6113,
                "Venda de produção do estabelecimento remetida anteriormente em consignação mercantil",
            ),
            (
                6114,
                "Venda de mercadoria adquirida ou recebida de terceiros remetida anteriormente em consignação mercantil",
            ),
            (
                6115,
                "Venda de mercadoria adquirida ou recebida de terceiros, recebida anteriormente em consignação mercantil",
            ),
            (
                6116,
                "Venda de produção do estabelecimento originada de encomenda para entrega futura",
            ),
            (
                6117,
                "Venda de mercadoria adquirida ou recebida de terceiros, originada de encomenda para entrega futura",
            ),
            (
                6118,
                "Venda de produção do estabelecimento entregue ao destinatário por conta e ordem do adquirente originário, em venda à ordem",
            ),
            (
                6119,
                "Venda de mercadoria adquirida ou recebida de terceiros entregue ao destinatário por conta e ordem do adquirente originário, em venda à ordem",
            ),
            (
                6120,
                "Venda de mercadoria adquirida ou recebida de terceiros entregue ao destinatário pelo vendedor remetente, em venda à ordem",
            ),
            (
                6122,
                "Venda de produção do estabelecimento remetida para industrialização, por conta e ordem do adquirente, sem transitar pelo estabelecimento do adquirente",
            ),
            (
                6123,
                "Venda de mercadoria adquirida ou recebida de terceiros remetida para industrialização, por conta e ordem do adquirente, sem transitar pelo estabelecimento do adquirente",
            ),
            (6124, "Industrialização efetuada para outra empresa"),
            (
                6125,
                "Industrialização efetuada para outra empresa quando a mercadoria recebida para utilização no processo de industrialização não transitar pelo estabelecimento adquirente da mercadoria",
            ),
            (
                6129,
                "Venda de insumo importado e de mercadoria industrializada sob o amparo do Regime Aduaneiro Especial de Entreposto Industrial sob Controle Informatizado do Sistema Público de Escrituração Digital (Recof-Sped)",
            ),
            (
                6131,
                "Remessa de produção de estabelecimento, com previsão de posterior ajuste ou fixação de preço de ato cooperativo",
            ),
            (
                6132,
                "Fixação de preço de produção do estabelecimento, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço ou fixação de preço de ato cooperativo",
            ),
            (6150, "TRANSFERÊNCIAS DE PRODUÇÃO PRÓPRIA OU DE TERCEIROS"),
            (6151, "Transferência de produção do estabelecimento"),
            (
                6152,
                "Transferência de mercadoria adquirida ou recebida de terceiros",
            ),
            (6153, "Transferência de energia elétrica"),
            (
                6155,
                "Transferência de produção do estabelecimento, que não deva por ele transitar",
            ),
            (
                6156,
                "Transferência de mercadoria adquirida ou recebida de terceiros, que não deva por ele transitar",
            ),
            (
                6159,
                "Fornecimento de produção do estabelecimento de ato cooperativo",
            ),
            (
                6160,
                "Fornecimento de mercadoria adquirida ou recebida de terceiros de ato cooperativo",
            ),
            (
                6200,
                "DEVOLUÇÕES DE COMPRAS PARA INDUSTRIALIZAÇÃO, PRODUÇÃO RURAL, COMERCIALIZAÇÃO OU ANULAÇÕES DE VALORES",
            ),
            (
                6201,
                "Devolução de compra para industrialização ou produção rural",
            ),
            (6202, "Devolução de compra para comercialização"),
            (
                6205,
                "Anulação de valor relativo a aquisição de serviço de comunicação",
            ),
            (
                6206,
                "Anulação de valor relativo a aquisição de serviço de transporte",
            ),
            (
                6207,
                "Anulação de valor relativo à compra de energia elétrica",
            ),
            (
                6208,
                "Devolução de mercadoria recebida em transferência para industrialização ou produção rural",
            ),
            (
                6209,
                "Devolução de mercadoria recebida em transferência para comercialização",
            ),
            (
                6210,
                "Devolução de compra para utilização na prestação de serviço",
            ),
            (
                6213,
                "Devolução de entrada de mercadoria com previsão de posterior ajuste ou fixação de preço, em ato cooperativo",
            ),
            (
                6214,
                "Devolução de fixação de preço de produção do estabelecimento produtor, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço, de ato cooperativo, para comercialização",
            ),
            (
                6215,
                "Devolução de fixação de preço de produção do estabelecimento produtor, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço, de ato cooperativo para industrialização",
            ),
            (6250, "VENDAS DE ENERGIA ELÉTRICA"),
            (
                6251,
                "Venda de energia elétrica para distribuição ou comercialização",
            ),
            (
                6252,
                "Venda de energia elétrica para estabelecimento industrial",
            ),
            (
                6253,
                "Venda de energia elétrica para estabelecimento comercial",
            ),
            (
                6254,
                "Venda de energia elétrica para estabelecimento prestador de serviço de transporte",
            ),
            (
                6255,
                "Venda de energia elétrica para estabelecimento prestador de serviço de comunicação",
            ),
            (
                6256,
                "Venda de energia elétrica para estabelecimento de produtor rural",
            ),
            (
                6257,
                "Venda de energia elétrica para consumo por demanda contratada",
            ),
            (6258, "Venda de energia elétrica a não contribuinte"),
            (6300, "PRESTAÇÕES DE SERVIÇOS DE COMUNICAÇÃO"),
            (
                6301,
                "Prestação de serviço de comunicação para execução de serviço da mesma natureza",
            ),
            (
                6302,
                "Prestação de serviço de comunicação a estabelecimento industrial",
            ),
            (
                6303,
                "Prestação de serviço de comunicação a estabelecimento comercial",
            ),
            (
                6304,
                "Prestação de serviço de comunicação a estabelecimento de prestador de serviço de transporte",
            ),
            (
                6305,
                "Prestação de serviço de comunicação a estabelecimento de geradora ou de distribuidora de energia elétrica",
            ),
            (
                6306,
                "Prestação de serviço de comunicação a estabelecimento de produtor rural",
            ),
            (
                6307,
                "Prestação de serviço de comunicação a não contribuinte",
            ),
            (6350, "PRESTAÇÕES DE SERVIÇOS DE TRANSPORTE"),
            (
                6351,
                "Prestação de serviço de transporte para execução de serviço da mesma natureza",
            ),
            (
                6352,
                "Prestação de serviço de transporte a estabelecimento industrial",
            ),
            (
                6353,
                "Prestação de serviço de transporte a estabelecimento comercial",
            ),
            (
                6354,
                "Prestação de serviço de transporte a estabelecimento de prestador de serviço de comunicação",
            ),
            (
                6355,
                "Prestação de serviço de transporte a estabelecimento de geradora ou de distribuidora de energia elétrica",
            ),
            (
                6356,
                "Prestação de serviço de transporte a estabelecimento de produtor rural",
            ),
            (
                6357,
                "Prestação de serviço de transporte a não contribuinte",
            ),
            (
                6359,
                "Prestação de serviço de transporte a contribuinte ou a não contribuinte quando a mercadoria transportada está dispensada de emissão de nota fiscal",
            ),
            (
                6360,
                "Prestação de serviço de transporte a contribuinte substituto em relação ao serviço de transporte",
            ),
            (
                6400,
                "SAÍDAS DE MERCADORIAS SUJEITAS AO REGIME DE SUBSTITUIÇÃO TRIBUTÁRIA",
            ),
            (
                6401,
                "Venda de produção do estabelecimento em operação com produto sujeito ao regime de substituição tributária, na condição de contribuinte substituto",
            ),
            (
                6402,
                "Venda de produção do estabelecimento de produto sujeito ao regime de substituição tributária, em operação entre contribuintes substitutos do mesmo produto",
            ),
            (
                6403,
                "Venda de mercadoria adquirida ou recebida de terceiros em operação com mercadoria sujeita ao regime de substituição tributária, na condição de contribuinte substituto",
            ),
            (
                6404,
                "Venda de mercadoria sujeita ao regime de substituição tributária, cujo imposto já tenha sido retido anteriormente",
            ),
            (
                6408,
                "Transferência de produção do estabelecimento em operação com produto sujeito ao regime de substituição tributária",
            ),
            (
                6409,
                "Transferência de mercadoria adquirida ou recebida de terceiros em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                6410,
                "Devolução de compra para industrialização ou produção rural em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                6411,
                "Devolução de compra para comercialização em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                6412,
                "Devolução de bem do ativo imobilizado, em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                6413,
                "Devolução de mercadoria destinada ao uso ou consumo, em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                6414,
                "Remessa de produção do estabelecimento para venda fora do estabelecimento em operação com produto sujeito ao regime de substituição tributária",
            ),
            (
                6415,
                "Remessa de mercadoria adquirida ou recebida de terceiros para venda fora do estabelecimento, em operação com mercadoria sujeita ao regime de substituição tributária",
            ),
            (
                6500,
                "REMESSAS PARA FORMAÇÃO DE LOTE E COM FIM ESPECÍFICO DE EXPORTAÇÃO E EVENTUAIS DEVOLUÇÕES",
            ),
            (
                6501,
                "Remessa de produção do estabelecimento, com fim específico de exportação",
            ),
            (
                6502,
                "Remessa de mercadoria adquirida ou recebida de terceiros, com fim específico de exportação",
            ),
            (
                6503,
                "Devolução de mercadoria recebida com fim específico de exportação",
            ),
            (
                6504,
                "Remessa de mercadorias para formação de lote de exportação, de produtos industrializados ou produzidos pelo próprio estabelecimento",
            ),
            (
                6505,
                "Remessa de mercadorias, adquiridas ou recebidas de terceiros, para formação de lote de exportação",
            ),
            (
                6550,
                "OPERAÇÕES COM BENS DE ATIVO IMOBILIZADO E MATERIAIS PARA USO OU CONSUMO",
            ),
            (6551, "Venda de bem do ativo imobilizado"),
            (6552, "Transferência de bem do ativo imobilizado"),
            (6553, "Devolução de compra de bem para o ativo imobilizado"),
            (
                6554,
                "Remessa de bem do ativo imobilizado para uso fora do estabelecimento",
            ),
            (
                6555,
                "Devolução de bem do ativo imobilizado de terceiro, recebido para uso no estabelecimento",
            ),
            (6556, "Devolução de compra de material de uso ou consumo"),
            (6557, "Transferência de material de uso ou consumo"),
            (6600, "CRÉDITOS E RESSARCIMENTOS DE ICMS"),
            (
                6603,
                "Ressarcimento de ICMS retido por substituição tributária",
            ),
            (
                6650,
                "SAÍDAS DE COMBUSTÍVEIS, DERIVADOS OU NÃO DE PETRÓLEO E LUBRIFICANTES",
            ),
            (
                6651,
                "Venda de combustível ou lubrificante de produção do estabelecimento destinado à industrialização subseqüente",
            ),
            (
                6652,
                "Venda de combustível ou lubrificante de produção do estabelecimento destinado à comercialização",
            ),
            (
                6653,
                "Venda de combustível ou lubrificante de produção do estabelecimento destinado a consumidor ou usuário final",
            ),
            (
                6654,
                "Venda de combustível ou lubrificante adquirido ou recebido de terceiros destinado à industrialização subseqüente",
            ),
            (
                6655,
                "Venda de combustível ou lubrificante adquirido ou recebido de terceiros destinado à comercialização",
            ),
            (
                6656,
                "Venda de combustível ou lubrificante adquirido ou recebido de terceiros destinado a consumidor ou usuário final",
            ),
            (
                6657,
                "Remessa de combustível ou lubrificante adquirido ou recebido de terceiros para venda fora do estabelecimento",
            ),
            (
                6658,
                "Transferência de combustível ou lubrificante de produção do estabelecimento",
            ),
            (
                6659,
                "Transferência de combustível ou lubrificante adquirido ou recebido de terceiro",
            ),
            (
                6660,
                "Devolução de compra de combustível ou lubrificante adquirido para industrialização subseqüente",
            ),
            (
                6661,
                "Devolução de compra de combustível ou lubrificante adquirido para comercialização",
            ),
            (
                6662,
                "Devolução de compra de combustível ou lubrificante adquirido por consumidor ou usuário final",
            ),
            (
                6663,
                "Remessa para armazenagem de combustível ou lubrificante",
            ),
            (
                6664,
                "Retorno de combustível ou lubrificante recebido para armazenagem",
            ),
            (
                6665,
                "Retorno simbólico de combustível ou lubrificante recebido para armazenagem",
            ),
            (
                6666,
                "Remessa por conta e ordem de terceiros de combustível ou lubrificante recebido para armazenagem",
            ),
            (
                6667,
                "Venda de combustível ou lubrificante a consumidor ou usuário final estabelecido em outra unidade da Federação diferente da que ocorrer o consumo",
            ),
            (
                6900,
                "OUTRAS SAÍDAS DE MERCADORIAS OU PRESTAÇÕES DE SERVIÇOS",
            ),
            (6901, "Remessa para industrialização por encomenda"),
            (
                6902,
                "Retorno de mercadoria utilizada na industrialização por encomenda",
            ),
            (
                6903,
                "Retorno de mercadoria recebida para industrialização e não aplicada no referido processo",
            ),
            (6904, "Remessa para venda fora do estabelecimento"),
            (6905, "Remessa para depósito fechado ou armazém geral"),
            (
                6906,
                "Retorno de mercadoria depositada em depósito fechado ou armazém geral",
            ),
            (
                6907,
                "Retorno simbólico de mercadoria depositada em depósito fechado ou armazém geral",
            ),
            (6908, "Remessa de bem por conta de contrato de comodato"),
            (
                6909,
                "Retorno de bem recebido por conta de contrato de comodato",
            ),
            (6910, "Remessa em bonificação, doação ou brinde"),
            (6911, "Remessa de amostra grátis"),
            (
                6912,
                "Remessa de mercadoria ou bem para demonstração, mostruário ou treinamento",
            ),
            (
                6913,
                "Retorno de mercadoria ou bem recebido para demonstração ou mostruário",
            ),
            (6914, "Remessa de mercadoria ou bem para exposição ou feira"),
            (6915, "Remessa de mercadoria ou bem para conserto ou reparo"),
            (
                6916,
                "Retorno de mercadoria ou bem recebido para conserto ou reparo",
            ),
            (
                6917,
                "Remessa de mercadoria em consignação mercantil ou industrial",
            ),
            (
                6918,
                "Devolução de mercadoria recebida em consignação mercantil ou industrial",
            ),
            (
                6919,
                "Devolução simbólica de mercadoria vendida ou utilizada em processo industrial, recebida anteriormente em consignação mercantil ou industrial",
            ),
            (6920, "Remessa de vasilhame ou sacaria"),
            (6921, "Devolução de vasilhame ou sacaria"),
            (
                6922,
                "Lançamento efetuado a título de simples faturamento decorrente de venda para entrega futura",
            ),
            (
                6923,
                "Remessa de mercadoria por conta e ordem de terceiros, em venda à ordem ou em operações com armazém geral ou depósito fechado",
            ),
            (
                6924,
                "Remessa para industrialização por conta e ordem do adquirente da mercadoria, quando esta não transitar pelo estabelecimento do adquirente",
            ),
            (
                6925,
                "Retorno de mercadoria recebida para industrialização por conta e ordem do adquirente da mercadoria, quando aquela não transitar pelo estabelecimento do adquirente",
            ),
            (
                6929,
                "Lançamento efetuado em decorrência de emissão de documento fiscal relativo a operação ou prestação também registradaem equipamento Emissorde Cupom Fiscal - ECF",
            ),
            (
                6931,
                "Lançamento efetuado em decorrência da responsabilidade de retenção do imposto por substituição tributária, atribuída ao remetente ou alienante da mercadoria, pelo serviço de transporte realizado por transportador autônomo ou por transportador não inscrito na unidade da Federação onde iniciado o serviço",
            ),
            (
                6932,
                "Prestação de serviço de transporte iniciada em unidade da Federação diversa daquela onde inscrito o prestador",
            ),
            (6933, "Prestação de serviço tributado pelo ISSQN"),
            (
                6934,
                "Remessa simbólica de mercadoria depositada em armazém geral ou depósito fechado",
            ),
            (
                6949,
                "Outra saída de mercadoria ou prestação de serviço não especificado",
            ),
            (7000, "SAÍDAS OU PRESTAÇÕES DE SERVIÇOS PARA O EXTERIOR"),
            (7100, "VENDAS DE PRODUÇÃO PRÓPRIA OU DE TERCEIROS"),
            (7101, "Venda de produção do estabelecimento"),
            (
                7102,
                "Venda de mercadoria adquirida ou recebida de terceiros",
            ),
            (
                7105,
                "Venda de produção do estabelecimento, que não deva por ele transitar",
            ),
            (
                7106,
                "Venda de mercadoria adquirida ou recebida de terceiros, que não deva por ele transitar",
            ),
            (
                7127,
                "Venda de produção do estabelecimento sob o regime de 'drawback'",
            ),
            (
                7129,
                "Venda de produção do estabelecimento ao mercado externo de mercadoria industrializada sob o amparo do Regime Aduaneiro Especial de Entreposto Industrial sob Controle Informatizado do Sistema Público de Escrituração Digital (Recof-Sped)",
            ),
            (
                7200,
                "DEVOLUÇÕES DE COMPRAS PARA INDUSTRIALIZAÇÃO, PRODUÇÃO RURAL, COMERCIALIZAÇÃO OU ANULAÇÕES DE VALORES",
            ),
            (
                7201,
                "Devolução de compra para industrialização ou produção rural",
            ),
            (7202, "Devolução de compra para comercialização"),
            (
                7205,
                "Anulação de valor relativo à aquisição de serviço de comunicação",
            ),
            (
                7206,
                "Anulação de valor relativo a aquisição de serviço de transporte",
            ),
            (
                7207,
                "Anulação de valor relativo à compra de energia elétrica",
            ),
            (
                7210,
                "Devolução de compra para utilização na prestação de serviço",
            ),
            (
                7211,
                "Devolução de compras para industrialização sob o regime de drawback",
            ),
            (
                7212,
                "Devolução de compras para industrialização sob o regime de Regime Aduaneiro Especial de Entreposto Industrial sob Controle Informatizado do Sistema Público de Escrituração Digital (Recof-Sped)",
            ),
            (7250, "VENDAS DE ENERGIA ELÉTRICA"),
            (7251, "Venda de energia elétrica para o exterior"),
            (7300, "PRESTAÇÕES DE SERVIÇOS DE COMUNICAÇÃO"),
            (
                7301,
                "Prestação de serviço de comunicação para execução de serviço da mesma natureza",
            ),
            (7350, "PRESTAÇÕES DE SERVIÇO DE TRANSPORTE"),
            (7358, "Prestação de serviço de transporte"),
            (
                7500,
                "EXPORTAÇÃO DE MERCADORIAS RECEBIDAS COM FIM ESPECÍFICO DE EXPORTAÇÃO",
            ),
            (
                7501,
                "Exportação de mercadorias recebidas com fim específico de exportação",
            ),
            (
                7504,
                "Exportação de mercadoria que foi objeto de formação de lote de exportação",
            ),
            (
                7550,
                "OPERAÇÕES COM BENS DE ATIVO IMOBILIZADO E MATERIAIS PARA USO OU CONSUMO",
            ),
            (7551, "Venda de bem do ativo imobilizado"),
            (7553, "Devolução de compra de bem para o ativo imobilizado"),
            (7556, "Devolução de compra de material de uso ou consumo"),
            (
                7650,
                "SAÍDAS DE COMBUSTÍVEIS, DERIVADOS OU NÃO DE PETRÓLEO E LUBRIFICANTES",
            ),
            (
                7651,
                "Venda de combustível ou lubrificante de produção do estabelecimento",
            ),
            (
                7654,
                "Venda de combustível ou lubrificante adquirido ou recebido de terceiros",
            ),
            (
                7667,
                "Venda de combustível ou lubrificante a consumidor ou usuário final",
            ),
            (
                7900,
                "OUTRAS SAÍDAS DE MERCADORIAS OU PRESTAÇÕES DE SERVIÇOS",
            ),
            (
                7930,
                "Lançamento efetuado a título de devolução de bem cuja entrada tenha ocorrido sob amparo de regime especial aduaneiro de admissão temporária",
            ),
            (
                7949,
                "Outra saída de mercadoria ou prestação de serviço não especificado",
            ),
                ]
        .into_iter()
        .collect()
});

pub fn obter_descricao_do_cfop(cfop_opt: Option<u16>) -> String {
    cfop_opt
        .map(|cfop| match CFOP_DESCRICAO_RESUMIDA.get(&cfop) {
            Some(&descricao) => format!("{:04} - {}", cfop, descricao),
            None => String::new(),
        })
        .unwrap_or_default()
}

/// Verifica se um CFOP corresponde a uma operação de Importação.
/// Centraliza a regra "3000..=3999".
#[inline]
pub fn is_importacao(cfop: u16) -> bool {
    (3000..=3999).contains(&cfop)
}
