use crate::{
    BUFFER_CAPACITY, Bloco0, DocsFiscais, EFDError, EFDResult, Informacoes, NEWLINE_BYTE,
    Registro0000, ResultExt, SpedContext, SpedFile, SpedRecord, extractor::process_block_lines,
    parser::parse_sped_fields,
};

use chrono::Datelike;
use claudiofsr_lib::{FileExtension, digit_count, get_style, open_file};
use encoding_rs::WINDOWS_1252;
use indicatif::{MultiProgress, ProgressBar};
use rayon::prelude::*;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    sync::Arc,
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

    // 3. Construção do Contexto (tabelas globais) passando o Bloco0.
    // Necessário processar sequencialmente o Bloco0 antes dos demais.
    // Ao final desta função, Bloco0 sairá de escopo e será LIBERADO da memória.
    // Passa bloco0 POR VALOR (move ownership).
    let context = Arc::new(SpedContext::new(sped_file.take_bloco_0(), arquivo)?);
    // println!("sped_context: {sped_context:?}");

    // Aqui, Bloco0 já foi destruído automaticamente pelo Rust!
    // A memória está limpa para o Rayon começar o trabalho pesado.

    // 4. Agora o sped_file (sem o Bloco0) vai para o Arc.
    // As threads do Rayon não carregarão o peso do Bloco0 morto.
    // Encapsula em Arc para compartilhamento barato entre threads
    let sped_file_arc = Arc::new(sped_file);

    // 5. Processamento dos Blocos em Paralelo
    // Definimos a ordem de blocos. Bloco 0 já foi processado no contexto.
    let blocks_to_process = ['A', 'C', 'D', 'F', 'I', 'M', '1'];

    // println!("processar blocos do arquivo EFD: {:?}", arquivo);

    // 1. Scatter (Paralelo)
    // O unzip() ou collect() aqui é muito eficiente e mantém a ordem dos blocos
    // unzip() separa os (Vec<Docs>, Vec<String>) em dois Vec<Vec<...>>
    let (docs_vecs, msgs_vecs): (Vec<Vec<DocsFiscais>>, Vec<Vec<String>>) = blocks_to_process
        .into_par_iter()
        .map(|bloco| process_block_lines(bloco, &sped_file_arc, &context))
        .unzip();

    // std::process::exit(0);

    // 2. Gather de Documentos (Sequencial e Otimizado)
    let total_docs: usize = docs_vecs.iter().map(|v| v.len()).sum();
    let mut all_docs = Vec::with_capacity(total_docs);
    for mut docs in docs_vecs {
        all_docs.append(&mut docs); // Move os ponteiros, zero cópia de dados
    }

    // 3. Gather de Mensagens (Sequencial e Otimizado)
    let total_msg_len: usize = msgs_vecs
        .iter()
        .flat_map(|v| v.iter().map(|s| s.len()))
        .sum();
    let mut all_messages = String::with_capacity(total_msg_len);
    for msgs in msgs_vecs {
        for msg in msgs {
            all_messages.push_str(&msg);
        }
    }

    // Enumerar todas as linhas
    // all_docs.par_iter_mut().enumerate().for_each(|(index, docs)| {docs.linhas = index + 2;});

    // 3. Agregação final das mensagens
    // Se o relatório existir, adicionamos ao final das mensagens acumuladas
    if let Some(registro_0111) = &context.registro_0111 {
        all_messages.push_str(&registro_0111.generate_report());
        all_messages.push('\n');
    }

    let cnpj_base = context
        .estabelecimento_cnpj_base
        .parse::<u32>()
        .map_loc(|e| EFDError::ParseIntError(e, context.estabelecimento_cnpj_base.to_string()))?;

    // Return the aggregated results.
    Ok(Informacoes {
        cnpj_base,
        periodo_de_apuracao: context.periodo_de_apuracao.unwrap_or_default(),
        messages: all_messages, // Mensagens acumuladas
        all_docs,
    })
}

/// Lê o arquivo e converte em estrutura SpedFile usando Rayon.
///
/// Padrão "Hybrid Iterator" (Sequencial para o Header -> Paralelo para o Body),
/// que é a forma mais eficiente de processar arquivos lineares onde o início dita o contexto.
pub fn read_and_parse_file(
    path: &Path,
    multiprogressbar: &MultiProgress,
    index: usize,
    total: usize,
) -> EFDResult<SpedFile> {
    let file = open_file(path).map_loc(|e| EFDError::InOut {
        source: e,
        path: path.to_path_buf(),
    })?;

    let file_number = index + 1;
    let (progressbar, number_of_lines) = initialize_progressbar(multiprogressbar, index, path)?;

    // Define a frequência de atualização da barra de progresso (ex: a cada 1%)
    let delta: u64 = (number_of_lines / 100).max(1);

    // Converte u64 para usize de forma segura uma única vez
    let delta_usize: usize = delta.try_into()?;

    // 1. ITERADOR SEQUENCIAL INICIAL
    let mut lines_iter = BufReader::with_capacity(BUFFER_CAPACITY, file)
        .split(NEWLINE_BYTE)
        .enumerate();

    // 2. CHAMADA DA FUNÇÃO EXTERNA (HEADER SEQUENCIAL)
    // Passamos &mut lines_iter para manter o estado do cursor.
    let mut sped_file =
        parse_header_sequentially(&mut lines_iter, path, &progressbar, file_number, total)?;

    // 3. PROCESSAMENTO PARALELO DO RESTANTE
    // O lines_iter agora começa exatamente da linha APÓS o 0000.
    let parallel_results = lines_iter
        // OTIMIZAÇÃO: Interrompe o iterador sequencial ANTES do paralelismo
        // scan(estado_inicial, closure with two arguments)
        .scan(false, |parar_proxima_iteracao, (idx, line_result)| {
            // Se a flag foi ativada na iteração anterior, mata o iterador AQUI.
            // Isso evita que qualquer byte da assinatura digital passe adiante.
            if *parar_proxima_iteracao {
                return None; // Para o iterador imediatamente
            }

            // .map_loc() é "lazy" [a closure só será executada se houver um erro de leitura]
            // path.to_path_buf() só será chamado se line_result for Err.
            let line_result = line_result.map_loc(|e| EFDError::InOutDetalhado {
                source: e,
                path: path.to_path_buf(),
                line_number: idx + 1,
            });

            // Verificamos o fim do arquivo aproveitando que o Result já está na mão
            if let Ok(ref bytes) = line_result {
                // Checagem ultra-rápida sem alocação (6 bytes na Stack)
                // Após Registro9999, consta na EFD a assinatura digital binária.
                if bytes.starts_with(b"|9999|") {
                    *parar_proxima_iteracao = true;
                }
            }

            Some((idx, line_result))
        })
        .par_bridge() // Transforma o iterador serial em paralelo. O Rayon agora só recebe linhas válidas até o |9999|
        .try_fold(
            // O "estado inicial" de cada thread é uma tupla: (O arquivo, O buffer de linha e do registro)
            || (SpedFile::new(), String::with_capacity(1024), [0u8; 4]),
            |state, (idx, line_result)| -> EFDResult<(SpedFile, String, [u8; 4])> {
                let (mut acc, mut line_buf, mut reg_buf) = state;
                let line_number = idx + 1;

                // O erro já foi envelopado no scan, aqui apenas propagamos com ?
                let line_bytes = line_result?;

                // Trim ASCII whitespace from both ends of the byte slice.
                let trimmed = line_bytes.trim_ascii();

                if !trimmed.is_empty() {
                    // Decode bytes to string, handling potential encoding issues.
                    // Reutiliza a String (Isso economiza MUITA memória)
                    get_string_utf8(trimmed, &mut line_buf, line_number, path)?;

                    // Faz o parse (CPU intensivo)
                    if let Some(record) =
                        parse_sped_fields(path, line_number, &line_buf, &mut reg_buf)?
                    {
                        acc.add_record(record);
                    }
                }

                // Incrementar ProgressoBar
                if line_number.is_multiple_of(delta_usize) {
                    progressbar.inc(delta);
                }

                // Retorna a tupla para a próxima iteração da mesma thread
                Ok((acc, line_buf, reg_buf))
            },
        )
        // REDUCER: Funde os resultados das threads
        .try_reduce(
            || (SpedFile::new(), String::new(), [0u8; 4]), // Identidade para a redução
            |mut main_tuple, thread_tuple| {
                // Unimos apenas os SpedFiles, o buffer da thread pode ser descartado
                main_tuple.0.merge(thread_tuple.0);
                Ok(main_tuple)
            },
        )?;

    // 4. MERGE FINAL: Unir o Header (sequencial) com o Body (paralelo)
    sped_file.merge(parallel_results.0);

    progressbar.finish();
    Ok(sped_file)
}

/// Processa o início do arquivo sequencialmente até encontrar o Registro 0000.
///
/// Retorna um `SpedFile` contendo os registros iniciais encontrados.
/// O iterador `lines_iter` é passado por referência mutável para que o cursor
/// permaneça na posição correta após o `break`.
fn parse_header_sequentially<I>(
    lines_iter: &mut I,
    path: &Path,
    progressbar: &ProgressBar,
    file_number: usize,
    total: usize,
) -> EFDResult<SpedFile>
where
    I: Iterator<Item = (usize, std::io::Result<Vec<u8>>)>,
{
    let mut sped_file = SpedFile::new();

    // ========================================================================
    // ALOCAÇÕES ÚNICAS (SCRATCHPADS)
    // Inicializados fora do loop para evitar alocações repetitivas na Heap/Stack.
    // ========================================================================
    let mut line_buf = String::with_capacity(1024);
    let mut reg_buf = [0u8; 4]; // Buffer fixo para normalização de registros (ex: c100 -> C100)

    for (idx, line_result) in lines_iter.by_ref() {
        let line_number = idx + 1;

        // .map_loc() é lazy e só processa o erro se ele ocorrer
        let bytes = line_result.map_loc(|e| EFDError::InOut {
            source: e,
            path: path.to_path_buf(),
        })?;

        // Trim ASCII whitespace (rápido, sem alocação)
        let trimmed = bytes.trim_ascii();

        if trimmed.is_empty() {
            continue;
        }

        // 1. Decodifica os bytes para a String (Reutiliza a memória de line_buf)
        get_string_utf8(trimmed, &mut line_buf, line_number, path)?;

        // 2. Faz o parse usando os buffers reutilizáveis
        // O vetor de campos (fields) será criado localmente dentro desta função,
        // o que é seguro e performático.
        if let Some(record) = parse_sped_fields(path, line_number, &line_buf, &mut reg_buf)? {
            // Lógica específica para identificar o Registro 0000 (Início da EFD)
            if let SpedRecord::Bloco0(boxed_bloco) = &record {
                // Desreferencia o Box para acessar o Enum Bloco0
                if let Bloco0::R0000(reg_0000) = boxed_bloco.as_ref() {
                    // Atualiza a descrição da barra de progresso com os dados do arquivo
                    update_progressbar_header(progressbar, reg_0000, file_number, total);

                    sped_file.add_record(record);
                    break; // Interrompe o processamento sequencial após encontrar o 0000
                }
            }

            // Adiciona registros que porventura venham antes do 0000
            sped_file.add_record(record);
        }
    }

    Ok(sped_file)
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
) -> EFDResult<(ProgressBar, u64)> {
    let file: File = open_file(arquivo)?;
    let number_of_lines: u64 = file.count_lines()?;

    let progressbar: ProgressBar =
        multiprogressbar.insert(index, ProgressBar::new(number_of_lines));
    let style = get_style(0, 0, 35).map_loc(|_| EFDError::InvalidStyle)?;
    progressbar.set_style(style);

    Ok((progressbar, number_of_lines))
}

/// Configurar o título de progressbar
fn update_progressbar_header(
    progressbar: &ProgressBar,
    reg_0000: &Registro0000,
    file_number: usize,
    total: usize,
) {
    let periodo = reg_0000.obter_periodo_de_apuracao();
    let msg = format!(
        "EFD Contribuição nº {:0>width$} de {mes:02}/{ano:04}",
        file_number,
        mes = periodo.month(),
        ano = periodo.year(),
        width = digit_count(total)
    );
    progressbar.set_message(msg);
}

/// Decodifica bytes para um buffer String pré-alocado (Scratchpad).
///
/// # Argumentos
/// * `slice_bytes` - Os bytes brutos da linha (sem o terminador \n).
/// * `buffer` - Referência mutável para a String que será limpa e preenchida.
/// * `line_number` - Para fins de relatório de erro.
/// * `path` - Caminho do arquivo para fins de relatório de erro.
pub fn get_string_utf8(
    slice_bytes: &[u8],
    buffer: &mut String,
    line_number: usize,
    path: &Path,
) -> EFDResult<()> {
    // 1. Limpa o conteúdo anterior sem liberar a capacidade alocada (reuso de memória).
    buffer.clear();

    // 2. FAST PATH: Tenta decodificar como UTF-8 (Padrão mais comum).
    // Usamos std::str::from_utf8 que é extremamente otimizado (muitas vezes via SIMD).
    match str::from_utf8(slice_bytes) {
        Ok(valid_str) => {
            // Se for UTF-8 válido, apenas copiamos para o buffer existente.
            buffer.push_str(valid_str);
        }
        Err(utf8_error) => {
            // 3. SLOW PATH: Fallback para WINDOWS-1252 (Padrão de arquivos SPED antigos).
            // Em vez de DecodeReaderBytesBuilder (que é para Streams), usamos
            // encoding_rs diretamente para processar o slice de bytes em memória.

            // O método decode() retorna um Cow<str>.
            // - Se o dado for apenas ASCII, ele não aloca.
            // - Se houver caracteres especiais, ele cria uma String temporária apenas para a conversão.
            let (decoded_res, _encoding_used, has_errors) = WINDOWS_1252.decode(slice_bytes);

            if has_errors {
                // Se nem o Windows-1252 conseguiu processar, emitimos erro estrutural.
                return Err(EFDError::Utf8DecodeError {
                    arquivo: path.to_path_buf(),
                    linha_num: line_number,
                    utf8_error,
                    // Criamos um erro sintético para o campo win_1252_error
                    win_1252_error: std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Falha crítica: Codificação não reconhecida (UTF-8/WIN1252)",
                    ),
                })
                .loc();
            }

            // Copia o resultado decodificado para o nosso scratchpad.
            buffer.push_str(&decoded_res);
        }
    }

    Ok(())
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
