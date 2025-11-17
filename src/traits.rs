use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    ops::Deref,
    path::PathBuf,
    str::FromStr,
};

use chrono::NaiveDate;
use claudiofsr_lib::IteratorBack;
use rust_decimal::Decimal;

use crate::{
    AnaliseDosCreditos, ConsolidacaoCST, DELIMITER_CHAR, EFDError, EFDResult, SMALL_VALUE,
    structures::{analise_dos_creditos::Chaves, consolidacao_cst::Keys},
};

// --- Start: Definir traits para Ano, Mes, CST e CNPJBase ---
pub trait Ano {
    fn get_ano(&self) -> Option<i32>;
}

pub trait Mes {
    fn get_mes(&self) -> Option<u32>;
    fn set_mes(&mut self, m: Option<u32>);
}

pub trait Cst {
    fn get_cst(&self) -> Option<u16>;
}

pub trait CNPJBase {
    fn get_cnpj_base(&self) -> String;
}
// --- Final: Definir traits para Ano, Mes, CST e CNPJBase ---

// --- Start: Impl traits to Chaves ---
impl Ano for Chaves {
    fn get_ano(&self) -> Option<i32> {
        self.ano
    }
}

impl Mes for Chaves {
    fn get_mes(&self) -> Option<u32> {
        self.mes
    }
    fn set_mes(&mut self, m: Option<u32>) {
        self.mes = m;
    }
}

impl Cst for Chaves {
    fn get_cst(&self) -> Option<u16> {
        self.cst
    }
}

impl CNPJBase for Chaves {
    fn get_cnpj_base(&self) -> String {
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
    fn get_mes(&self) -> Option<u32> {
        self.mes
    }
    fn set_mes(&mut self, m: Option<u32>) {
        self.mes = m;
    }
}

impl Cst for Keys {
    fn get_cst(&self) -> Option<u16> {
        self.cst
    }
}

impl CNPJBase for Keys {
    fn get_cnpj_base(&self) -> String {
        self.cnpj_base.clone()
    }
}
// --- Final: Impl traits to keys ---

pub fn verificar_periodo_multiplo<T, U>(resultado: &HashMap<T, U>) -> bool
where
    T: Ano + Mes + Cst + CNPJBase,
{
    let mut cnpjs: HashSet<String> = HashSet::new();
    let mut hashmap: HashMap<String, HashSet<u32>> = HashMap::new();

    // encontrar todos os CNPJs distintos.
    for chave in resultado.keys() {
        cnpjs.insert(chave.get_cnpj_base());
    }

    for cnpj in cnpjs {
        let mut hashset: HashSet<u32> = HashSet::new();

        for chave in resultado.keys() {
            if cnpj != chave.get_cnpj_base() || chave.get_cst().is_none() {
                continue;
            }

            let ano = chave.get_ano(); // 2022
            let mes = chave.get_mes(); // 04
            let ano_mes = ano.and_then(|a: i32| Some((a as u32) * 100 + mes?)); // 202200 + 4 = 202204

            if !(mes >= Some(1) && mes <= Some(12)) {
                continue;
            }

            hashset.insert(ano_mes.unwrap());
        }

        hashmap.insert(cnpj, hashset);
    }

    let mut periodo_multiplo = false;

    for set in hashmap.values() {
        if set.len() > 1 {
            periodo_multiplo = true;
            break;
        }
    }

    //println!("hashmap: {hashmap:#?}");
    //println!("periodo_multiplo: {periodo_multiplo}");

    periodo_multiplo
}

/*
https://stackoverflow.com/questions/73680402/how-to-implement-iterator-for-array-optionf64-n-with-n-elements
https://practice.rs/generics-traits/const-generics.html
https://github.com/sunface/rust-by-practice/blob/master/solutions/generics-traits/const-generics.md
https://stackoverflow.com/questions/37410672/expected-type-parameter-found-u8-but-the-type-parameter-is-u8
https://saveriomiroddi.github.io/Rust-lulz-implementing_floating_point_approximate_equality_via_traits/

I have several structures with N (can be distinct) fields of type Option<f64>.
I want with a single function to evaluate whether or not I keep small values (values ​​< SMALL_VALUE = 0.005) ​​of these fields.
For this, I must implement an iterator that captures only the desired N fields from the structures.
*/

// --- AllValues --- //

/// Trait for types that have all their values as `Option<f64>`.
trait AllValues {
    /// Returns a vector of references to the `Option<f64>` values.
    fn get_all_values(&mut self) -> Vec<&mut Option<f64>>;
}

impl AllValues for ConsolidacaoCST {
    fn get_all_values(&mut self) -> Vec<&mut Option<f64>> {
        vec![
            &mut self.valor_item,
            &mut self.valor_bc,
            &mut self.valor_pis,
            &mut self.valor_cofins,
        ]
    }
}

impl AllValues for AnaliseDosCreditos {
    fn get_all_values(&mut self) -> Vec<&mut Option<f64>> {
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
    /// Sets all values in `self` to `None` if their absolute value is less than `SMALL_VALUE`.
    ///
    /// <https://stackoverflow.com/questions/73680402/how-to-implement-iterator-for-array-optionf64-n-with-n-elements>
    fn despise_small_values(&mut self);
}

impl<T: AllValues> Despise for T {
    fn despise_small_values(&mut self) {
        for value in self.get_all_values() {
            if let Some(v) = value
                && v.abs() < SMALL_VALUE
            {
                *value = None;
            }
        }
    }
}

/// A trait for splitting a string into individual fields using a delimiter.
pub trait SplitLine {
    /**
    Splits a line into individual fields using the delimiter.
    ```
        use efd_contribuicoes::SplitLine;
        use claudiofsr_lib::svec;

        let line = " | campo1| campo2 | ...... |campoN | ";
        let campos = line.split_line();
        // As a result, we will have:
        let result: Vec<String> = svec![
            "campo1",
            "campo2",
            "......",
            "campoN",
        ];
        assert_eq!(result, campos);
    ```
    */
    fn split_line(self) -> Vec<String>;
}

impl<T> SplitLine for T
where
    T: Deref<Target = str>,
{
    fn split_line(self) -> Vec<String> {
        self.split(DELIMITER_CHAR)
            .skip_last() // Skip the last element (empty string)
            .skip(1) // Skip the first element (empty string)
            .map(|campo| campo.trim().to_string())
            .collect()
    }
}

/// A trait to convert an `Option<T>` into an `EFDResult<Option<Decimal>>`.
/// Handles missing/empty fields and Decimal parsing errors.
pub trait ToDecimal {
    /// Converts `self` (an `Option<T>`) into `EFDResult<Option<Decimal>>`.
    ///
    /// # Returns
    /// - `Ok(None)` if `self` is `None` or contains an empty string.
    /// - `Ok(Some(Decimal))` if parsing is successful.
    /// - `Err(EFDError::ParseDecimalError)` if the string cannot be parsed as a Decimal.
    fn to_decimal(
        self,
        file_path: PathBuf,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<Decimal>>;
}

impl<T: ToString> ToDecimal for Option<T> {
    fn to_decimal(
        self,
        file_path: PathBuf,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<Decimal>> {
        self.map(|s| s.to_string()) // Convert Option<T> to Option<String>
            .filter(|s| !s.is_empty()) // Convert Some("") to None, treating empty as missing
            .map(|s| {
                // If Some(s) is still present, attempt parsing
                let s_parsed = s.replace('.', "").replace(',', "."); // SPED-specific parsing logic

                Decimal::from_str_exact(&s_parsed) // Attempt to parse into Decimal
                    .map_err(|source| {
                        // Map parsing error to EFDError::ParseDecimalError
                        EFDError::ParseDecimalError {
                            source,
                            valor_str: s, // Use the original string for error context
                            campo_nome: field_name.to_string(),
                            arquivo: file_path,
                            linha_num: line_number,
                        }
                    })
            }) // Result is now Option<Result<Decimal, EFDError>>
            .transpose() // Convert Option<Result<Decimal, EFDError>> to Result<Option<Decimal>, EFDError>
    }
}

/// A trait to convert an `Option<&&str>` to `EFDResult<Option<NaiveDate>>`.
/// Handles missing fields, empty fields, and date parsing errors.
pub trait ToOptionalNaiveDate {
    /// Converts the `Option<&&str>` into `Option<NaiveDate>`.
    ///
    /// - Returns `Ok(Some(NaiveDate))` if the string is a valid date.
    /// - Returns `Ok(None)` if the string is empty or the `Option` itself is `None`.
    /// - Returns `Err(EFDError::ParseDateError)` if the string is non-empty but contains an invalid date format.
    fn to_optional_date(
        self,
        file_path: PathBuf,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<NaiveDate>>;
}

impl<T: ToString> ToOptionalNaiveDate for Option<T> {
    fn to_optional_date(
        self,
        file_path: PathBuf,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<NaiveDate>> {
        self.map(|s| s.to_string()) // Option<T> -> Option<String>
            .filter(|s| s.len() == 6 || s.len() == 8)
            .map(|date_str| {
                let date_format = "%-d%-m%Y";
                if date_str.len() == 8 {
                    // Assuming DDMMYYYY format for SPED
                    NaiveDate::parse_from_str(&date_str, date_format)
                } else {
                    // Assuming MMYYYY format for SPED, assumed '01' for day
                    let day_month_year = format!("01{}", date_str);
                    NaiveDate::parse_from_str(&day_month_year, date_format)
                }
                .map_err(|source| EFDError::ParseDateError {
                    source,
                    data_str: date_str,
                    campo_nome: field_name.to_string(),
                    arquivo: file_path,
                    line_number,
                })
            })
            .transpose() // Convert Option<Result<T,E>> to Result<Option<T>,E>
    }
}

/// A trait to convert an `Option<&&str>` to `EFDResult<NaiveDate>`.
/// Designed for mandatory date fields, returning an error if the field is missing or empty.
pub trait ToNaiveDate {
    /// Converts the `Option<&&str>` into `NaiveDate`.
    ///
    /// - Returns `Ok(NaiveDate)` if the string is a valid date.
    /// - Returns `Err(EFDError::KeyNotFound)` if the string is empty or the `Option` itself is `None`.
    /// - Returns `Err(EFDError::ParseDateError)` if the string is non-empty but contains an invalid date format.
    fn to_date(
        self,
        file_path: PathBuf,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<NaiveDate>;
}

impl<T: ToString> ToNaiveDate for Option<T> {
    fn to_date(
        self,
        file_path: PathBuf,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<NaiveDate> {
        self.map(|s| s.to_string()) // Option<T> -> Option<String>
            .filter(|s| s.len() == 6 || s.len() == 8)
            .map(|date_str| {
                let date_format = "%-d%-m%Y";
                if date_str.len() == 8 {
                    // Assuming DDMMYYYY format for SPED
                    NaiveDate::parse_from_str(&date_str, date_format)
                } else {
                    // Assuming MMYYYY format for SPED, assumed '01' for day
                    let day_month_year = format!("01{}", date_str);
                    NaiveDate::parse_from_str(&day_month_year, date_format)
                }
                .map_err(|source| EFDError::ParseDateError {
                    source,
                    data_str: date_str,
                    campo_nome: field_name.to_string(),
                    arquivo: file_path,
                    line_number,
                })
            })
            .transpose()? // Convert Option<Result<T,E>> to Result<Option<T>,E>
            .ok_or_else(|| EFDError::KeyNotFound(field_name.to_string())) // If it's Ok(None), means it was empty/None
    }
}

/// Um trait para converter uma referência de string para um `Option<String>`.
/// Retorna `None` se a string, após `trim()`, tiver comprimento zero.
/// Caso contrário, retorna `Some(String)`.
pub trait ToOptionalString {
    /// Converte o `Option<&&str>` em `Option<String>`.
    ///
    /// # Retorna
    /// - `None` se o `Option` original for `None` ou se a string interna estiver vazia.
    /// - `Some(String)` contendo o valor da string (já trimado) se não estiver vazia.
    fn to_optional_string(self) -> Option<String>; // `self` por valor para consumir o Option
}

// Implementação para Option<&&str> (o tipo retornado por fields.get(index))
impl<T: ToString> ToOptionalString for Option<T> {
    fn to_optional_string(self) -> Option<String> {
        self.map(|s| s.to_string()) // Option<T> -> Option<String>
            .and_then(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            })
    }
}

/// A trait for converting an `Option` of a string-like type to an `EFDResult<Option<U>>`,
/// where `U` is an integer type.
/// Handles missing/empty fields and integer parsing errors.
pub trait ToOptionalInteger<U>
where
    U: FromStr + Debug, // U must be parsable from a string and debuggable
    <U as FromStr>::Err: Into<std::num::ParseIntError>,
{
    /// Converts `self` into `EFDResult<Option<U>>`.
    ///
    /// # Returns
    /// - `Ok(None)` if `self` is `None` or contains an empty string.
    /// - `Ok(Some(U))` if parsing is successful.
    /// - `Err(EFDError::ParseIntegerError)` if the string cannot be parsed into `U`.
    fn to_optional_integer(
        self,
        file_path: PathBuf,
        line_number: usize,
        field_name: &str,
    ) -> Result<Option<U>, EFDError>;
}

// Implement ToOptionalInteger<U> for Option<T> where T is string-like (e.g., &&str, String)
impl<T, U> ToOptionalInteger<U> for Option<T>
where
    T: ToString,                                        // T must be convertible to String
    U: FromStr + Debug, // U must be parsable from a string and debuggable
    <U as FromStr>::Err: Into<std::num::ParseIntError>, // U's parsing error must convert to ParseIntError
{
    fn to_optional_integer(
        self,
        file_path: PathBuf,
        line_number: usize,
        field_name: &str,
    ) -> Result<Option<U>, EFDError> {
        self.map(|s| s.to_string()) // Convert Option<T> to Option<String>
            .filter(|s| !s.is_empty()) // Filter out empty strings, converting Some("") to None
            .map(|s| {
                // If Some(s) is still present, attempt parsing
                s.parse::<U>() // Attempt to parse the String into U
                    .map_err(|source_err| EFDError::ParseIntegerError {
                        // Map parsing error to EFDError
                        source: source_err.into(), // Convert specific parse error to std::num::ParseIntError
                        data_str: s,               // Use the original string for error context
                        campo_nome: field_name.to_string(),
                        arquivo: file_path,
                        line_number,
                    })
            }) // Result is now Option<Result<U, EFDError>>
            .transpose() // Convert Option<Result<U, EFDError>> to Result<Option<U>, EFDError>
    }
}

/// Para converter um `Option<&&str>` em `EFDResult<Option<String>>` para o CNPJ.
/// Lida com campos ausentes, campos vazios e validação de comprimento do CNPJ,
/// retornando `EFDError::InvalidCNPJ` se o comprimento não for 14.
pub trait ToOptionalCnpj {
    /// Converte o `Option<&&str>` em `EFDResult<Option<String>>`.
    ///
    /// # Retorna
    /// - `Ok(None)` se o `Option` original for `None` ou se a string interna estiver vazia.
    /// - `Ok(Some(String))` se a string tiver comprimento 14.
    /// - `Err(EFDError::InvalidCNPJ)` se a string não estiver vazia, mas tiver um comprimento diferente de 14.
    fn to_optional_cnpj(
        self,
        file_path: PathBuf,
        line_number: usize,
        registro: &str,
        field_name: &str,
    ) -> EFDResult<Option<String>>;
}

// Implementação do trait ToOptionalCnpj para Option<&&str>
impl<T: ToString> ToOptionalCnpj for Option<T> {
    fn to_optional_cnpj(
        self,
        file_path: PathBuf,
        line_number: usize,
        registro: &str,
        field_name: &str,
    ) -> EFDResult<Option<String>> {
        self.map(|s| s.to_string()) // Option<&&str> -> Option<String>
            .filter(|s| !s.is_empty()) // Some("") -> None
            .map(|s| {
                // Agora `s` é garantido ser uma String não vazia, se presente
                if s.len() == 14 {
                    Ok(s) // Comprimento válido, envolve em Ok
                } else {
                    // Comprimento inválido, envolve erro em Err
                    Err(EFDError::InvalidCNPJ {
                        arquivo: file_path,
                        linha_num: line_number,
                        registro: registro.to_string(),
                        campo_nome: field_name.to_string(),
                        cnpj: s,
                    })
                }
            })
            .transpose() // Converte Option<Result<String, EFDError>> para Result<Option<String>, EFDError>
    }
}

/// Para converter um `Option<&&str>` em `EFDResult<String>` para o CNPJ.
pub trait ToCnpj {
    fn to_cnpj(
        self,
        file_path: PathBuf,
        line_number: usize,
        registro: &str,
        field_name: &str,
    ) -> EFDResult<String>;
}

impl<T: ToString> ToCnpj for Option<T> {
    fn to_cnpj(
        self,
        file_path: PathBuf,
        line_number: usize,
        registro: &str,
        field_name: &str,
    ) -> EFDResult<String> {
        self.map(|s| s.to_string()) // Option<T> -> Option<String>
            .filter(|s| !s.is_empty()) // Filter out empty strings, converting Some("") to None
            .ok_or_else(|| {
                // This converts the Option<String> into a Result<String, EFDError>.
                // Se for None aqui, significa que o campo estava ausente ou vazio.
                EFDError::KeyNotFound(field_name.to_string())
            }) // This converts the Option<String> into a Result<String, EFDError>.
            .and_then(|s| {
                // `s` here is the String from the Ok variant of the Result
                if s.len() == 14 {
                    Ok(s) // Valid length, wrap in Ok
                } else {
                    // Invalid length, wrap error in Err
                    Err(EFDError::InvalidCNPJ {
                        arquivo: file_path,
                        linha_num: line_number,
                        registro: registro.to_string(),
                        campo_nome: field_name.to_string(),
                        cnpj: s,
                    })
                }
            })
    }
}

/// This trait provides a common interface for all SPED record structs,
/// enabling polymorphism and allowing you to interact with different
/// records in a generic way (e.g., getting line_number, bloco, registro_name).
pub trait SpedRecordTrait: Debug + Send + Sync {
    fn nivel(&self) -> u16;
    fn bloco(&self) -> char;
    fn registro_name(&self) -> &str;
    fn line_number(&self) -> usize;
    fn as_any(&self) -> &dyn std::any::Any; // Adicionado para downcasting, se necessário
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//
//
// cargo test -- --help
// cargo test -- --nocapture
// cargo test -- --show-output

/// Run tests with:
/// cargo test -- --show-output trait_tests
#[cfg(test)]
mod trait_tests {
    use super::*;

    #[test]
    fn test_split_line() {
        let line = " | campo1| campo2 |campo3 | ";
        let campos = line.split_line();

        // Espera que o resultado seja a lista de campos esperada
        assert_eq!(campos, vec!["campo1", "campo2", "campo3"]);
    }

    #[test]
    fn test_split_line_empty() {
        let line1 = " foo ";
        let campos1 = line1.split_line();

        let line2 = " foo | bar ";
        let campos2 = line2.split_line();

        let line3 = " foo | campo nº1 | bar ";
        let campos3 = line3.split_line();

        // Espera que a lista de campos seja vazia
        assert!(campos1.is_empty());
        assert!(campos2.is_empty());
        assert_eq!(campos3, vec!["campo nº1"]);
    }

    #[test]
    fn test_split_line_with_empty_string() {
        let line = " | campo 1| | campo_2 | foo bar |campo 3 | ";
        let campos: Vec<String> = line.split_line();
        assert_eq!(campos, vec!["campo 1", "", "campo_2", "foo bar", "campo 3"]);
    }
}

/// Run tests with:
/// cargo test -- --show-output optional_integer_tests
#[cfg(test)]
mod optional_integer_tests {
    use super::*;

    // --- Novos testes para ToOptionalInteger ---

    #[test]
    fn to_optional_integer_success_u64() -> EFDResult<()> {
        let path = PathBuf::from("test_file.txt");
        let line = 1;
        let field = "ID";

        let input: Option<&str> = Some("12345");
        let result: Option<u64> = input.to_optional_integer(path.clone(), line, field)?;
        assert_eq!(result, Some(12345u64));
        Ok(())
    }

    #[test]
    fn to_optional_integer_success_i32() -> EFDResult<()> {
        let path = PathBuf::from("test_file.txt");
        let line = 2;
        let field = "Valor";

        let input: Option<&&str> = Some(&"-500");
        let result: Option<i32> = input.to_optional_integer(path.clone(), line, field)?;
        assert_eq!(result, Some(-500i32));
        Ok(())
    }

    #[test]
    fn to_optional_integer_empty_string_returns_none() -> EFDResult<()> {
        let path = PathBuf::from("test_file.txt");
        let line = 3;
        let field = "Quantidade";

        let input: Option<&&str> = Some(&"");
        let result: Option<u16> = input.to_optional_integer(path.clone(), line, field)?;
        assert!(result.is_none());
        Ok(())
    }

    #[test]
    fn to_optional_integer_none_input_returns_none() -> EFDResult<()> {
        let path = PathBuf::from("test_file.txt");
        let line = 4;
        let field = "Código";

        let input: Option<&&str> = None;
        let result: Option<usize> = input.to_optional_integer(path.clone(), line, field)?;
        assert!(result.is_none());
        Ok(())
    }

    #[test]
    fn to_optional_integer_invalid_string_returns_error() {
        let path = PathBuf::from("test_file.txt");
        let line = 5;
        let field = "Preço";

        let input: Option<&&str> = Some(&"abc");
        let result: Result<Option<u32>, EFDError> =
            input.to_optional_integer(path.clone(), line, field);

        assert!(result.is_err());
        if let Err(EFDError::ParseIntegerError {
            data_str,
            campo_nome,
            arquivo,
            line_number,
            ..
        }) = result
        {
            assert_eq!(data_str, "abc");
            assert_eq!(campo_nome, field);
            assert_eq!(arquivo, path);
            assert_eq!(line_number, line);
        } else {
            panic!("Expected ParseIntError but got {:?}", result);
        }
    }

    #[test]
    /// cargo test -- --show-output integer_overflow
    fn to_optional_integer_overflow_returns_error() {
        let path = PathBuf::from("test_file.txt");
        let line = 6;
        let field = "PequenoID";

        let input: Option<&&str> = Some(&"256"); // Max u8 is 255
        let result: Result<Option<u8>, EFDError> =
            input.to_optional_integer(path.clone(), line, field);

        assert!(result.is_err());
        if let Err(EFDError::ParseIntegerError {
            data_str,
            campo_nome,
            arquivo,
            line_number,
            source,
        }) = result
        {
            assert_eq!(data_str, "256");
            assert_eq!(campo_nome, field);
            assert_eq!(arquivo, path);
            assert_eq!(line_number, line);
            // Verifica o tipo de erro de parse subjacente, se possível
            assert_eq!(source.kind(), &std::num::IntErrorKind::PosOverflow);
        } else {
            panic!("Expected ParseIntError but got {:?}", result);
        }
    }
}
