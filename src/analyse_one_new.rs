use crate::{
    DocsFiscais, EFDError, EFDResult, Informacoes, NEWLINE_BYTE, SpedFile,
    info_new::{build_sped_context, process_block_lines},
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

use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;

/// Analisa um único arquivo EFD extraindo as informações para DocsFiscais.
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
    let context = Arc::new(build_sped_context(&sped_file_arc, arquivo)?);

    // 4. Processamento dos Blocos de Movimento em Paralelo
    // Definimos a ordem de blocos. Bloco 0 já foi processado no contexto.
    let blocks_to_process = vec!['A', 'C', 'D', 'F', 'I', 'M', 'P', '1', '9'];

    let result_vecs: Vec<Vec<DocsFiscais>> = blocks_to_process
        .par_iter()
        .map(|&bloco| {
            // Cada bloco processa seus registros de forma independente,
            // mas com acesso somente leitura ao Contexto e ao Arquivo Completo.
            process_block_lines(bloco, &sped_file_arc, &context)
        })
        .collect();

    // 5. Consolidação dos Resultados
    let all_docs: Vec<DocsFiscais> = result_vecs.into_iter().flatten().collect();

    let cnpj_base = context
        .cnpj_base
        .parse::<u32>()
        .map_err(|e| EFDError::ParseIntError(e, context.cnpj_base.clone()))?;

    Ok((
        cnpj_base,
        context.pa.unwrap_or_default(),
        context.messages.clone(), // Mensagens acumuladas
        all_docs,
    ))
}

/// Lê o arquivo e converte em estrutura SpedFile usando Rayon.
fn read_and_parse_file(
    arquivo: &Path,
    multiprogressbar: &MultiProgress,
    index: usize,
    _total: usize,
) -> EFDResult<SpedFile> {
    let file = open_file(arquivo)?;
    let progressbar = initialize_progressbar(multiprogressbar, index, arquivo)?;

    // Estrutura segura para escrita paralela
    let sped_file_data = Arc::new(Mutex::new(SpedFile::new()));

    // Leitura com decoding WINDOWS-1252 (Padrão SPED)
    let reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(WINDOWS_1252))
            .build(file),
    );

    // Processamento paralelo das linhas
    reader
        .split(NEWLINE_BYTE)
        .enumerate()
        .par_bridge()
        .try_for_each(|(idx, line_result)| -> EFDResult<()> {
            let line_number = idx + 1;
            let line_bytes = line_result.map_err(EFDError::Io)?;

            // Trim básico de bytes antes de string
            if line_bytes.trim_ascii().is_empty() {
                return Ok(());
            }

            // Conversão para String (já decodificado pelo DecodeReader, mas garantindo UTF8 final)
            let line = String::from_utf8(line_bytes).map_err(|e| {
                EFDError::Utf8DecodeError(
                    arquivo.to_path_buf(),
                    line_number,
                    e.utf8_error(),
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "UTF8 Error"),
                )
            })?;

            if line.trim().is_empty() {
                return Ok(());
            }

            // Parse da linha usando a lógica do parser.rs
            if let Some(record) = parse_sped_fields(arquivo, line_number, &line)? {
                let mut guard = sped_file_data.lock().unwrap();
                guard.add_record(record);
            }

            // Atualiza PB a cada X linhas para não travar mutex demais?
            // Na verdade, inc(1) no indicatif é thread-safe.
            if line_number % 100 == 0 {
                progressbar.inc(100);
            }

            Ok(())
        })?;

    progressbar.finish_with_message("Leitura concluída");

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
