use super::*;
use crate::{
    EFDResult, SpedFile, SpedRecordTrait, create_a_temp_file, read_and_parse_file, setup_logging,
};
use glob::glob;
use indicatif::MultiProgress;
use rust_decimal_macros::dec;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

const SPED_EFD: &str = "\
    XX lixo XX
    |0000|003|1||8A131222555502D2CD834A204E6666E4BFF8B99A1| 01012018 |31012018|EMPRESA Teste ABC|12345678901234|SP|3555338||00|0|
    |0001|3|
    |0001|
    |0100|Fulano de Tal|12345678901|1SP123456|1122334455|12345|Avenida Sem Nome|51||Bairro|12345678901|0000000000|nome@email.com.br|3535222|
    |0990|2297|
    |A001|0|
    |A010|123456789000221|
    |A990|3|
    |C001|0|
    |C010|987654321000221||
    |c100|0|1|865322|01|00|002|16798||15012018|26012018|2541,39|0|||2541,39|9||0|||||||25,52|117,57|0|0|
    
    |001|
    |c170|1|192428||1|31|1234,74|0|0|020|1653|16|16,67|18|20197,9|112210,57|0|0|0|49||0|0|0|56|1234,74|1,65|||2221,77|56|1234,74|7,6|||10233,61|9110200000101133|
    |M001|0|
    |M200|71785,57|44600,74|0|26825,83|26825,83|0|0|0|0|0|0|0|
    |M210|01|5130600,46|5130600,46|0|0|5130600,46|1,65|||68096,27|0|0|0|0|68096,27|
    |M210|01|12345678,47|12345678,47|1,65|0||470307,38|0|0|0|0|470307,38|
    |M600|336358,91|212547,48|0|123811,43|123811,43|0|0|0|0|0|0|0|
    |M610|01|5130600,46|5130600,46|0|0|5130600,46|7,6|||313655,53|0|0|0|0|313655,53|
    |M610|01|12345678,47|12345678,47|7,6|0||2166264,29|0|0|0|0|2166264,29|
    |M990|90|
    |1500|042024|01||202|0,69|0|0,69|0|0|0|0,69|0||0|0|0|0,69|
    |1507|042024|01||302|19,53|0|19,53|0|0|0|19,53|0||0|0|0|19,53|
    |1990|127|
    |9001|0|
    |9900|0000|1|
    |9900|P990|1|
    |9900|9900|60|
    |9990|63|
    |9999|7402|
    01RCAAEPDR0
    |POS_9999_A|Linha que deve ser ignorada|
    |POS_9999_B|Outra linha para ignorar|
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

    //let file = File::open(path)?;
    //let efd_files = get_efd_files()?;
    //println!("efd_files: {efd_files:#?}");
    //let path = &efd_files[15];

    let multiprogressbar: MultiProgress = MultiProgress::new();
    let index = 0;
    let total = 1;
    let mut sped_file = read_and_parse_file(path, &multiprogressbar, index, total)?;

    // Desbloqueia e obtém a SpedFile para ordenação e impressão
    // let mut sped_file = sped_file_data.lock().unwrap();

    sped_file.sort_records_by_line_number();
    sped_file.print_structure();

    let registros_do_bloco_c = sped_file.get_bloco_c();
    println!("Bloco C: {:?}\n", registros_do_bloco_c);

    // Se for o Registro 0000, podemos extrair o PA e o CNPJ base para o progress bar
    // Retorna Result<&Registro0000, EFDError>
    // O '?' já trata o erro se não encontrar ou se o tipo estiver errado
    let registro_0000: &Registro0000 = sped_file.obter_registro::<Registro0000>("0000")?;

    println!("Registro 0000 encontrado:");
    println!("registro_0000: {registro_0000:?}");
    println!("registro_0000.dt_ini: {:?}", registro_0000.dt_ini);
    println!("registro_0000.cnpj: {:?}", registro_0000.cnpj);

    // referência mutável a um bloco e precisar alterar um campo específico
    if let Some(reg) = sped_file.obter_registro_mut::<RegistroC170>("C170") {
        let valor_original = reg.vl_bc_cofins;
        println!("Valor Original: {:?}", reg.vl_bc_cofins);
        reg.vl_bc_cofins = Some(dec!(1500.00));
        let valor_alterado = reg.vl_bc_cofins;
        println!("Valor Alterado: {:?}", reg.vl_bc_cofins);
        assert_eq!(valor_original, Some(dec!(1234.74)));
        assert_eq!(valor_alterado, Some(dec!(1500.00)));
    }

    // Retorna Vec<&RegistroC100>
    let registros_c100: Vec<&RegistroC100> =
        sped_file.obter_lista_registros::<RegistroC100>("C100");
    println!("Total de RegistroC100: {}", registros_c100.len());
    println!("registros_C100: {registros_c100:?}");
    assert!(
        registros_c100
            .first()
            .is_some_and(|r| r.registro_name() == "C100")
    );

    // Iteração direta no resultado
    for registro_c100 in registros_c100 {
        println!(
            "registro_c100 Linha {}: ValorDoc {:?}",
            registro_c100.line_number, registro_c100.vl_doc
        ); // Exemplo
    }

    Ok(())
}
