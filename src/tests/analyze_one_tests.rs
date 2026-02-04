use crate::{create_a_temp_file, setup_logging};

use super::*;
use glob::glob;
use rust_decimal_macros::dec;
use std::path::PathBuf;

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

const SPED_EFD: &str = "\
    |0000|003|1||8A131222555502D2CD834A204E6666E4BFF8B99A1| 01012018 |31012018|EMPRESA Teste ABC|12345678901234|SP|3555338||00|0|\n\
    |0001|0|\n\
    |0100|Fulano de Tal|12345678901|1SP123456|1122334455|12345|Avenida Sem Nome|51||Bairro|12345678901|0000000000|nome@email.com.br|3535222|\n\
    |0990|2297|\n\
    |A001|0|\n\
    |A010|123456789000221|\n\
    |A990|3|\n\
    |C001|0|\n\
    |C010|12345678900001|1|\n\
    |C190|55|15022018|15022018|776973|04022120||1234567,04|\n\
    |C191|12345678901238|66|1105|400,5||301,76|1,65|||111,0|cod_cta_pis|\n\
    |C191|12345678901112|50|1234|600,2||500,93|1,65|||666,0|cod_cta_aab|\n\
    |C191|12345678901111|50|1234|600,2||500,93|2,65|||444,0|cod_cta_aac|\n\
    |C191|12345678901111|50|1234|600,2||500,93|3,65|||333,0|cod_cta_aad|\n\
    |C191|12345678901113|50|1235|600,2||510,93|4,65|||777,0|cod_cta_ccc|\n\
    |C191|12345678901115|50|1236|600,2||500,94|5,65|||888,0|cod_cta_aaa|\n\
    |C191|12345678901111|50|1236|600,2||500,94|5,65|||888,0|cod_cta_xxx|\n\
    |C191|12345678901113|50|1235|600,2||500,92|6,65|||999,0|cod_cta_bbb|\n\
    |C191|12345678900000||1235|600,2||500,92|1,30|||999,0|cod_cta_bbb|\n\
    |C191|12345678900000|56|1235|||500,92|1,40|||999,0|cod_cta_bbb|\n\
    |C191|12345678900000||1235|||500,92|1,50|||999,0|cod_cta_bbb|\n\
    |C195|12345678901237|66|1101|400,5||301,76|7,60|||555,0|cod_cta_cof|\n\
    |C195|12345678901111|50|1234|600,2||500,92|9,60|||222,0|cod_cta_aaa|\n\
    |C195|12345678900000||1234|600,2||500,92|9,60|||222,0|cod_cta_aaa|\n\
    |C195|12345678900000|56|1234|||500,92|9,60|||222,0|cod_cta_aaa|\n\
    |C195|12345678900000||1234|||500,92|9,60|||222,0|cod_cta_aaa|\n\
    |1500|042024|01||202|0,69|0|0,69|0|0|0|0,69|0||0|0|0|0,69|\n\
    |9999|10|\n\
    ";

#[test]
/// cargo test -- --show-output test_analyze_one_file_new
fn test_analyze_one_file_new() -> EFDResult<()> {
    setup_logging(); // Initialize logger for this test

    let efd_files = get_efd_files()?;
    println!("efd_files: {efd_files:#?}");

    let path = if let Some(p) = efd_files.get(18) {
        p
    } else {
        return Ok(());
    };

    // indicatif ProgressBar + rayon
    let total: usize = efd_files.len();
    let multiprogressbar: MultiProgress = MultiProgress::new();
    let index = 0;

    let informacaoes =
        analyze_one_file(&multiprogressbar, path, index, total).map_loc(|error| {
            // Aqui mapeamos o EFDError retornado por analyze_one_file
            // para a nossa nova variante AnalyzeFileError
            EFDError::AnalyzeFileError {
                source: Box::new(error),
                arquivo: path.clone(),
            }
        })?;

    println!("cnpj_base: {}", informacaoes.cnpj_base);
    println!("periodo_de_apuracao: {}", informacaoes.periodo_de_apuracao);
    println!("mensagens: {}", informacaoes.messages);

    let all_docs = informacaoes.all_docs;

    all_docs
        .iter()
        .enumerate() // 1. Anexa o índice original a cada item: (index, doc)
        //.filter(|(_, doc)| doc.registro.starts_with('M'))
        .filter(|(_, doc)| doc.registro.eq_ignore_ascii_case("A170"))
        //.filter(|(_, doc)| doc.aliq_cofins.is_some_and(|v| v > 0.0)) // 2. Filtra apenas os que atendem a condição
        .take(20) // 3. Pega apenas os primeiros 10 resultados que passaram no filtro
        .for_each(|(index, doc)| {
            // 4. Executa a ação para cada item restante
            println!("docs[{index}]: {doc:?}\n");
        });

    println!("all_docs.len(): {:?}", all_docs.len());

    Ok(())
}

#[test]
/// cargo test -- --show-output test_analyze_one_sped_file
fn test_analyze_one_sped_file() -> EFDResult<()> {
    setup_logging(); // Initialize logger for this test

    let efd_file = create_a_temp_file(SPED_EFD, true)?;
    let path = efd_file.path();

    // indicatif ProgressBar + rayon
    let total: usize = 1; // Apenas 1 arquivo EFD
    let multiprogressbar: MultiProgress = MultiProgress::new();
    let index = 0;

    let informacaoes =
        analyze_one_file(&multiprogressbar, path, index, total).map_loc(|error| {
            // Aqui mapeamos o EFDError retornado por analyze_one_file
            // para a nossa nova variante AnalyzeFileError
            EFDError::AnalyzeFileError {
                source: Box::new(error),
                arquivo: path.to_path_buf(),
            }
        })?;

    println!("cnpj_base: {}", informacaoes.cnpj_base);
    println!("periodo_de_apuracao: {}", informacaoes.periodo_de_apuracao);
    println!("mensagens: {}", informacaoes.messages);

    let all_docs = informacaoes.all_docs;

    all_docs.iter().enumerate().for_each(|(index, doc)| {
        println!("docs[{index}]: {doc:?}\n");
    });

    println!("all_docs.len(): {:?}", all_docs.len());

    assert_eq!(
        all_docs[0].aliq_pis,
        Some(dec!(1.65)),
        "Valor da alíquota incorreto!"
    );

    assert_eq!(
        all_docs[0].tipo_de_credito,
        Some(crate::TipoDeCredito::PresumidoAgroindustria)
    );

    assert_eq!(
        all_docs[1].aliq_pis,
        Some(dec!(3.65)),
        "Valor da alíquota incorreto!"
    );

    assert_eq!(
        all_docs[1].tipo_de_credito,
        Some(crate::TipoDeCredito::AliquotasDiferenciadas)
    );

    assert_eq!(
        all_docs[2].aliq_pis,
        Some(dec!(1.30)),
        "Valor da alíquota incorreto!"
    );

    assert!(all_docs[3].aliq_pis.is_none());
    assert!(all_docs[4].aliq_pis.is_none());

    Ok(())
}
