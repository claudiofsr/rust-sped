use crate::{
    DELIMITER_CHAR, EFDResult, SpedRecord, SpedRecordTrait, blocos::*, dispatch_sped_parsers,
};

use log::warn;
use std::path::Path;

/// A trait for parsing different types of SPED records.
///
/// Each concrete SPED record struct (e.g., `Registro0000`) should implement this trait
/// to provide a standardized way of parsing its specific fields from a raw line.
pub trait SpedParser {
    /// The concrete output type of the parser, which must implement `SpedRecordTrait`.
    ///
    /// This associated type ensures that all records can be treated polymorphically
    /// through the `SpedRecordTrait` and can be moved safely across threads (`Send + Sync`).
    type Output: SpedRecordTrait + Send + Sync + 'static;

    /// Parses a slice of SPED fields into a specific record type.
    ///
    /// # Arguments
    /// * `file_path` - The path to the SPED file being parsed, used for error reporting.
    /// * `line_number` - The line number in the file, used for error reporting.
    /// * `fields` - A slice of string slices, where each element represents a field
    ///   from the SPED line, separated by `DELIMITER_CHAR`.
    ///
    /// # Returns
    /// * `EFDResult<Self::Output>` - `Ok` containing the parsed record struct if successful,
    ///   or `Err(EFDError)` if a parsing error occurs within
    ///   the specific record's logic (e.g., invalid date format).
    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output>;
}

/// Parses a single line of a SPED file into a `SpedRecord` variant.
///
/// This function acts as the central dispatcher for parsing SPED file lines.
/// It tokenizes the input line by the `DELIMITER_CHAR`, performs basic validation,
/// identifies the record type from the second field, and then uses the
/// `dispatch_sped_parsers!` macro to call the specific parser for that record.
///
/// # Arguments
/// * `file_path` - The path to the SPED file, used for detailed error/warning messages.
/// * `line_number` - The current line number being processed, used for context in logs.
/// * `line` - A string slice representing a single line from the SPED file.
///
/// # Returns
/// * `EFDResult<Option<SpedRecord>>` -
///     * `Ok(Some(SpedRecord))` if the line was successfully parsed into a known record type.
///     * `Ok(None)` if the line is malformed, empty, or represents an unsupported record type,
///       and should be skipped (a warning will be logged).
///     * `Err(EFDError)` if a critical parsing error occurred within a specific record's
///       parsing logic.
pub fn parse_sped_fields(
    file_path: &Path,
    line_number: usize,
    line: &str,
) -> EFDResult<Option<SpedRecord>> {
    // Split the line by the delimiter and trim whitespace from each field.
    let fields: Vec<&str> = line.split(DELIMITER_CHAR).map(|s| s.trim()).collect();

    // Basic validation: A valid SPED record line must have at least 4 fields:
    // '|', RECORD_TYPE, FIELD1, FIELD2, ..., '|'.
    // And the record type (fields[1]) must not be empty.
    if fields.len() < 4 || fields[1].is_empty() {
        let msg = "Line with insufficient number of fields or empty record identifier.";
        warn!(
            "[{}:Linha nº {:2}] {msg}\nExpected at least 4 fields, found {}. Line: '{}'",
            file_path.display(),
            line_number,
            fields.len(),
            line
        );
        return Ok(None); // Log a warning and skip processing this line.
    }

    // Extract the record identifier (e.g., "0000", "C100") and convert to uppercase
    // to ensure case-insensitive matching if necessary (SPED typically uses uppercase).
    let registro = fields[1].to_uppercase();

    // Dispatch to the appropriate parser based on the record identifier.
    // The `dispatch_sped_parsers!` macro handles the `match` statement internally,
    // calling the `parse_registro!` macro for each record type.
    dispatch_sped_parsers!(
        registro.as_str(), // Convert String to &str for pattern matching
        file_path,
        line_number,
        &fields, // Pass a slice of fields

        simple => [
            // Blocos de 0 a P - Registros sem lógica condicional de despacho.
            // Each entry is `("REGISTRO_NAME_LITERAL", StructForRecord)`.
            // Blocos are listed in numerical/alphabetical order for readability.

            // Block 0
            ("0000", Registro0000), ("0001", Registro0001), ("0035", Registro0035),
            ("0100", Registro0100), ("0110", Registro0110), ("0111", Registro0111),
            ("0120", Registro0120), ("0140", Registro0140), ("0145", Registro0145),
            ("0150", Registro0150), ("0190", Registro0190), ("0200", Registro0200),
            ("0205", Registro0205), ("0206", Registro0206), ("0208", Registro0208),
            ("0400", Registro0400), ("0450", Registro0450), ("0500", Registro0500),
            ("0600", Registro0600), ("0900", Registro0900), ("0990", Registro0990),

            // Block 1
            ("1001", Registro1001), ("1010", Registro1010), ("1011", Registro1011),
            ("1020", Registro1020), ("1050", Registro1050), ("1100", Registro1100),
            ("1101", Registro1101), ("1102", Registro1102), ("1200", Registro1200),
            ("1210", Registro1210), ("1220", Registro1220), ("1300", Registro1300),
            ("1500", Registro1500), ("1501", Registro1501), ("1502", Registro1502),
            ("1600", Registro1600), ("1610", Registro1610), ("1620", Registro1620),
            ("1700", Registro1700), ("1800", Registro1800), ("1809", Registro1809),
            ("1900", Registro1900), ("1990", Registro1990),

            // Block A
            ("A001", RegistroA001), ("A010", RegistroA010), ("A100", RegistroA100),
            ("A110", RegistroA110), ("A111", RegistroA111), ("A120", RegistroA120),
            ("A170", RegistroA170), ("A990", RegistroA990),

            // Block C
            ("C001", RegistroC001), ("C010", RegistroC010), ("C100", RegistroC100),
            ("C110", RegistroC110), ("C111", RegistroC111), ("C120", RegistroC120),
            ("C170", RegistroC170), ("C175", RegistroC175), ("C180", RegistroC180),
            ("C181", RegistroC181), ("C185", RegistroC185), ("C188", RegistroC188),
            ("C190", RegistroC190), ("C191", RegistroC191), ("C195", RegistroC195),
            ("C198", RegistroC198), ("C199", RegistroC199), ("C380", RegistroC380),
            ("C381", RegistroC381), ("C385", RegistroC385), ("C395", RegistroC395),
            ("C396", RegistroC396), ("C400", RegistroC400), ("C405", RegistroC405),
            ("C481", RegistroC481), ("C485", RegistroC485), ("C489", RegistroC489),
            ("C490", RegistroC490), ("C491", RegistroC491), ("C495", RegistroC495),
            ("C499", RegistroC499), ("C500", RegistroC500), ("C501", RegistroC501),
            ("C505", RegistroC505), ("C509", RegistroC509), ("C600", RegistroC600),
            ("C601", RegistroC601), ("C605", RegistroC605), ("C609", RegistroC609),
            ("C800", RegistroC800), ("C810", RegistroC810), ("C820", RegistroC820),
            ("C830", RegistroC830), ("C860", RegistroC860), ("C870", RegistroC870),
            ("C880", RegistroC880), ("C890", RegistroC890), ("C990", RegistroC990),

            // Block D
            ("D001", RegistroD001), ("D010", RegistroD010), ("D100", RegistroD100),
            ("D101", RegistroD101), ("D105", RegistroD105), ("D111", RegistroD111),
            ("D200", RegistroD200), ("D201", RegistroD201), ("D205", RegistroD205),
            ("D209", RegistroD209), ("D300", RegistroD300), ("D309", RegistroD309),
            ("D350", RegistroD350), ("D359", RegistroD359), ("D500", RegistroD500),
            ("D501", RegistroD501), ("D505", RegistroD505), ("D509", RegistroD509),
            ("D600", RegistroD600), ("D601", RegistroD601), ("D605", RegistroD605),
            ("D609", RegistroD609), ("D990", RegistroD990),

            // Block F
            ("F001", RegistroF001), ("F010", RegistroF010), ("F100", RegistroF100),
            ("F111", RegistroF111), ("F120", RegistroF120), ("F129", RegistroF129),
            ("F130", RegistroF130), ("F139", RegistroF139), ("F150", RegistroF150),
            ("F200", RegistroF200), ("F205", RegistroF205), ("F210", RegistroF210),
            ("F211", RegistroF211), ("F500", RegistroF500), ("F509", RegistroF509),
            ("F510", RegistroF510), ("F519", RegistroF519), ("F525", RegistroF525),
            ("F550", RegistroF550), ("F559", RegistroF559), ("F560", RegistroF560),
            ("F569", RegistroF569), ("F600", RegistroF600), ("F700", RegistroF700),
            ("F800", RegistroF800), ("F990", RegistroF990),

            // Block I
            ("I001", RegistroI001), ("I010", RegistroI010), ("I100", RegistroI100),
            ("I199", RegistroI199), ("I200", RegistroI200), ("I299", RegistroI299),
            ("I300", RegistroI300), ("I399", RegistroI399), ("I990", RegistroI990),

            // Block M (Excludes conditional M210 and M610, which are in the conditional block)
            ("M001", RegistroM001), ("M100", RegistroM100), ("M105", RegistroM105),
            ("M110", RegistroM110), ("M115", RegistroM115), ("M200", RegistroM200),
            ("M205", RegistroM205), ("M211", RegistroM211), ("M215", RegistroM215),
            ("M220", RegistroM220), ("M225", RegistroM225), ("M230", RegistroM230),
            ("M300", RegistroM300), ("M350", RegistroM350), ("M400", RegistroM400),
            ("M410", RegistroM410), ("M500", RegistroM500), ("M505", RegistroM505),
            ("M510", RegistroM510), ("M515", RegistroM515), ("M600", RegistroM600),
            ("M605", RegistroM605), ("M611", RegistroM611), ("M615", RegistroM615),
            ("M620", RegistroM620), ("M625", RegistroM625), ("M630", RegistroM630),
            ("M700", RegistroM700), ("M800", RegistroM800), ("M810", RegistroM810),
            ("M990", RegistroM990),

            // Block P
            ("P001", RegistroP001), ("P010", RegistroP010), ("P100", RegistroP100),
            ("P110", RegistroP110), ("P199", RegistroP199), ("P200", RegistroP200),
            ("P210", RegistroP210), ("P990", RegistroP990),

            // Block 9
            ("9001", Registro9001), ("9900", Registro9900), ("9990", Registro9990),
            ("9999", Registro9999),
        ],
        conditional => [
            // Registros com despacho condicional baseado na quantidade de campos (`fields.len()`).
            // Each entry is `("REGISTRO_NAME_LITERAL", CONDITION_EXPRESSION, StructIfTrue, StructIfFalse)`.
            ("M210", fields.len() == 15, RegistroM210Antigo, RegistroM210),
            ("M610", fields.len() == 15, RegistroM610Antigo, RegistroM610),
        ]
    )
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//
//
// cargo test -- --help
// cargo test -- --nocapture
// cargo test -- --show-output

/// Run tests with:
/// cargo test -- --show-output parser_tests
#[cfg(test)]
mod parser_tests {
    use super::*;
    use crate::{EFDResult, NEWLINE_BYTE, SpedFile, create_a_temp_file, get_string_utf8};
    use glob::glob;
    use rayon::prelude::*;
    use std::{
        fs::File,
        io::{BufRead, BufReader},
        path::PathBuf,
        sync::{Arc, Mutex},
    };

    // Don't forget to initialize logging in your test setup if you want to see warnings
    // from tests. This can be done once in a `before_each` style setup if you have one,
    // or at the start of individual tests.
    // For simple test cases, you can just call it once.
    // However, `env_logger::init()` panics if called multiple times, so use `try_init()`
    // or a `Once` guard for tests.
    use std::sync::Once;
    static INIT: Once = Once::new();

    fn setup_logging() {
        INIT.call_once(|| {
            env_logger::builder()
                .is_test(true) // Configures the logger for testing, often redirects to stderr
                .filter_level(log::LevelFilter::Warn) // Set minimum level to show warnings
                .try_init()
                .expect("Failed to initialize logger");
        });
    }

    const SPED_EFD: &str = "\
    |0000|003|1||8A131222555502D2CD834A204E6666E4BFF8B99A1| 01012018 |31012018|EMPRESA Teste ABC|12345678901234|SP|3555338||00|0|\n\
    |0001|0|\n\
    |0001|\n\
    |0100|Fulano de Tal|12345678901|1SP123456|1122334455|12345|Avenida Sem Nome|51||Bairro|12345678901|0000000000|nome@email.com.br|3535222|\n\
    |0990|2297|\n\
    |A001|0|\n\
    |A010|123456789000221|\n\
    |A990|3|\n\
    |C001|0|\n\
    |C010|987654321000221||\n\
    |C100|0|1|865322|01|00|002|16798||15012018|26012018|2541,39|0|||2541,39|9||0|||||||25,52|117,57|0|0|\n\
    \n\
    |001|\n\
    |C170|1|192428||1|31|134652,74|0|0|020|1653|16|16,67|18|20197,9|112210,57|0|0|0|49||0|0|0|56|134652,74|1,65|||2221,77|56|134652,74|7,6|||10233,61|9110200000101133|\n\
    |1500|042024|01||202|0,69|0|0,69|0|0|0|0,69|0||0|0|0|0,69|\n\
    |1500|042024|01||302|19,53|0|19,53|0|0|0|19,53|0||0|0|0|19,53|\n\
    |1990|127|\n\
    |9001|0|\n\
    |9900|0000|1|\n\
    |9900|P990|1|\n\
    |9900|9900|60|\n\
    |9990|63|\n\
    |9999|7402|\n\
    01RCAAEPDR0\n\
    |POS_9999_A|Linha que deve ser ignorada|\n\
    |POS_9999_B|Outra linha para ignorar|\n\
    ";

    #[allow(dead_code)]
    fn get_efd_files() -> EFDResult<Vec<PathBuf>> {
        let mut efd_files = Vec::new();
        // O padrão "PISCOFINS_*.txt" irá corresponder a arquivos que começam com "PISCOFINS_" e terminam com ".txt"
        // `glob` retorna um iterador de Results, onde cada Ok contém um PathBuf
        for entry in glob("PISCOFINS_*.txt")? {
            // Desembrulha o Result para cada caminho encontrado
            efd_files.push(entry?);
        }
        Ok(efd_files)
    }

    #[test]
    /// cargo test -- --show-output parser_serial
    fn parser_serial() -> EFDResult<()> {
        setup_logging(); // Initialize logger for this test
        let temp_file = create_a_temp_file(SPED_EFD, true)?;
        let path = temp_file.path();
        let file = File::open(path)?;

        // Reading a file line by line (for large files or streaming):
        // For large files where reading the entire content into memory is inefficient,
        // or if you need to process the file line by line,
        // you can use std::io::BufReader with lines().
        let reader = BufReader::new(file);

        let mut sped_file_data = SpedFile::new(); // Instancia a estrutura principal

        for (index, result_line) in reader.lines().enumerate() {
            let line_number = index + 1;
            let line: String = result_line?;

            if line.trim().is_empty() {
                continue;
            }

            // Now, parse_sped_fields returns Option<SpedRecord>
            match parse_sped_fields(path, line_number, &line)? {
                Some(sped_record) => {
                    // Only print if a record was actually parsed
                    sped_record.println();

                    // Adiciona o registro à estrutura SpedFile
                    sped_file_data.add_record(sped_record);
                }
                None => {
                    // This line was skipped, do nothing or log that it was skipped.
                    // The warning message is already handled inside parse_sped_fields.
                }
            }
        }

        // Após ler todo o arquivo, ordene os registros dentro de cada bloco
        sped_file_data.sort_records_by_line_number();

        // Imprime a estrutura final (opcional, para verificação)
        sped_file_data.print_structure();

        Ok(())
    }

    #[test]
    /// cargo test -- --show-output run_parser
    fn run_parser() -> EFDResult<()> {
        // Initialize logger for this test
        setup_logging();

        // Call the separate function that contains the main logic and can return Result
        let run_result = parser_serial();

        // Now handle the result returned by the 'run' function
        match run_result {
            Ok(_) => {
                // If it's Ok, return Ok to the test framework
                Ok(())
            }
            Err(error) => {
                eprintln!("Operation failed:");
                eprintln!("Error: {}", error); // Using Display prints the #[error] message
                // If it's an Err, return the error to the test framework
                // This will cause the test to fail and show the error.
                Err(error)
            }
        }
    }

    #[test]
    /// cargo test -- --show-output parser_parallel
    fn parser_parallel() -> EFDResult<()> {
        setup_logging(); // Initialize logger for this test
        let temp_file = create_a_temp_file(SPED_EFD, true)?;
        let path = temp_file.path();

        //let efd_files = get_efd_files()?;
        //println!("efd_files: {efd_files:#?}");
        //let path = &efd_files[15];

        let file = File::open(path)?;

        // Usamos Arc<Mutex<SpedFile>> para permitir que múltiplas threads adicionem registros
        let sped_file_data = Arc::new(Mutex::new(SpedFile::new()));

        BufReader::new(file)
            .split(NEWLINE_BYTE)
            .enumerate()
            .par_bridge() // parallelize
            .filter_map(|(count, result_vec_bytes)| {
                let line_number = count + 1;

                let vec_bytes: Vec<u8> = match result_vec_bytes {
                    Ok(v) => v,
                    Err(e) => {
                        // Log a specific error for reading bytes if needed, then skip
                        warn!(
                            "[{}:{}] Erro ao ler bytes da linha: {}",
                            path.display(),
                            line_number,
                            e
                        );
                        return None;
                    }
                };

                if vec_bytes.trim_ascii().is_empty() {
                    return None; // If line is empty, skip this line
                }

                let result_line = get_string_utf8(&vec_bytes, line_number, path);

                match result_line {
                    Ok(line) => Some((line_number, line)),
                    Err(e) => {
                        // The get_string_utf8 function already logs the error, just skip here
                        warn!(
                            "[{}:{}] Erro de decodificação UTF-8, linha ignorada: {}",
                            path.display(),
                            line_number,
                            e
                        );
                        None // If UTF-8 conversion fails, skip this line
                    }
                }
            })
            .try_for_each(|(line_number, line)| -> EFDResult<()> {
                // Now, parse_sped_fields returns Option<SpedRecord>
                match parse_sped_fields(path, line_number, &line)? {
                    Some(sped_record) => {
                        sped_record.println();

                        // Bloqueia o mutex e adiciona o registro
                        sped_file_data.lock().unwrap().add_record(sped_record);

                        Ok(()) // Successfully parsed and printed, continue
                    }
                    None => {
                        // Line was skipped, but processing should continue.
                        // We still return Ok(()) because it's not an unrecoverable error
                        // for the `try_for_each` loop.
                        Ok(())
                    }
                }
            })?;

        // Desbloqueia e obtém a SpedFile para ordenação e impressão
        let mut final_sped_file = sped_file_data.lock().unwrap();
        final_sped_file.sort_records_by_line_number();
        final_sped_file.print_structure();

        if let Some(registros) = final_sped_file.obter_bloco_option('C') {
            println!("Bloco C: {:?}\n", registros);
        }

        println!(
            "final_sped_file.blocos[&'C'].registros: {:?}\n",
            final_sped_file.blocos[&'C'].registros
        );

        // Se for o Registro 0000, podemos extrair o PA e o CNPJ base para o progress bar
        // Retorna Result<&Registro0000, EFDError>
        // O '?' já trata o erro se não encontrar ou se o tipo estiver errado
        let registro_0000: &Registro0000 =
            final_sped_file.obter_registro::<Registro0000>("0000")?;

        println!("Registro 0000 encontrado:");
        println!("registro_0000: {registro_0000:?}");
        println!("registro_0000.dt_ini: {:?}", registro_0000.dt_ini);
        println!("registro_0000.cnpj: {:?}", registro_0000.cnpj);

        // Retorna Vec<&RegistroC100>
        let registros_c100: Vec<&RegistroC100> =
            final_sped_file.obter_lista_registros::<RegistroC100>("C100");
        println!("Total de RegistroC100: {}", registros_c100.len());
        println!("registros_C100: {registros_c100:?}");

        // Iteração direta no resultado
        for registro_c100 in registros_c100 {
            println!(
                "registro_c100 Linha {}: ValorDoc {:?}",
                registro_c100.line_number, registro_c100.vl_doc
            ); // Exemplo
        }

        Ok(())
    }
}
