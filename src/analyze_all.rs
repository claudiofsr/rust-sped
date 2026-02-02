use chrono::{Datelike, NaiveDate};
use indicatif::MultiProgress;
use rayon::{join, prelude::*};
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use crate::{
    AppConfig, BUFFER_CAPACITY, DELIMITER_CHAR, DocsFiscais, EFDError, EFDResult, Informacoes,
    OUTPUT_DIRECTORY, ResultExt, TipoDeOperacao, analyze_one_file, create_xlsx,
    structures::{analise_dos_creditos, consolidacao_cst},
};

// ============================================================================
// Executar programa
// ============================================================================

pub fn executar_programa(config: &AppConfig, write: &mut dyn Write) -> EFDResult<Vec<DocsFiscais>> {
    // 1. Setup inicial (Sequencial)
    fs::create_dir_all(OUTPUT_DIRECTORY)
        .map_loc(|e| EFDError::DirectoryCreationFailed(OUTPUT_DIRECTORY.to_string(), e))?;

    // 2. Análise dos arquivos
    let (pa_total, all_lines) = analyze_all_files(config, write)?;

    // 3. Relatório do Período Total
    imprimir_resumo_periodo(&pa_total, write)?;

    // 4. Consolidação Global
    // Define se imprimirá as tabelas detalhadas baseado na quantidade de arquivos
    let print_table = pa_total.len() > 1;
    let (consolidacao_cst, consolidacao_nat) =
        consolidar_resultados(config, &all_lines, print_table, write)?;

    // 5. Filtragem (Pipeline Funcional)
    // Combinamos os filtros para maior clareza e eficiência
    let filtered_lines: Vec<DocsFiscais> = all_lines
        .into_par_iter()
        .filter(|doc| should_keep_record(doc, config))
        .collect();

    // 6. Preparação dos caminhos (Usando patterns funcionais)
    let path_csv = config.path_csv();
    let path_xlsx = config.path_xlsx();

    // Logs de intenção
    if !config.no_excel {
        writeln!(write, "Write Excel xlsx file: {:?}\n", path_xlsx.display())?;
    }
    if config.print_csv {
        writeln!(write, "Write csv file: {:?}\n", path_csv.display())?;
    }

    // 7. Execução paralela (Pipeline Funcional)
    // O join executa as duas closures. Se uma for "pulada", retorna Ok(()) instantaneamente.
    // O rayon::join permite capturar erros de ambos os lados
    let (res_xlsx, res_csv) = join(
        || {
            if config.no_excel {
                Ok(())
            } else {
                create_xlsx(
                    &path_xlsx,
                    &filtered_lines,
                    &consolidacao_cst,
                    &consolidacao_nat,
                )
            }
        },
        || {
            if config.print_csv {
                write_csv(&path_csv, &filtered_lines)
            } else {
                Ok(())
            }
        },
    );

    // 8. Verificamos se houve erro antes de escrever no output principal
    res_xlsx?;
    res_csv?;

    Ok(filtered_lines)
}

// ============================================================================
// Helpers & Predicados
// ============================================================================

/// Decide se um registro deve ser mantido com base nos argumentos de CLI.
/// Combina a lógica de exclusão de saídas e operações de crédito.
#[inline]
fn should_keep_record(doc: &DocsFiscais, config: &AppConfig) -> bool {
    // Se NÃO for operação de entrada/saída, aprovamos direto (Early Return com TRUE), mantemos DocsFiscais.
    if !doc.operacoes_de_entrada_ou_saida() {
        return true; // Note o "true" (MANTER)
    }

    // Regra 1: Se a flag estiver ativa E for operação de saída, removemos.
    if config.excluir_saidas && doc.tipo_de_operacao == Some(TipoDeOperacao::Saida) {
        return false;
    }

    // Regra 2: Se a flag estiver ativa E a natureza for None, removemos.
    if config.operacoes_de_creditos && doc.natureza_bc.is_none() {
        return false;
    }

    // Se sobreviveu aos filtros, mantém.
    true
}

/// Imprime o resumo do período de apuração total.
/// Otimização: Usa min/max (O(N)) em vez de sort (O(N log N)).
fn imprimir_resumo_periodo(dates: &[NaiveDate], write: &mut dyn Write) -> EFDResult<()> {
    // Só imprime se houver mais de um arquivo/data, conforme lógica original
    if dates.len() <= 1 {
        return Ok(());
    }

    // Encontra min e max em uma única periodossagem (ou duas O(N), muito mais rápido que sort)
    let min_date = dates.iter().min();
    let max_date = dates.iter().max();

    if let (Some(first), Some(last)) = (min_date, max_date) {
        writeln!(
            write,
            "Período de Apuração Total ({} arquivos): {:02}/{} a {:02}/{}\n",
            dates.len(),
            first.month(),
            first.year(),
            last.month(),
            last.year()
        )?;
    }

    Ok(())
}

// ============================================================================
// Analyze all Sped EFD files
// ============================================================================

fn analyze_all_files(
    config: &AppConfig,
    mut write: &mut dyn Write,
) -> EFDResult<(Vec<NaiveDate>, Vec<DocsFiscais>)> {
    let arquivos_efd: &[PathBuf] = &config.all_files;
    print_arquivos_selecionados(arquivos_efd, &mut write)?;

    // indicatif ProgressBar + rayon
    let total_files_count: usize = arquivos_efd.len();
    let multiprogressbar: MultiProgress = MultiProgress::new();

    // 1. Parsing e Análise (Mantido igual para permitir ordenação posterior)
    let mut all_info: Vec<Informacoes> = arquivos_efd
        .into_par_iter()
        .enumerate()
        // "Não dê mais de 1 arquivo para a mesma thread processar enquanto houver threads ociosas".
        .with_max_len(1)
        .filter_map(|(index, arquivo)| {
            // Executamos a análise
            match analyze_one_file(&multiprogressbar, arquivo, index, total_files_count) {
                // Condição de Sucesso:
                // Usa .then_some para converter o booleano diretamente em Option<Result>
                // "Se não estiver vazio, então retorna Some(Ok(info)), senão None"
                Ok(info) => (!info.all_docs.is_empty()).then_some(Ok(info)),

                // Condição de Erro:
                // Mapeamos para o erro customizado e mantemos (Some)
                Err(error) => Some(
                    Err(EFDError::AnalyzeFileError {
                        source: Box::new(error),
                        arquivo: arquivo.clone(),
                    })
                    .loc(),
                ),
            }
        })
        .collect::<EFDResult<Vec<Informacoes>>>()?;

    // 2. Ordenação (Necessária para garantir a ordem dos relatórios EFD 01, EFD 02...)
    all_info.par_sort_by_key(|info| {
        (
            info.cnpj_base,
            info.periodo_de_apuracao.year(),
            info.periodo_de_apuracao.month(),
        )
    });

    // 3. Otimização de RAM: Pré-calculamos a capacidade total necessária.
    // Isso evita que o vetor 'all_data' tenha que redimensionar (e copiar dados) várias vezes.
    let total_docs_count: usize = all_info.iter().map(|info| info.all_docs.len()).sum();

    // 4. Processamento Final (Relatórios + Achatamento) via try_fold
    // Retorna Result pois consolidar_resultados pode falhar

    let (pa_total, mut all_data) = all_info.into_iter().enumerate().try_fold(
        (
            Vec::with_capacity(total_files_count),
            Vec::with_capacity(total_docs_count),
        ),
        |(mut periodos, mut docs), (index, mut info)| -> EFDResult<_> {
            // Escreve o cabeçalho do arquivo atual diretamente no log de saída
            writeln!(
                write,
                "EFD {:02}: {}",
                index + 1,
                info.all_docs[0].arquivo_efd
            )?;
            writeln!(
                write,
                "Período de Apuração: {:02}/{}\n",
                info.periodo_de_apuracao.month(),
                info.periodo_de_apuracao.year()
            )?;

            write.write_all(info.messages.as_bytes())?;

            // Consolidação individual por arquivo
            consolidar_resultados(config, &info.all_docs, true, write)?;

            // Acumula os dados
            periodos.push(info.periodo_de_apuracao);

            // Otimização: 'append' move os elementos do vetor de origem para o destino.
            // Como 'info' será descartado, append é mais eficiente que 'extend'.
            docs.append(&mut info.all_docs);

            Ok((periodos, docs))
        },
    )?;

    // 4. PARALELO: Atualização final (O(N) rápido)
    update_line_counter(&mut all_data);

    Ok((pa_total, all_data))
}

// ============================================================================
// Funções Auxiliares (Helpers Funcionais)
// ============================================================================

/// Atualizar globalmente o contador de nº das linhas
fn update_line_counter(all_lines: &mut [DocsFiscais]) {
    all_lines
        .par_iter_mut() // rayon: parallel iterator
        .enumerate()
        .for_each(|(j, docs_fiscais)| {
            docs_fiscais.linhas = j + 2;
        });
}

fn print_arquivos_selecionados(arquivos: &[PathBuf], write: &mut dyn Write) -> EFDResult<()> {
    let number_of_files = arquivos.len();

    writeln!(write)?;

    if number_of_files == 1 {
        writeln!(write, "Analisar o arquivo SPED EFD:\n")?;
    } else {
        writeln!(write, "Analisar os {number_of_files} arquivos SPED EFD:\n")?;
    }

    for arquivo in arquivos {
        writeln!(write, "   {}", arquivo.display())?;
    }

    writeln!(write)?;

    Ok(())
}

fn consolidar_resultados(
    config: &AppConfig,
    database: &[DocsFiscais],
    print_table: bool,
    write: &mut dyn Write,
) -> EFDResult<(
    Vec<consolidacao_cst::ConsolidacaoCST>,
    Vec<analise_dos_creditos::AnaliseDosCreditos>,
)> {
    // Use rayon::join em vez de std::thread::scope.
    // O rayon::join permite "work-stealing", prevenindo que a thread fique
    // bloqueada inutilmente enquanto o pool está saturado.

    let (cst_result, nat_result) = rayon::join(
        || consolidacao_cst::consolidar_operacoes_por_cst(database),
        || analise_dos_creditos::consolidar_natureza_da_base_de_calculo(config, database),
    );

    /*
    // This creates the scope for the threads
    let (rresult_cst, rresult_nat) = thread::scope(|s| {

        let thread_cst = s.spawn(|| {
            consolidacao_cst::consolidar_operacoes_por_cst(database)
        });

        let thread_nat = s.spawn(|| {
            analise_dos_creditos::consolidar_natureza_da_base_de_calculo(database)
        });

        // Wait for background thread to complete
        (thread_cst.join(), thread_nat.join())
    });

    let (cst, nat) = match (rresult_cst, rresult_nat) {
        (Ok(result_cst), Ok(result_nat)) => {
            match (result_cst, result_nat) {
            (Ok(cst), Ok(nat)) => (cst, nat),
            _ => panic!("Falha na Consolidação!"),
            }
        },
        _ => panic!("thread fault!"),
    };
    */

    /*
    let mut cst = (String::new(), Vec::new());
    let mut nat = (String::new(), String::new(), Vec::new());

    // This creates the scope for the threads
    thread::scope(|s| {
        s.spawn(|| -> EFDResult<()> {
            cst = consolidacao_cst::consolidar_operacoes_por_cst(database)?;
            Ok(())
        });

        s.spawn(|| -> EFDResult<()> {
            nat = analise_dos_creditos::consolidar_natureza_da_base_de_calculo(config, database)?;
            Ok(())
        });
    });
    */

    let (tabela_cst, consolidacao_cst) = cst_result?;
    let (tabela_nat, tabela_rb, consolidacao_nat) = nat_result?;

    if print_table {
        let title =
            "Receita Bruta Apurada e Segregada Conforme CST para Fins de Rateio dos Créditos";
        writeln!(write, "{title}")?;
        writeln!(write, "{tabela_rb}\n")?;

        let title = "REGISTROS FISCAIS - CONSOLIDAÇÃO DAS OPERAÇÕES POR CST:";
        writeln!(write, "{title}")?;
        writeln!(write, "{tabela_cst}\n")?;

        let title = "Natureza da Base de Cálculo dos Créditos - CONSOLIDAÇÃO DAS OPERAÇÕES por Tipo de Crédito, CST e Alíquotas das Contribuições:";
        writeln!(write, "{title}")?;
        writeln!(write, "{tabela_nat}\n")?;
    }

    Ok((consolidacao_cst, consolidacao_nat))
}

/// Imprimir CSV com Writer interno e buffering para máxima eficiência de IO.
fn write_csv(path_csv: &Path, data: &[DocsFiscais]) -> EFDResult<()> {
    let file = File::create(path_csv).map_loc(|e| EFDError::InOut {
        source: e,
        path: path_csv.to_path_buf(),
    })?;

    let buffer = BufWriter::with_capacity(BUFFER_CAPACITY, file);

    let mut csv_builder = csv::WriterBuilder::new()
        .delimiter(DELIMITER_CHAR as u8)
        .has_headers(true)
        .quote_style(csv::QuoteStyle::NonNumeric) // Necessário para polars carregar arquivos csv corretamente!
        .from_writer(buffer);

    // When writing records without Serde, the header record is written just like any other record.
    let header_names = DocsFiscais::get_headers();
    csv_builder.write_record(&header_names)?;

    for docs_fiscais in data {
        csv_builder.write_record(docs_fiscais.get_values())?;
        //writer.serialize(docs_fiscais)?;
    }

    csv_builder.flush()?;
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
/// cargo test -- --show-output analyze_all
#[cfg(test)]
mod tests_analyze_all {
    use super::*;
    use claudiofsr_lib::{blake3_hash, my_print};
    use rust_decimal_macros::dec;

    // cargo test -- --help
    // cargo test -- --nocapture
    // cargo test -- --show-output test_executar_programa

    #[test]
    fn test_executar_programa() -> EFDResult<()> {
        let filename: &str = "examples/efd_data_random";
        let arquivo: PathBuf = PathBuf::from(filename);

        let mut write_buffer: Vec<u8> = vec![];
        let mut write: Box<&mut dyn Write> = Box::new(&mut write_buffer);

        let config = AppConfig {
            app_name: "efd_contribuicoes".to_string(),
            all_files: vec![arquivo.clone()],
            print_csv: true,
            ..Default::default()
        };

        let all_lines: Vec<DocsFiscais> = executar_programa(&config, &mut write)?;

        // Imprimir todos os registros.
        // println!("all_lines:\n{all_lines:#?}");

        // Imprimir os primeiros 5 registros de cada bloco.
        // chunk_by agrupa elementos consecutivos que satisfazem a condição.
        all_lines
            .chunk_by(|a, b| {
                // Otimização: comparar o primeiro byte é mais rápido que chars().next()
                a.registro.as_bytes().first() == b.registro.as_bytes().first()
            })
            .for_each(|chunk| {
                // Como o chunk não é vazio, pegamos o caractere do primeiro item
                let bloco = chunk[0].bloco();

                println!("=== Bloco {bloco} ===");

                // Esta parte interna é a forma mais idiomática de limitar os itens
                chunk
                    .iter()
                    .take(5)
                    .enumerate()
                    .for_each(|(i, doc)| println!("  {}. {:#?}", i + 1, doc));

                if chunk.len() > 5 {
                    println!("  ... e mais {} registros", chunk.len() - 5);
                }
            });

        let output_name = [&config.app_name, "-output.txt"].concat();
        let output_file: PathBuf = PathBuf::from(output_name);

        my_print(&write_buffer, &output_file)?;

        let csv_file: PathBuf = [
            OUTPUT_DIRECTORY,
            "Info do Contribuinte EFD Contribuicoes.csv",
        ]
        .iter()
        .collect();

        let arq_file_hash = blake3_hash(&arquivo)?;
        let out_file_hash = blake3_hash(&output_file)?;
        let csv_file_hash = blake3_hash(&csv_file)?;

        assert_eq!(all_lines[0].descr_item, "MANTER DE 50ºC À 90ºC".into());
        assert_eq!(
            all_lines[1].descr_item,
            "“ASPAS”, SÍMBOLO EUROPEU (€) E TRAÇOS FANTASIA (– E —)".into()
        );
        assert_eq!(all_lines[22].num_linha_efd, Some(121));
        assert_eq!(all_lines[22].registro, "C170".into());
        assert_eq!(
            all_lines[22].chave_doc,
            "74-3014-23.125.825/8364-49-12-016-867.204.387-416.648.086-8".into()
        );
        assert!(all_lines[22].valor_item.is_some_and(|v| v == dec!(27256.0)));
        assert_eq!(all_lines[45].num_linha_efd, Some(193));
        assert!(all_lines[45].valor_item.is_some_and(|v| v == dec!(58051.0)));
        assert_eq!(
            "2612118ea298d2365a30808e9e2227a7c210d9e91e6580ec3efc6ef875ca35c7",
            arq_file_hash
        );
        assert_eq!(
            "fbc27188e8702af8386969dfdaf515840aae2a4c46f5d9867a1eb3a5e7b99047",
            out_file_hash
        );
        assert_eq!(
            "b467412ecee458d1dac439a3d4af507d09455ecb9054301ed6e78f8e1b11fe20",
            csv_file_hash
        );

        Ok(())
    }
}
