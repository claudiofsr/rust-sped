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

/// Um trait para converter um `Option<&&str>` para `EFDResult<Option<Decimal>>`.
/// Lida com campos ausentes, campos vazios e erros de parsing de Decimal.
pub trait ToDecimal {
    /// Converte o `Option<&&str>` em `EFDResult<Option<Decimal>>`.
    ///
    /// # Retorna
    /// - `Ok(None)` se o `Option` original for `None` ou se a string interna (após `trim()`) estiver vazia.
    /// - `Ok(Some(Decimal))` se o parsing for bem-sucedido.
    /// - `Err(EFDError::ParseDecimalError)` se a string não puder ser parseada como Decimal.
    fn to_decimal(
        self,               // self agora é por valor
        file_path: PathBuf, // Mantém PathBuf aqui para o contexto de erro
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<Decimal>>;
}

// Implementação para Option<&&str>
impl ToDecimal for Option<&&str> {
    // <--- Implementação para Option<&&str>
    fn to_decimal(
        self,
        file_path: PathBuf,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<Decimal>> {
        match self {
            None => Ok(None), // Se o Option original for None (campo ausente), retorna Ok(None)
            Some(s) => {
                if s.is_empty() {
                    return Ok(None); // Se a string estiver vazia, retorna Ok(None)
                }

                let s_parsed = s.replace('.', "").replace(',', "."); // Lógica de parsing do SPED

                Decimal::from_str_exact(&s_parsed)
                    .map(Some) // Se Ok(decimal), transforma em Ok(Some(decimal))
                    .map_err(|source| EFDError::ParseDecimalError {
                        // Se Err(parse_err), transforma em Err(EFDError)
                        source,
                        valor_str: s.to_string(), // Usa o valor trimado para o erro
                        campo_nome: field_name.to_string(),
                        arquivo: file_path,
                        linha_num: line_number,
                    })
            }
        }
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

impl ToOptionalNaiveDate for Option<&&str> {
    fn to_optional_date(
        self,
        file_path: PathBuf,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<NaiveDate>> {
        let original_s = self.map(|s| (*s).to_string());

        self.filter(|s| !s.is_empty())
            .map(|s| NaiveDate::parse_from_str(s, "%-d%-m%Y")) // Assuming DDMMYYYY format for SPED
            .transpose() // Convert Option<Result<T,E>> to Result<Option<T>,E>
            .map_err(|source| EFDError::ParseDateError {
                source,
                data_str: original_s.unwrap_or_default(),
                campo_nome: field_name.to_string(),
                arquivo: file_path,
                line_number,
            })
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

impl ToNaiveDate for Option<&&str> {
    fn to_date(
        self,
        file_path: PathBuf,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<NaiveDate> {
        let original_s = self.map(|s| (*s).to_string());

        self.filter(|s| !s.is_empty()) // Filter out empty strings first
            .map(|s| NaiveDate::parse_from_str(s, "%-d%-m%Y")) // Attempt to parse
            .transpose() // Convert Option<Result<T,E>> to Result<Option<T>,E>
            .map_err(|source| EFDError::ParseDateError {
                // Map parsing errors
                source,
                data_str: original_s.unwrap_or_default(),
                campo_nome: field_name.to_string(),
                arquivo: file_path.to_path_buf(), // Clonar PathBuf aqui
                line_number,
            })? // Propagate ParseDateError immediately if parsing fails
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
impl ToOptionalString for Option<&&str> {
    fn to_optional_string(self) -> Option<String> {
        self.and_then(|s| {
            if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        })
    }
}

/// Trait para converter um `Option<&&str>` em `Result<Option<T>>`, onde `T` é um tipo inteiro.
///
/// Este trait é genérico para qualquer tipo `T` que possa ser parseado de uma string
/// e cujo erro de parse possa ser convertido para `std::num::ParseIntError`.
pub trait ToOptionalInteger<T>
where
    T: FromStr + Debug, // T deve ser capaz de ser parseado de uma string e ser debugável
    <T as FromStr>::Err: Into<std::num::ParseIntError>,
{
    /// Analisa o slice de string em um `Option<T>`.
    ///
    /// - Retorna `Ok(Some(T))` se a string for um número válido.
    /// - Retorna `Ok(None)` se a string for vazia ou o `Option` em si for `None`.
    /// - Retorna `Err(EFDError::ParseIntError)` se a string for não vazia com `T` inválido.
    fn to_optional_integer(
        self,
        file_path: PathBuf,
        line_number: usize,
        field_name: &str,
    ) -> Result<Option<T>, EFDError>;
}

// Implementação do trait para `Option<&&str>` para qualquer tipo `T` que atenda às restrições.
impl<T> ToOptionalInteger<T> for Option<&&str>
where
    T: FromStr + Debug,
    <T as FromStr>::Err: Into<std::num::ParseIntError>,
{
    fn to_optional_integer(
        self,
        file_path: PathBuf,
        line_number: usize,
        field_name: &str,
    ) -> Result<Option<T>, EFDError> {
        // Captura a string original para relatórios de erro detalhados, se necessário.
        let original_s = self.map(|s| (*s).to_string());

        self.filter(|s| !s.is_empty()) // Converte Some("") em None
            .map(|s| s.parse::<T>()) // Tenta parsear a string para T, resultando em Option<Result<T, Err>>
            .transpose() // Converte Option<Result<T, Err>> para Result<Option<T>, Err>
            .map_err(|source_err| EFDError::ParseIntegerError {
                // Mapeia o erro de parse para o seu EFDError
                source: source_err.into(), // Converte o erro específico de parse para std::num::ParseIntError
                data_str: original_s.unwrap_or_default(), // Usa a string original para contexto de erro
                campo_nome: field_name.to_string(),
                arquivo: file_path,
                line_number,
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

        let input: Option<&&str> = Some(&"12345");
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
