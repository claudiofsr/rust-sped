use super::*;
use glob::glob;
use std::{path::PathBuf, sync::Once};

// Don't forget to initialize logging in your test setup if you want to see warnings
// from tests. This can be done once in a `before_each` style setup if you have one,
// or at the start of individual tests.
// For simple test cases, you can just call it once.
// However, `env_logger::init()` panics if called multiple times, so use `try_init()`
// or a `Once` guard for tests.
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
/// cargo test -- --show-output test_analyze_one_file_new
fn test_analyze_one_file_new() -> EFDResult<()> {
    setup_logging(); // Initialize logger for this test

    let efd_files = get_efd_files()?;
    println!("efd_files: {efd_files:#?}");

    let path = if let Some(p) = efd_files.get(1) {
        p
    } else {
        return Ok(());
    };

    // indicatif ProgressBar + rayon
    let total: usize = efd_files.len();
    let multiprogressbar: MultiProgress = MultiProgress::new();
    let index = 0;

    let informacaoes =
        analyze_one_file_new(&multiprogressbar, path, index, total).map_err(|error| {
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
        .filter(|(_, doc)| doc.registro.eq_ignore_ascii_case("F100"))
        //.filter(|(_, doc)| doc.aliq_cofins.is_some_and(|v| v > 0.0)) // 2. Filtra apenas os que atendem a condição
        .take(20) // 3. Pega apenas os primeiros 10 resultados que passaram no filtro
        .for_each(|(index, doc)| {
            // 4. Executa a ação para cada item restante
            println!("docs[{index}]: {doc:?}\n");
        });

    println!("all_docs.len(): {:?}", all_docs.len());

    Ok(())
}
