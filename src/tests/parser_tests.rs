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
        // Tenta iniciar, mas se falhar (já iniciado), converte o erro em Ok e segue a vida
        let _ = env_logger::builder()
            .is_test(true) // Configures the logger for testing, often redirects to stderr
            .filter_level(log::LevelFilter::Warn) // Set minimum level to show warnings
            .try_init();
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
    |c100|0|1|865322|01|00|002|16798||15012018|26012018|2541,39|0|||2541,39|9||0|||||||25,52|117,57|0|0|\n\
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

    let mut final_sped_file = Arc::try_unwrap(sped_file_data)
        .expect("Mutex still has multiple strong references after parallel processing")
        .into_inner()
        .expect("Mutex poisoned");

    // Desbloqueia e obtém a SpedFile para ordenação e impressão
    // let mut final_sped_file = sped_file_data.lock().unwrap();

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
    let registro_0000: &Registro0000 = final_sped_file.obter_registro::<Registro0000>("0000")?;

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
