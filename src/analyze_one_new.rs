use crate::{
    DocsFiscais, EFDError, EFDResult, Informacoes, NEWLINE_BYTE, SpedFile, get_string_utf8,
    info_new::{SpedContext, process_block_lines},
    parser::parse_sped_fields,
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
        .cnpj_base
        .parse::<u32>()
        .map_err(|e| EFDError::ParseIntError(e, context.cnpj_base.clone()))?;

    // Return the aggregated results.
    Ok((
        cnpj_base,
        context.pa.unwrap_or_default(),
        context.messages.join("\n"), // Mensagens acumuladas
        all_docs,
    ))
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
            if line_number % 100 == 0 {
                progressbar.inc(100);
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
