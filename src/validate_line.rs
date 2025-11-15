use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;
use std::{collections::HashMap, io::Read, ops::Deref, path::Path, str};

use crate::{EFDError, EFDResult, SplitLine, Tipo, sped_efd};

/// Represents a processed and validated line from the EFD file.
/// Contains the original line number and its parsed fields.
#[derive(Debug)]
pub struct ValidatedLine {
    pub line_number: usize,
    pub fields: Vec<String>,
}

impl ValidatedLine {
    /// Attempts to create a `ValidatedLine` from raw byte data.
    ///
    /// This function handles:
    /// 1. I/O errors during byte retrieval.
    /// 2. UTF-8 decoding, falling back to WINDOWS-1252 if necessary.
    /// 3. Initial filtering for lines with insufficient fields.
    /// 4. Standardizing the record identifier (e.g., "d101" -> "D101").
    /// 5. Validating if the record type is defined in the EFD schema.
    ///
    /// Lines that are ignorable (e.g., too short, unrecognized record type)
    /// will result in `Ok(None)`. Critical errors will be propagated as `Err(EFDError)`.
    ///
    /// # Arguments
    /// * `line_bytes_result` - Result of reading raw bytes for a line.
    /// * `line_number` - The 1-based line number in the file.
    /// * `arquivo` - The path to the EFD file being processed.
    /// * `registros_efd` - A map defining valid EFD record structures.
    ///
    /// # Returns
    /// An `EFDResult<Option<Self>>`:
    /// * `Ok(Some(ValidatedLine))`: If the line was successfully parsed and validated.
    /// * `Ok(None)`: If the line was ignorable (e.g., empty, unrecognized record type).
    /// * `Err(EFDError)`: If a critical error occurred (e.g., unrecoverable decoding error).
    pub fn try_from_raw_bytes(
        line_bytes_result: std::io::Result<Vec<u8>>,
        line_number: usize,
        arquivo: &Path,
        registros_efd: &HashMap<&'static str, HashMap<u16, (&'static str, Tipo)>>,
    ) -> EFDResult<Option<Self>> {
        let line_bytes = line_bytes_result?; // Propagate I/O error

        // Trim ASCII whitespace from both ends of the byte slice.
        let trimmed_bytes = line_bytes.trim_ascii();

        // Skip empty lines after trimming.
        if trimmed_bytes.is_empty() {
            return Ok(None);
        }

        // Decode bytes to string, handling potential encoding issues.
        let line_string = get_string_utf8(trimmed_bytes, line_number, arquivo)?;

        let mut fields = line_string.split_line();

        // Filtering rule 1: Line must have at least two fields (record identifier and at least one data field).
        if fields.len() < 2 {
            // Log or count ignored lines if necessary for debugging.
            return Ok(None);
        }

        // Standardize the record identifier (e.g., "d101" -> "D101", handle old M210/M610).
        padronizar_registro(&mut fields);

        // Filtering rule 2: Validate if the record type exists in the predefined schema.
        // This will return an `EFDError::UndefinedRecord` if the record is unknown,
        // otherwise `Ok(true)`.
        if !is_record_valid(registros_efd, &fields, &line_number, arquivo)? {
            // `is_record_valid` now propagates `UndefinedRecord` as an error.
            // If it returns `Ok(false)` (which it doesn't anymore with the current logic),
            // it would be for genuinely ignorable but defined records.
            // For now, this branch is effectively unreachable if `is_record_valid` returns Err for undefined.
            return Ok(None);
        }

        // If all checks pass, return a successfully validated line.
        Ok(Some(ValidatedLine {
            line_number,
            fields,
        }))
    }
}

/// Converts a slice of bytes to a `String`, attempting to handle different encodings.
///
/// It first tries to decode the bytes as UTF-8. If that fails, it attempts to decode
/// them using WINDOWS-1252 encoding. If both fail, it returns an `EFDError::Utf8DecodeError`.
///
/// # Arguments
/// * `slice_bytes` - A slice of bytes to convert.
/// * `line_number` - The line number (for error reporting).
/// * `path` - The path to the file (for error reporting).
///
/// # Returns
/// A `Result` containing the decoded `String` if successful, or an `EFDError` if decoding fails.
pub fn get_string_utf8(slice_bytes: &[u8], line_number: usize, path: &Path) -> EFDResult<String> {
    // Attempt to decode as UTF-8 first, which is the preferred and standard encoding.
    match str::from_utf8(slice_bytes) {
        Ok(s) => Ok(s.to_string()),
        Err(utf8_error) => {
            // If UTF-8 decoding fails, attempt WINDOWS-1252 decoding as a common fallback for Brazilian EFD files.
            let mut decoder = DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1252))
                .build(slice_bytes);

            let mut buffer = String::new();
            match decoder.read_to_string(&mut buffer) {
                Ok(_) => Ok(buffer),
                Err(io_error_context) => Err(EFDError::Utf8DecodeError(
                    path.to_path_buf(),
                    line_number,
                    utf8_error,
                    io_error_context,
                )),
            }
        }
    }
}

/// Standardizes the record identifier (first field) in place.
///
/// Converts the record to uppercase and appends "_antigo" for specific
/// historical records (M210, M610) if they match a specific length.
///
/// # Arguments
/// * `fields` - A mutable slice of strings representing the fields of a line.
pub fn padronizar_registro(fields: &mut [String]) {
    let fields_len = fields.len();

    let Some(first_field) = fields.first_mut() else {
        return; // No fields to process.
    };

    first_field.make_ascii_uppercase(); // Convert in place.

    // Check for "old" record types based on identifier and field count.
    // Substituir registro M210 por M210_antigo ou M610 por M610_antigo,
    // se campos.len() == 13
    if sped_efd::registros_antigos(first_field, fields_len) {
        first_field.push_str("_antigo");
    }
}

/// Checks if a given record type is defined in the EFD schema.
///
/// Returns `Ok(true)` if the record is found.
/// Returns an `Err(EFDError::UndefinedRecord)` if the record is not found,
/// providing detailed context for debugging.
///
/// # Arguments
/// * `registros_efd` - A map defining valid EFD record structures.
/// * `fields` - The parsed fields of the line, where the first field is the record type.
/// * `num_line` - The 1-based line number.
/// * `arquivo` - The path to the EFD file.
///
/// # Returns
/// An `EFDResult<bool>`: `Ok(true)` if valid, `Err(EFDError::UndefinedRecord)` if undefined.
fn is_record_valid<T>(
    registros_efd: &HashMap<&'static str, HashMap<u16, (&'static str, Tipo)>>,
    fields: &[T],
    num_line: &usize,
    arquivo: &Path,
) -> EFDResult<bool>
where
    T: Deref<Target = str> + std::fmt::Debug,
{
    let record_type: &str = fields[0].deref();
    if registros_efd.contains_key(record_type) {
        Ok(true)
    } else {
        Err(EFDError::UndefinedRecord {
            record: record_type.to_string(),
            arquivo: arquivo.to_path_buf(),
            linha_num: *num_line,
            fields: fields.iter().map(|f| f.deref().to_string()).collect(),
        })
    }
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

/// Run tests with:
/// cargo test -- --show-output validade_line_tests
#[cfg(test)]
mod validade_line_tests {
    use super::*;
    use crate::{Tipo, obter_valores};
    use std::{io::ErrorKind, path::PathBuf};

    #[test]
    fn analisar_linhas() -> EFDResult<()> {
        // cargo test -- --show-output analisar_linhas
        let registros_efd = sped_efd::registros(); // tabela de registros
        let arquivo = PathBuf::from("teste");
        let num_line = 1;

        let line: &str =
            "| m210|  01  teste  |11890046,5|11890046,5| 1,65 |0||196185,7|1| 2|3|4|196185,77 |";
        println!("line: '{line}'");

        let mut campos: Vec<String> = line.split_line();
        println!("campos: {campos:?}");

        padronizar_registro(&mut campos);
        println!("campos: {campos:?}");

        let registro: &str = campos[0].as_str();
        println!("registro: {registro}");

        let valores: HashMap<String, String> =
            obter_valores(&registros_efd, &campos, num_line, &arquivo)?;
        println!("valores: {valores:#?}\n");

        assert_eq!(campos.len(), 13);
        assert_eq!(registro, "M210_antigo");
        assert_eq!(valores["REG"], "M210_antigo");
        assert_eq!(valores["VL_CONT_PER"], "196185.77");
        assert_eq!(valores["ALIQ_PIS"], "1.6500");
        assert_eq!(valores["COD_CONT"], "01 teste");

        let line: String = "| m210 |  01   teste  |25066,45|25066,45|0,00|0,00|25066,45|1,65|| |413,62|0,00|0,00|0,00|0,00| 413,6 | ".to_string();
        println!("line: '{line}'");

        let mut campos: Vec<String> = line.split_line();
        println!("campos: {campos:?}");

        padronizar_registro(&mut campos);
        println!("campos: {campos:?}");

        let registro: &str = campos[0].as_str();
        println!("registro: {registro}");

        let valores: HashMap<String, String> =
            obter_valores(&registros_efd, &campos, num_line, &arquivo)?;
        println!("valores: {valores:#?}");

        assert_eq!(campos.len(), 16);
        assert_eq!(registro, "M210");
        assert_eq!(valores["REG"], "M210");
        assert_eq!(valores["VL_CONT_PER"], "413.60");
        assert_eq!(valores["ALIQ_PIS"], "1.6500");
        assert_eq!(valores["COD_CONT"], "01 teste");

        Ok(())
    }

    // Helper function to create a dummy registros_efd for tests
    fn create_dummy_registros_efd() -> HashMap<&'static str, HashMap<u16, (&'static str, Tipo)>> {
        let mut map = HashMap::new();

        let mut reg_0000 = HashMap::new();
        reg_0000.insert(0, ("nivel", Tipo::N)); // Placeholder for level
        reg_0000.insert(1, ("COD_VER", Tipo::N));
        map.insert("0000", reg_0000);

        let mut reg_c100 = HashMap::new();
        reg_c100.insert(0, ("nivel", Tipo::N));
        reg_c100.insert(1, ("IND_OPER", Tipo::C));
        reg_c100.insert(2, ("IND_EMIT", Tipo::C));
        map.insert("C100", reg_c100);

        let mut reg_m210 = HashMap::new(); // Original M210 with 13 fields (M210_antigo)
        reg_m210.insert(0, ("nivel", Tipo::N));
        reg_m210.insert(1, ("FIELD_1", Tipo::N));
        reg_m210.insert(2, ("FIELD_2", Tipo::N));
        reg_m210.insert(3, ("FIELD_3", Tipo::N));
        reg_m210.insert(4, ("FIELD_4", Tipo::N));
        reg_m210.insert(5, ("FIELD_5", Tipo::N));
        reg_m210.insert(6, ("FIELD_6", Tipo::N));
        reg_m210.insert(7, ("FIELD_7", Tipo::N));
        reg_m210.insert(8, ("FIELD_8", Tipo::N));
        reg_m210.insert(9, ("FIELD_9", Tipo::N));
        reg_m210.insert(10, ("FIELD_10", Tipo::N));
        reg_m210.insert(11, ("FIELD_11", Tipo::N));
        reg_m210.insert(12, ("FIELD_12", Tipo::N));
        reg_m210.insert(13, ("FIELD_13", Tipo::N));
        map.insert("M210_antigo", reg_m210);

        let mut reg_m210_novo = HashMap::new(); // New M210 (fewer fields)
        reg_m210_novo.insert(0, ("nivel", Tipo::N));
        reg_m210_novo.insert(1, ("FIELD_1", Tipo::N));
        reg_m210_novo.insert(2, ("FIELD_2", Tipo::N));
        reg_m210_novo.insert(3, ("FIELD_3", Tipo::N));
        reg_m210_novo.insert(4, ("FIELD_4", Tipo::N));
        reg_m210_novo.insert(5, ("FIELD_5", Tipo::N));
        reg_m210_novo.insert(6, ("FIELD_6", Tipo::N));
        reg_m210_novo.insert(7, ("FIELD_7", Tipo::N));
        reg_m210_novo.insert(8, ("FIELD_8", Tipo::N));
        reg_m210_novo.insert(9, ("FIELD_9", Tipo::N));
        reg_m210_novo.insert(10, ("FIELD_10", Tipo::N));
        map.insert("M210", reg_m210_novo);

        map
    }
    #[test]
    fn test_validated_line_try_from_raw_bytes_success() {
        let registros = create_dummy_registros_efd();
        let path = Path::new("test.efd");
        let line_bytes = Ok(b"|0000|001|\n".to_vec());
        let line_number = 1;

        let result = ValidatedLine::try_from_raw_bytes(line_bytes, line_number, path, &registros);
        assert!(result.is_ok());
        let validated_line = result.unwrap().unwrap();
        assert_eq!(validated_line.line_number, 1);
        assert_eq!(validated_line.fields, vec!["0000", "001"]);
    }

    #[test]
    fn test_validated_line_try_from_raw_bytes_empty_line() {
        let registros = create_dummy_registros_efd();
        let path = Path::new("test.efd");
        let line_bytes = Ok(b"  \n".to_vec()); // Line with only whitespace
        let line_number = 1;

        let result = ValidatedLine::try_from_raw_bytes(line_bytes, line_number, path, &registros);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // Should return None for ignorable
    }

    #[test]
    fn test_validated_line_try_from_raw_bytes_io_error() {
        let registros = create_dummy_registros_efd();
        let path = Path::new("test.efd");
        let line_bytes = Err(std::io::Error::new(ErrorKind::InvalidData, "Test IO error"));
        let line_number = 1;

        let result = ValidatedLine::try_from_raw_bytes(line_bytes, line_number, path, &registros);
        assert!(result.is_err());
        // You might want to assert the specific error type if EFDError wrapped io::Error
        if let Err(EFDError::Io(e)) = result {
            assert_eq!(e.kind(), ErrorKind::InvalidData);
        } else {
            panic!("Expected EFDError::Io");
        }
    }

    #[test]
    fn test_validated_line_try_from_raw_bytes_insufficient_fields() {
        let registros = create_dummy_registros_efd();
        let path = Path::new("test.efd");
        let line_bytes = Ok(b"|0000|\n".to_vec()); // Only one field
        let line_number = 1;

        let result = ValidatedLine::try_from_raw_bytes(line_bytes, line_number, path, &registros);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // Should return None for ignorable
    }

    #[test]
    fn test_validated_line_try_from_raw_bytes_undefined_record() {
        let registros = create_dummy_registros_efd();
        let path = Path::new("test.efd");
        let line_bytes = Ok(b"|UNKNOWN|FIELD1|FIELD2|\n".to_vec());
        let line_number = 1;

        let result = ValidatedLine::try_from_raw_bytes(line_bytes, line_number, path, &registros);
        assert!(result.is_err());
        if let Err(EFDError::UndefinedRecord { record, .. }) = result {
            assert_eq!(record, "UNKNOWN");
        } else {
            panic!("Expected EFDError::UndefinedRecord");
        }
    }

    #[test]
    fn test_get_string_utf8_valid_utf8() {
        let path = Path::new("test.efd");
        let bytes = "Olá mundo! 🙌".as_bytes();
        let result = get_string_utf8(bytes, 1, path).unwrap();
        assert_eq!(result, "Olá mundo! 🙌");
    }

    #[test]
    fn test_get_string_utf8_windows_1252_fallback() {
        let path = Path::new("test.efd");
        // Bytes for "ação" in WINDOWS-1252
        let bytes = &[0x61, 0xe7, 0xe3, 0x6f];
        let result = get_string_utf8(bytes, 1, path).unwrap();
        assert_eq!(result, "ação");
    }

    #[test]
    /// cargo test -- --show-output windows_1252_valid
    fn test_get_string_utf8_windows_1252_valid() {
        let path = Path::new("test.efd");
        // A sequência de bytes é inválida em UTF-8, mas VÁLIDA em WINDOWS-1252
        let bytes = &[0xFF, 0xFE, 0xFD];
        let result = get_string_utf8(bytes, 1, path);
        println!("{result:?}");
        assert!(result.is_ok()); // Esperamos sucesso porque WINDOWS-1252 consegue decodificar
        assert_eq!(result.unwrap(), "ÿþý".to_string()); // Verifica o valor decodificado
    }

    #[test]
    /// cargo test -- --show-output invalid_utf8_valid_windows_1252_with_special_chars
    fn test_get_string_utf8_invalid_utf8_valid_windows_1252_special() {
        let path = Path::new("test.efd");
        // 'ação' em WINDOWS-1252 (0xE7 é 'ç', 0xE3 é 'ã', 0xF5 é 'õ')
        let bytes = &[0x61, 0xe7, 0xe3, 0x6f];
        let result = get_string_utf8(bytes, 1, path);
        println!("{result:?}");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "ação".to_string()); // 'o' em vez de 'õ' se 0xF5 não for o caractere certo.
        // Para 'ação', seria 0xE7, 0xE3, 0x6F
    }

    #[test]
    fn test_padronizar_registro_uppercase() {
        let mut fields = vec!["d101".to_string(), "0".to_string()];
        padronizar_registro(&mut fields);
        assert_eq!(fields[0], "D101");
    }

    #[test]
    fn test_padronizar_registro_m210_antigo() {
        // M210 with 13 fields should become M210_antigo
        let mut fields = vec![
            "m210".to_string(),
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
            "5".to_string(),
            "6".to_string(),
            "7".to_string(),
            "8".to_string(),
            "9".to_string(),
            "10".to_string(),
            "11".to_string(),
            "12".to_string(),
        ];
        padronizar_registro(&mut fields);
        assert_eq!(fields[0], "M210_antigo");
    }

    #[test]
    fn test_padronizar_registro_m210_novo() {
        // M210 with fewer fields should remain M210
        let mut fields = vec![
            "m210".to_string(),
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
            "5".to_string(),
        ];
        padronizar_registro(&mut fields);
        assert_eq!(fields[0], "M210");
    }

    #[test]
    fn test_is_record_valid_found() {
        let registros = create_dummy_registros_efd();
        let fields = vec!["0000".to_string(), "data".to_string()];
        let path = Path::new("test.efd");
        let line_num = 1;
        let result = is_record_valid(&registros, &fields, &line_num, path);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_is_record_valid_not_found() {
        let registros = create_dummy_registros_efd();
        let fields = vec!["XXXX".to_string(), "data".to_string()];
        let path = Path::new("test.efd");
        let line_num = 1;
        let result = is_record_valid(&registros, &fields, &line_num, path);
        assert!(result.is_err());
        if let Err(EFDError::UndefinedRecord { record, .. }) = result {
            assert_eq!(record, "XXXX");
        } else {
            panic!("Expected EFDError::UndefinedRecord");
        }
    }
}
