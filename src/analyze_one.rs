use crate::{
    DocsFiscais, EFDError, EFDResult, Informacoes, NEWLINE_BYTE, Registro0000, Registro0111,
    SpedContext, SpedFile, SpedRecord, extractor::process_block_lines, parser::parse_sped_fields,
};

use chrono::Datelike;
use claudiofsr_lib::{FileExtension, digit_count, get_style, open_file};
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;
use indicatif::{MultiProgress, ProgressBar};
use rayon::prelude::*;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
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
pub fn analyze_one_file(
    multiprogressbar: &MultiProgress,
    arquivo: &Path,
    index: usize,
    total: usize,
) -> EFDResult<Informacoes> {
    // 1. Leitura e Parsing do Arquivo (IO + CPU Paralelo)
    let mut sped_file = read_and_parse_file(arquivo, multiprogressbar, index, total)?;

    // 2. Ordenação (Essencial para garantir hierarquia Pai -> Filho)
    // Crucial: Garante que C100 venha antes de C170, mesmo após parse paralelo.
    // O processamento paralelo de leitura pode embaralhar as linhas dentro dos blocos.
    sped_file.sort_records_by_line_number();

    let registro_0111 = sped_file.obter_registro::<Registro0111>("0111")?;
    let relatorio_0111 = format!("{}\n", registro_0111.generate_report());

    // Encapsula em Arc para compartilhamento barato entre threads
    let sped_file_arc = Arc::new(sped_file);

    // 3. Construção do Contexto (Bloco 0 e tabelas globais)
    // Necessário processar sequencialmente o Bloco 0 antes dos demais.
    let sped_context = SpedContext::new(&sped_file_arc, arquivo)?;
    // println!("sped_context: {sped_context:?}");
    let context = Arc::new(sped_context);

    // 4. Processamento dos Blocos em Paralelo
    // Definimos a ordem de blocos. Bloco 0 já foi processado no contexto.
    let blocks_to_process = vec!['A', 'C', 'D', 'F', 'I', 'M', '1'];

    let (all_docs, mut all_messages): (Vec<DocsFiscais>, String) = blocks_to_process
        .into_par_iter()
        // 1. FOLD: Acumula resultados dentro de cada thread local
        .fold(
            // Estado inicial (Identidade)
            || (Vec::new(), String::new()),
            // Função de acumulação
            |(mut acc_docs, mut acc_msgs), bloco| {
                let (new_docs, new_msgs) = process_block_lines(bloco, &sped_file_arc, &context);

                acc_docs.extend(new_docs);

                // Concatena as mensagens do bloco na String local da thread
                for msg in new_msgs {
                    acc_msgs.push_str(&msg);
                }
                (acc_docs, acc_msgs)
            },
        )
        // 2. REDUCE: Junta os resultados das threads
        .reduce(
            // Identidade global (caso vazio)
            || (Vec::new(), String::new()),
            // Função de fusão de dois resultados parciais
            |(mut docs_a, mut msgs_a), (docs_b, msgs_b)| {
                docs_a.extend(docs_b);
                msgs_a.push_str(&msgs_b);
                (docs_a, msgs_a)
            },
        );

    // Enumerar todas as linhas
    // all_docs.par_iter_mut().enumerate().for_each(|(index, docs)| {docs.linhas = index + 2;});

    all_messages.push_str(&relatorio_0111);

    let cnpj_base = context
        .estabelecimento_cnpj_base
        .parse::<u32>()
        .map_err(|e| EFDError::ParseIntError(e, context.estabelecimento_cnpj_base.to_string()))?;

    // Return the aggregated results.
    Ok(Informacoes {
        cnpj_base,
        periodo_de_apuracao: context.periodo_de_apuracao.unwrap_or_default(),
        messages: all_messages, // Mensagens acumuladas
        all_docs,
    })
}

/// Lê o arquivo e converte em estrutura SpedFile usando Rayon.
fn read_and_parse_file(
    path: &Path,
    multiprogressbar: &MultiProgress,
    index: usize,
    total: usize,
) -> EFDResult<SpedFile> {
    let file = open_file(path).map_err(|e| EFDError::InOut {
        source: e,
        path: path.to_path_buf(),
    })?;

    let file_number = index + 1;
    let progressbar = initialize_progressbar(multiprogressbar, index, path)?;

    // Estrutura segura para escrita paralela
    let sped_file_data = Arc::new(Mutex::new(SpedFile::new()));

    // Processamento paralelo das linhas.
    BufReader::new(file)
        .split(NEWLINE_BYTE)
        .enumerate()
        .par_bridge() // Rayon parallel iterator, transforma iterador serial em paralelo
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

            // 1. Faz o parse (CPU intensivo)
            let record_opt = parse_sped_fields(path, line_number, &line)?;

            // 2. Atualiza a ProgressBar usando o objeto já parseado (Sem String Split/Parse)
            // Passamos 'as_ref()' para emprestar o record sem mover a ownership
            update_progressbar(
                &progressbar,
                record_opt.as_ref(),
                line_number,
                file_number,
                total,
            );

            // 3. Guarda no Mutex (Se for um registro válido)
            if let Some(sped_record) = record_opt {
                sped_file_data.lock().unwrap().add_record(sped_record);
            }

            // Verificação de fim de arquivo (Opcional, pois o parser já filtra)
            if line.starts_with("|9999|") {
                return Ok(());
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

/// Atualiza a mensagem e o incremento da ProgressBar.
///
/// OTIMIZAÇÃO: Recebe `Option<&SpedRecord>` em vez de `&str`.
/// Se o registro for "0000", faz cast direto e lê a data sem reprocessar string.
fn update_progressbar(
    progressbar: &ProgressBar,
    record: Option<&SpedRecord>,
    line_number: usize,
    file_number: usize,
    total: usize,
) {
    // 1. Atualizar Mensagem (Apenas se for Registro 0000)
    if let Some(r) = record {
        // Checagem barata de string antes de tentar o downcast
        if r.registro_name() == "0000" {
            // Tenta converter (downcast) para a struct concreta Registro0000
            if let Ok(reg_0000) = r.downcast_ref::<Registro0000>() {
                // Acesso direto ao campo dt_ini (NaiveDate) já parseado!
                let periodo_de_apuracao = reg_0000.obter_periodo_de_apuracao();
                let mes = periodo_de_apuracao.month();
                let ano = periodo_de_apuracao.year();
                let width = digit_count(total);

                let msg =
                    format!("EFD Contribuição nº {file_number:0>width$} de {mes:02}/{ano:04}");

                progressbar.set_message(msg);
            }
        }
    }

    // 2. Incrementar Progresso
    // Atualizar a cada 1000 linhas em processamento paralelo causa muita contenção no Atomic interno.
    // O usuário mal percebe a diferença visual entre 10 e 1000 em arquivos grandes.
    if line_number.is_multiple_of(1000) {
        progressbar.inc(1000);
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

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//
//
// cargo test -- --help
// cargo test -- --nocapture
// cargo test -- --show-output

/// Run tests with:
/// cargo test -- --show-output analyze_one_tests
#[cfg(test)]
#[path = "tests/analyze_one_tests.rs"]
mod analyze_one_tests;
