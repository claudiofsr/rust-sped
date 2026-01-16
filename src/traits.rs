use std::{
    any::Any,
    collections::{HashMap, HashSet},
    fmt::Debug,
    path::Path,
    str::FromStr,
    sync::Arc,
};

use chrono::NaiveDate;
use claudiofsr_lib::{FormatStyle, IteratorBack, StrExtension, thousands_separator};
use compact_str::CompactString;
use rust_decimal::Decimal;

use crate::{
    AnaliseDosCreditos, CodigoSituacaoTributaria, ConsolidacaoCST, DELIMITER_CHAR, EFDError,
    EFDResult, MesesDoAno, PRECISAO_FLOAT, SMALL_VALUE,
    structures::{analise_dos_creditos::Chaves, consolidacao_cst::Keys},
};

// --- Start: Definir traits para Ano, Mes, CST e CNPJBase ---
pub trait Ano {
    fn get_ano(&self) -> Option<i32>;
}

pub trait Mes {
    fn get_mes(&self) -> Option<MesesDoAno>;
    fn set_mes(&mut self, m: Option<MesesDoAno>);
    /// Alterar mês para MesesDoAno::Soma (índice 13)
    fn set_mes_para_soma(&mut self);
    fn is_soma(&self) -> bool;
}

pub trait Cst {
    fn get_cst(&self) -> Option<CodigoSituacaoTributaria>;
}

pub trait CNPJBase {
    fn get_cnpj_base(&self) -> CompactString;
}
// --- Final: Definir traits para Ano, Mes, CST e CNPJBase ---

// --- Start: Impl traits to Chaves ---
impl Ano for Chaves {
    fn get_ano(&self) -> Option<i32> {
        self.ano
    }
}

impl Mes for Chaves {
    fn get_mes(&self) -> Option<MesesDoAno> {
        self.mes
    }
    fn set_mes(&mut self, m: Option<MesesDoAno>) {
        self.mes = m;
    }
    fn set_mes_para_soma(&mut self) {
        // Define o mês como o enum "Soma" (Item 13)
        self.mes = Some(MesesDoAno::Soma);
    }
    fn is_soma(&self) -> bool {
        matches!(self.mes, Some(MesesDoAno::Soma))
    }
}

impl Cst for Chaves {
    fn get_cst(&self) -> Option<CodigoSituacaoTributaria> {
        self.cst
    }
}

impl CNPJBase for Chaves {
    fn get_cnpj_base(&self) -> CompactString {
        self.cnpj_base.clone()
    }
}
// --- Final: Impl traits to Chaves ---

// --- Start: Impl traits to keys ---
impl Ano for Keys {
    fn get_ano(&self) -> Option<i32> {
        self.ano
    }
}

impl Mes for Keys {
    fn get_mes(&self) -> Option<MesesDoAno> {
        self.mes
    }
    fn set_mes(&mut self, m: Option<MesesDoAno>) {
        self.mes = m;
    }
    fn set_mes_para_soma(&mut self) {
        // Define o mês como o enum "Soma" (Item 13)
        self.mes = Some(MesesDoAno::Soma);
    }
    fn is_soma(&self) -> bool {
        matches!(self.mes, Some(MesesDoAno::Soma))
    }
}

impl Cst for Keys {
    fn get_cst(&self) -> Option<CodigoSituacaoTributaria> {
        self.cst
    }
}

impl CNPJBase for Keys {
    fn get_cnpj_base(&self) -> CompactString {
        self.cnpj_base.clone()
    }
}
// --- Final: Impl traits to keys ---

pub fn verificar_periodo_multiplo<T, U>(resultado: &HashMap<T, U>) -> bool
where
    T: Ano + Mes + Cst + CNPJBase,
{
    // Mapa: CNPJ -> Conjunto de Períodos (AnoMes)
    let mut map_cnpj_periodos: HashMap<CompactString, HashSet<u32>> = HashMap::new();

    for chave in resultado.keys() {
        // Ignora chaves sem CST definido
        if chave.get_cst().is_none() {
            continue;
        }

        // Valida Ano e Mes
        if let (Some(ano), Some(mes)) = (chave.get_ano(), chave.get_mes()) {
            let ano_mes = (ano as u32) * 100 + (mes as u32);

            // 1. Obtém (ou cria) o HashSet para este CNPJ
            let periodos = map_cnpj_periodos.entry(chave.get_cnpj_base()).or_default();

            // 2. Insere o novo período (simplificação solicitada)
            periodos.insert(ano_mes);

            // 3. Otimização "Fail Fast":
            // Se detectarmos mais de 1 período para este CNPJ, paramos imediatamente.
            // Não é necessário continuar processando o resto do arquivo.
            if periodos.len() > 1 {
                return true;
            }
        }
    }

    // Se percorreu tudo e nenhum CNPJ teve > 1 período
    false
}

// --- AllValues --- //

trait AllValues {
    fn get_all_values(&mut self) -> Vec<&mut Decimal>;
}

impl AllValues for ConsolidacaoCST {
    fn get_all_values(&mut self) -> Vec<&mut Decimal> {
        vec![
            &mut self.valor_item,
            &mut self.valor_bc,
            &mut self.valor_pis,
            &mut self.valor_cofins,
        ]
    }
}

impl AllValues for AnaliseDosCreditos {
    fn get_all_values(&mut self) -> Vec<&mut Decimal> {
        vec![
            &mut self.valor_bc,
            &mut self.valor_rbnc_trib,
            &mut self.valor_rbnc_ntrib,
            &mut self.valor_rbnc_exp,
            &mut self.valor_rb_cum,
        ]
    }
}

/// Despise small values
pub trait Despise {
    fn despise_small_values(&mut self);
}

impl<T: AllValues> Despise for T {
    fn despise_small_values(&mut self) {
        for value in self.get_all_values() {
            if value.abs() < SMALL_VALUE {
                *value = Decimal::ZERO
            }
        }
    }
}

/// Extensão para facilitar comparações seguras com números de ponto flutuante (`f64`).
///
/// Em computação, `0.1 + 0.2 != 0.3` devido à precisão binária.
/// Portanto, nunca devemos usar `==` ou `>` diretamente para valores monetários em `f64`.
pub trait FloatExt {
    fn eh_zero(self) -> bool;
    fn eh_igual(self, other: f64) -> bool;
    fn eh_maior_que_zero(self) -> bool;
}

impl FloatExt for f64 {
    /// Verifica se o valor é virtualmente zero.
    ///
    /// Retorna `true` se o valor absoluto for menor que a tolerância de erro.
    #[inline]
    fn eh_zero(self) -> bool {
        self.abs() < PRECISAO_FLOAT
    }

    /// Verifica a igualdade entre dois floats considerando a margem de erro.
    #[inline]
    fn eh_igual(self, other: f64) -> bool {
        (self - other).abs() < PRECISAO_FLOAT
    }

    /// Verifica se o valor é positivo e significativo.
    ///
    /// Retorna `true` apenas se o número for maior que a tolerância (ex: 0.00000001).
    /// Valores extremamente pequenos (ruído numérico) são tratados como zero/falso.
    #[inline]
    fn eh_maior_que_zero(self) -> bool {
        self > PRECISAO_FLOAT
    }
}

// ============================================================================
// Decimal Extension
// ============================================================================

/// Trait de extensão: Adiciona métodos utilitários diretamente aos
/// tipos `Decimal` e `Option<Decimal>`.
pub trait DecimalExt {
    /// Verifica se o valor é estritamente positivo (maior que zero).
    fn eh_maior_que_zero(&self) -> bool;

    /// Verifica se o valor é nulo ou igual a zero.
    fn eh_zero(&self) -> bool;

    /// Verifica a igualdade exata com outro valor `Decimal`.
    fn eh_igual(&self, other: Decimal) -> bool;

    /// Formata o valor como String no padrão brasileiro (PtBr), com separadores de milhar.
    ///
    /// No arredondamento de `Decimal`, usa a estatégia `RoundingStrategy::MidpointNearestEven`.
    fn to_formatted_string(&self, decimals: usize) -> String;
}

impl DecimalExt for Decimal {
    fn eh_maior_que_zero(&self) -> bool {
        *self > Decimal::ZERO
    }

    fn eh_zero(&self) -> bool {
        self.is_zero()
    }

    fn eh_igual(&self, other: Decimal) -> bool {
        *self == other
    }

    fn to_formatted_string(&self, decimals: usize) -> String {
        thousands_separator(self, decimals, FormatStyle::PtBr)
    }
}

// Implementação para Option<Decimal> para facilitar chamadas diretas
impl DecimalExt for Option<Decimal> {
    fn eh_maior_que_zero(&self) -> bool {
        match self {
            Some(d) => d.eh_maior_que_zero(),
            None => false,
        }
    }

    fn eh_zero(&self) -> bool {
        match self {
            Some(d) => d.is_zero(),
            None => false,
        }
    }

    fn eh_igual(&self, other: Decimal) -> bool {
        match self {
            Some(d) => d.eh_igual(other),
            None => false,
        }
    }

    fn to_formatted_string(&self, decimals: usize) -> String {
        match self {
            Some(d) => d.to_formatted_string(decimals),
            None => String::new(),
        }
    }
}

// ============================================================================
// SEÇÃO 1: EXTENSIONS E UTILITÁRIOS
// Conversões seguras e funcionais para tipos primitivos e Options
// ============================================================================

/// Extension para facilitar o parsing de `Option<T>` para `Option<U>`.
///
/// U pode ser String ou &str.
pub trait StringParser {
    /// Parse `Option<T>` para `Option<U>`.
    ///
    /// Tenta realizar o parse para um tipo `U` (ex: u32, f64, Decimal).
    /// Retorna `None` se a entrada for `None`, vazia ou se o parse falhar.
    fn parse_opt<U: FromStr>(&self) -> Option<U>;

    /// Converte `Option<T>` para `Option<Arc<str>>`.
    ///
    /// Retorna `None` se a entrada for `None` ou string vazia (Economia de RAM).
    fn to_arc(&self) -> Option<Arc<str>>;

    /// Helper para converter string para Arc<str> uppercase de forma eficiente;.
    ///
    /// Só aloca nova string se houver alguma letra minúscula.
    /// - "NOTA 123" -> Retorna Arc(original) (Zero Copy)
    /// - "Nota 123" -> Retorna Arc("NOTA 123") (Alocação necessária)
    fn to_upper_arc(&self) -> Option<Arc<str>>;
}

impl<T> StringParser for Option<T>
where
    T: AsRef<str>,
{
    fn parse_opt<U: FromStr>(&self) -> Option<U> {
        self.as_ref()
            .map(|t| t.as_ref()) // 1. Obtém o &str (Zero Copy)
            .filter(|s| !s.is_empty()) // 2. "Fail fast": Se vazio, nem tenta parsear
            .and_then(|s| s.parse().ok())
    }

    fn to_arc(&self) -> Option<Arc<str>> {
        self.as_ref()
            .map(|t| t.as_ref()) // 1. Obtém o &str (Zero Copy)
            .filter(|s| !s.is_empty()) // 2. CRUCIAL: Transforma Some("") em None
            .map(|s| s.replace_multiple_whitespaces())
            .map(Arc::from) // 3. Só aloca na Heap se tiver conteúdo real
    }

    fn to_upper_arc(&self) -> Option<Arc<str>> {
        self.as_ref()
            .map(|t| t.as_ref().trim()) // 1. Obtém o &str de T
            .filter(|s| !s.is_empty()) // 2. Filtra vazios (consistência com to_arc)
            .map(|s| s.replace_multiple_whitespaces())
            .map(|s| {
                // Verifica se há minúsculas percorrendo os caracteres
                if s.chars().any(|c| c.is_lowercase()) {
                    // Caminho Lento: Aloca String (Heap) -> Move para Arc (Heap)
                    // Arc::from(String) é eficiente pois rouba o buffer da String se possível
                    Arc::from(s.to_uppercase())
                } else {
                    // Caminho Rápido: Copia bytes do &str direto para o Arc
                    // Evita a alocação intermediária de uma String desnecessária
                    Arc::from(s)
                }
            })
    }
}

/// A trait for splitting a string into individual fields using a delimiter.
pub trait SplitLine {
    fn split_line(&self) -> Vec<String>;
}

// Alterado para aceitar referências via AsRef, evitando clonar se não necessário
// Porém, como o retorno é Vec<String>, a alocação é inevitável no final.
impl<T> SplitLine for T
where
    T: AsRef<str>,
{
    fn split_line(&self) -> Vec<String> {
        self.as_ref()
            .split(DELIMITER_CHAR)
            .skip_last() // Skip the last element (empty string)
            .skip(1) // Skip the first element (empty string)
            .map(|campo| campo.trim().to_string())
            .collect()
    }
}

pub trait SpedRecordTrait: Debug + Any + Send + Sync {
    fn nivel(&self) -> u16;
    fn bloco(&self) -> char;
    fn registro_name(&self) -> &str;
    fn line_number(&self) -> usize;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// A trait to convert an `Option<T>` into an `EFDResult<Option<Decimal>>`.
pub trait ToDecimal {
    fn to_decimal(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<Decimal>>;
}

// Otimizado: T: AsRef<str> evita .to_string() imediato
impl<T: AsRef<str>> ToDecimal for Option<T> {
    fn to_decimal(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<Decimal>> {
        self.as_ref() // Option<T> -> Option<&T>
            .map(|s| s.as_ref().trim()) // Option<&T> -> Option<&str>
            .filter(|s| !s.is_empty())
            .map(|s| {
                // Decimal requer troca de vírgula por ponto (custo de alocação)
                // Se RustDecimal suportasse locale pt-BR nativamente, evitaríamos essa string.
                let s_parsed = s.replace('.', "").replace(',', ".");

                Decimal::from_str_exact(&s_parsed).map_err(|source| EFDError::ParseDecimalError {
                    source,
                    valor_str: s.to_string(), // Aloca apenas no erro para contexto
                    campo_nome: field_name.to_string(),
                    arquivo: file_path.to_path_buf(),
                    linha_num: line_number,
                })
            })
            .transpose()
    }
}

/// A trait to convert an `Option<T>` to `EFDResult<Option<NaiveDate>>`.
pub trait ToNaiveDate {
    fn to_optional_date(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<NaiveDate>>;

    fn to_date(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<NaiveDate>;
}

// Otimizado: Zero allocation no caminho feliz para datas completas (8 chars)
impl<T: AsRef<str>> ToNaiveDate for Option<T> {
    fn to_optional_date(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<NaiveDate>> {
        self.as_ref()
            .map(|s| s.as_ref().trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                let date_format = "%-d%-m%Y";
                let result = if s.len() == 8 {
                    // Formato DDMMYYYY: Parse direto do slice (Zero Copy)
                    NaiveDate::parse_from_str(s, date_format)
                } else {
                    // Formato MMYYYY: Requer alocação para prefixar '01'
                    let day_month_year = format!("01{}", s);
                    NaiveDate::parse_from_str(&day_month_year, date_format)
                };

                result.map_err(|source| EFDError::ParseDateError {
                    source,
                    data_str: s.to_string(),
                    campo_nome: field_name.to_string(),
                    arquivo: file_path.to_path_buf(),
                    line_number,
                })
            })
            .transpose()
    }

    fn to_date(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<NaiveDate> {
        self.to_optional_date(file_path, line_number, field_name)?
            .ok_or_else(|| EFDError::KeyNotFound(field_name.to_string()))
    }
}

/// Trait para converter `Option<T>` em `Option<String>` (trimada e não vazia).
pub trait ToOptionalString {
    fn to_optional_string(&self) -> Option<String>;
}

impl<T: AsRef<str>> ToOptionalString for Option<T> {
    fn to_optional_string(&self) -> Option<String> {
        self.as_ref()
            .map(|s| s.as_ref().trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()) // Aloca apenas se não for vazio
    }
}

/// Extension trait for parsing optional string-like values into integers.
pub trait ToOptionalInteger {
    /// Parses the inner value into generic type `U`.
    fn to_optional_integer<U>(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> Result<Option<U>, EFDError>
    where
        U: FromStr + Debug,
        <U as FromStr>::Err: Into<std::num::ParseIntError>;
}

// Blanket implementation for any Option containing a string-like type.
// High performance: Works on &str, allocates only on error.
impl<T> ToOptionalInteger for Option<T>
where
    T: AsRef<str>,
{
    fn to_optional_integer<U>(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> Result<Option<U>, EFDError>
    where
        U: FromStr + Debug,
        <U as FromStr>::Err: Into<std::num::ParseIntError>,
    {
        self.as_ref()
            .map(|s| s.as_ref().trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                s.parse::<U>().map_err(|e| EFDError::ParseIntegerError {
                    source: e.into(),
                    data_str: s.to_string(), // Allocates only on error
                    campo_nome: field_name.to_string(),
                    arquivo: file_path.to_path_buf(),
                    line_number,
                })
            })
            .transpose()
    }
}

/// Trait para validação e conversão de CNPJ.
pub trait ToCNPJ {
    fn to_optional_cnpj(
        &self,
        file_path: &Path,
        line_number: usize,
        registro: &str,
        field_name: &str,
    ) -> EFDResult<Option<String>>;

    fn to_cnpj(
        &self,
        file_path: &Path,
        line_number: usize,
        registro: &str,
        field_name: &str,
    ) -> EFDResult<String>;
}

impl<T: AsRef<str>> ToCNPJ for Option<T> {
    fn to_optional_cnpj(
        &self,
        file_path: &Path,
        line_number: usize,
        registro: &str,
        field_name: &str,
    ) -> EFDResult<Option<String>> {
        self.as_ref()
            .map(|s| s.as_ref().trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                // Validação feita no slice, alocação apenas no sucesso ou erro.
                if s.len() == 14 {
                    Ok(s.to_string())
                } else {
                    Err(EFDError::InvalidCNPJ {
                        arquivo: file_path.to_path_buf(),
                        linha_num: line_number,
                        registro: registro.to_string(),
                        campo_nome: field_name.to_string(),
                        cnpj: s.to_string(),
                    })
                }
            })
            .transpose()
    }

    fn to_cnpj(
        &self,
        file_path: &Path,
        line_number: usize,
        registro: &str,
        field_name: &str,
    ) -> EFDResult<String> {
        self.to_optional_cnpj(file_path, line_number, registro, field_name)?
            .ok_or_else(|| EFDError::KeyNotFound(field_name.to_string()))
    }
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//
//
// cargo test -- --help
// cargo test -- --nocapture
// cargo test -- --show-output

/// Run tests with:
/// cargo test -- --show-output traits_tests
#[cfg(test)]
#[path = "tests/traits_tests.rs"]
mod traits_tests;
