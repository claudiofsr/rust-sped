use crate::{
    DocsFiscais, EFDError, EFDResult, Informacoes, NEWLINE_BYTE, SpedContext, SpedFile,
    get_string_utf8, info_new::process_block_lines, parser::parse_sped_fields,
};

use claudiofsr_lib::{FileExtension, get_style, open_file};
use indicatif::{MultiProgress, ProgressBar};
use rayon::prelude::*;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    sync::{Arc, Mutex},
};

/// Analisa um único arquivo EFD, extraindo informações e gerando
/// documentos fiscais, DocsFiscais.
///
/// O processo é dividido em:
/// 1. Leitura e Parsing Paralelo do arquivo para memória (`SpedFile`).
/// 2. Ordenação dos registros para garantir integridade hierárquica.
/// 3. Criação do Contexto (Lookup Tables) baseado no Bloco 0.
/// 4. Processamento Paralelo dos Blocos de Movimento (A, C, D, F, I, M, P, 1, 9).
pub fn analyze_one_file_new(
    multiprogressbar: &MultiProgress,
    arquivo: &Path,
    index: usize,
    total: usize,
) -> EFDResult<Informacoes> {
    // 1. Leitura e Parsing do Arquivo (IO + CPU Paralelo)
    let mut sped_file = read_and_parse_file(arquivo, multiprogressbar, index, total)?;

    // 2. Ordenação (Essencial para garantir hierarquia Pai -> Filho)
    // O processamento paralelo de leitura pode embaralhar as linhas dentro dos blocos.
    sped_file.sort_records_by_line_number();

    // Encapsula em Arc para compartilhamento barato entre threads
    let sped_file_arc = Arc::new(sped_file);

    // 3. Construção do Contexto (Bloco 0 e tabelas globais)
    // Necessário processar sequencialmente o Bloco 0 antes dos demais.
    let sped_context = SpedContext::new(&sped_file_arc, arquivo)?;
    println!("sped_context: {sped_context:?}");
    let context = Arc::new(sped_context);

    // 4. Processamento dos Blocos de Movimento em Paralelo
    // Definimos a ordem de blocos. Bloco 0 já foi processado no contexto.
    let blocks_to_process = vec!['A', 'C', 'D', 'F', 'I', 'M', 'P', '1', '9'];

    let all_docs: Vec<DocsFiscais> = blocks_to_process
        .par_iter()
        .flat_map_iter(|&bloco| {
            // O retorno aqui é tratado como iterador serial
            process_block_lines(bloco, &sped_file_arc, &context)
        })
        .collect();

    let cnpj_base = context
        .estabelecimento_cnpj_base
        .parse::<u32>()
        .map_err(|e| EFDError::ParseIntError(e, context.estabelecimento_cnpj_base.clone()))?;

    // Return the aggregated results.
    Ok(Informacoes {
        cnpj_base,
        periodo_de_apuracao: context.periodo_de_apuracao.unwrap_or_default(),
        messages: context.messages.join("\n"), // Mensagens acumuladas
        all_docs,
    })
}

/// Lê o arquivo e converte em estrutura SpedFile usando Rayon.
fn read_and_parse_file(
    path: &Path,
    multiprogressbar: &MultiProgress,
    index: usize,
    _total: usize,
) -> EFDResult<SpedFile> {
    let file = open_file(path).map_err(|e| EFDError::InOut {
        source: e,
        path: path.to_path_buf(),
    })?;
    let progressbar = initialize_progressbar(multiprogressbar, index, path)?;

    // Estrutura segura para escrita paralela
    let sped_file_data = Arc::new(Mutex::new(SpedFile::new()));

    // Processamento paralelo das linhas.
    BufReader::new(file)
        .split(NEWLINE_BYTE)
        .enumerate()
        .par_bridge() // Rayon parallel iterator
        .try_for_each(|(idx, line_result)| -> EFDResult<()> {
            let line_number = idx + 1;
            let line_bytes: Vec<u8> = line_result.map_err(|e| EFDError::InOut {
                source: e,
                path: path.to_path_buf(),
            })?;

            // Trim ASCII whitespace from both ends of the byte slice.
            let trimmed_bytes = line_bytes.trim_ascii();

            // Trim básico de bytes antes de string
            if trimmed_bytes.is_empty() {
                // If line is empty, skip this line
                return Ok(());
            }

            // Decode bytes to string, handling potential encoding issues.
            let line = get_string_utf8(trimmed_bytes, line_number, path)?;

            // Parse da linha usando a lógica do parser.rs
            if let Some(sped_record) = parse_sped_fields(path, line_number, &line)? {
                sped_file_data.lock().unwrap().add_record(sped_record);
            }

            // Atualiza PB a cada X linhas para não travar mutex demais?
            // Na verdade, inc(1) no indicatif é thread-safe.
            if line_number % 10 == 0 {
                progressbar.inc(10);
            }

            Ok(())
        })?;

    progressbar.finish(); // Finalize the progress bar.

    // Retorna o SpedFile "desembrulhado"
    let final_sped_file = Arc::try_unwrap(sped_file_data)
        .expect("Falha ao desembrulhar Arc do SpedFile")
        .into_inner()
        .expect("Falha ao obter Mutex do SpedFile");

    Ok(final_sped_file)
}

/// Initializes and configures an `indicatif::ProgressBar` for a given file.
///
/// # Arguments
/// * `multiprogressbar` - The `MultiProgress` instance to add the progress bar to.
/// * `index` - The index of the file in the batch (used for insertion order).
/// * `arquivo` - The path to the file, used to get the total number of lines.
///
/// # Returns
/// An `EFDResult<ProgressBar>` on success, or `EFDError` if style cannot be retrieved or file cannot be opened.
fn initialize_progressbar(
    multiprogressbar: &MultiProgress,
    index: usize,
    arquivo: &Path,
) -> EFDResult<ProgressBar> {
    let file: File = open_file(arquivo)?;
    let number_of_lines: u64 = file.count_lines()?;

    let progressbar: ProgressBar =
        multiprogressbar.insert(index, ProgressBar::new(number_of_lines));
    let style = get_style(0, 0, 35).map_err(|_| EFDError::InvalidStyle)?;
    progressbar.set_style(style);

    Ok(progressbar)
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
mod analyze_one_tests {
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

        let path = if let Some(p) = efd_files.get(15) {
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
            .filter(|(_, doc)| doc.aliq_cofins.is_some_and(|v| v > 0.0)) // 2. Filtra apenas os que atendem a condição
            .take(10) // 3. Pega apenas os primeiros 10 resultados que passaram no filtro
            .for_each(|(index, doc)| {
                // 4. Executa a ação para cada item restante
                println!("docs[{index}]: {doc:?}\n");
            });

        println!("all_docs.len(): {:?}", all_docs.len());

        Ok(())
    }
}
