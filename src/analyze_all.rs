use chrono::{Datelike, NaiveDate};
use indicatif::MultiProgress;
use rayon::prelude::*;
use std::{fs, io::Write, path::PathBuf, thread};

use crate::{
    DELIMITER_CHAR,
    DocsFiscais,
    EFDError,
    EFDResult,
    Informacoes,
    OUTPUT_DIRECTORY,
    OUTPUT_FILENAME,
    TipoOperacao,
    //analyze_one_file,
    analyze_one_file_new,
    args::Arguments,
    create_xlsx,
    //make_dispatch_table,
    //sped_efd,
    structures::{analise_dos_creditos, consolidacao_cst},
};

// ============================================================================
// Executar programa
// ============================================================================

pub fn executar_programa(args: &Arguments, write: &mut dyn Write) -> EFDResult<Vec<DocsFiscais>> {
    // 1. Setup inicial
    fs::create_dir_all(OUTPUT_DIRECTORY)
        .map_err(|e| EFDError::DirectoryCreationFailed(OUTPUT_DIRECTORY.to_string(), e))?;

    // 2. Análise dos arquivos
    let (pa_total, all_lines) = analyze_all_files(args, write)?;

    // 3. Relatório do Período Total
    imprimir_resumo_periodo(&pa_total, write)?;

    // 4. Consolidação Global
    // Define se imprimirá as tabelas detalhadas baseado na quantidade de arquivos
    let print_table: bool = pa_total.len() > 1;
    let (consolidacao_cst, consolidacao_nat) =
        consolidar_resultados(args, &all_lines, print_table, write)?;

    // 5. Filtragem (Pipeline Funcional)
    // Combinamos os filtros para maior clareza e eficiência
    let filtered_lines: Vec<DocsFiscais> = all_lines
        .into_par_iter()
        .filter(|doc| should_keep_record(doc, args))
        .collect();

    // 6. Exportação dos Dados
    create_xlsx(&filtered_lines, &consolidacao_cst, &consolidacao_nat, write)?;

    if args.print_csv {
        print_csv_file(&filtered_lines, write)?;
    }

    Ok(filtered_lines)
}

// ============================================================================
// Helpers & Predicados
// ============================================================================

/// Decide se um registro deve ser mantido com base nos argumentos de CLI.
/// Combina a lógica de exclusão de saídas e operações de crédito.
#[inline]
fn should_keep_record(doc: &DocsFiscais, args: &Arguments) -> bool {
    // Se NÃO for operação de entrada/saída, aprovamos direto (Early Return com TRUE), mantemos DocsFiscais.
    if !doc.operacoes_de_entrada_ou_saida() {
        return true; // Note o "true" (MANTER)
    }

    // Regra 1: Se a flag estiver ativa E for operação de saída, removemos.
    if args.excluir_saidas && doc.tipo_de_operacao == Some(TipoOperacao::Saida) {
        return false;
    }

    // Regra 2: Se a flag estiver ativa E a natureza for None, removemos.
    if args.operacoes_de_creditos && doc.natureza_bc.is_none() {
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

    // Encontra min e max em uma única passagem (ou duas O(N), muito mais rápido que sort)
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
    args: &Arguments,
    mut write: &mut dyn Write,
) -> EFDResult<(Vec<NaiveDate>, Vec<DocsFiscais>)> {
    let arquivos_efd: &[PathBuf] = &args.all_files;
    print_arquivos_selecionados(arquivos_efd, &mut write)?;

    //let registros_efd = sped_efd::registros(); // tabela de registros
    //let dispatch_table = make_dispatch_table()?;

    // indicatif ProgressBar + rayon
    let total: usize = arquivos_efd.len();
    let multiprogressbar: MultiProgress = MultiProgress::new();

    // 1. Parsing e Análise (Mantido igual para permitir ordenação posterior)
    let mut all_info: Vec<Informacoes> = arquivos_efd
        .into_par_iter()
        .enumerate()
        .filter_map(|(index, arquivo)| {
            // Executamos a análise
            match analyze_one_file_new(
                //&registros_efd,
                //&dispatch_table,
                &multiprogressbar,
                arquivo,
                index,
                total,
            ) {
                // Condição de Sucesso:
                // Usa .then_some para converter o booleano diretamente em Option<Result>
                // "Se não estiver vazio, então retorna Some(Ok(info)), senão None"
                Ok(info) => (!info.all_docs.is_empty()).then_some(Ok(info)),

                // Condição de Erro:
                // Mapeamos para o erro customizado e mantemos (Some)
                Err(error) => Some(Err(EFDError::AnalyzeFileError {
                    source: Box::new(error),
                    arquivo: arquivo.clone(),
                })),
            }
        })
        .map(|result| result.map(Into::into))
        .collect::<EFDResult<Vec<Informacoes>>>()?;

    // 2. Ordenação (Necessária para garantir a ordem dos relatórios EFD 01, EFD 02...)
    all_info.par_sort_by_key(|info| {
        (
            info.cnpj_base,
            info.periodo_de_apuracao.year(),
            info.periodo_de_apuracao.month(),
        )
    });

    // 3. Processamento Final (Relatórios + Achatamento) via Fold/Reduce
    // Retorna Result pois consolidar_resultados pode falhar
    let (pa_total, mut all_data, report_buffer) = all_info
        .into_par_iter()
        .enumerate()
        .map(|(index, info)| {
            // MAP: Prepara os dados e gera o relatório em memória local
            processar_relatorio_individual(index, info, args)
        })
        // TRY_FOLD: Acumula resultados localmente na thread (tratando erros)
        .try_fold(|| (Vec::new(), Vec::new(), Vec::new()), acumular_resultados)
        // TRY_REDUCE: Funde os resultados das threads
        .try_reduce(|| (Vec::new(), Vec::new(), Vec::new()), fundir_resultados)?;

    // 4. Saída Final (IO) e Atualização
    write.write_all(&report_buffer)?; // Escreve todos os relatórios de uma vez
    update_line_counter(&mut all_data);

    Ok((pa_total, all_data))
}

// ============================================================================
// Funções Auxiliares (Helpers Funcionais)
// ============================================================================

/// Gera o buffer de texto e prepara os dados de um único arquivo EFD.
fn processar_relatorio_individual(
    index: usize,
    info: Informacoes,
    args: &Arguments,
) -> EFDResult<(Vec<NaiveDate>, Vec<DocsFiscais>, Vec<u8>)> {
    let mut buffer = Vec::new();

    // Escreve no buffer de memória (Vec<u8>) em vez de direto no disco
    writeln!(
        buffer,
        "EFD {:02}: {}",
        index + 1,
        info.all_docs[0].arquivo_efd
    )?;
    writeln!(
        buffer,
        "Período de Apuração: {:02}/{}\n",
        info.periodo_de_apuracao.month(),
        info.periodo_de_apuracao.year()
    )?;
    write!(buffer, "{}", info.messages)?;

    // Consolida resultados escrevendo no buffer local
    consolidar_resultados(args, &info.all_docs, true, &mut buffer)?;

    Ok((vec![info.periodo_de_apuracao], info.all_docs, buffer))
}

/// Acumula os dados dentro de uma thread (Fold).
fn acumular_resultados(
    mut acc: (Vec<NaiveDate>, Vec<DocsFiscais>, Vec<u8>),
    res: EFDResult<(Vec<NaiveDate>, Vec<DocsFiscais>, Vec<u8>)>,
) -> EFDResult<(Vec<NaiveDate>, Vec<DocsFiscais>, Vec<u8>)> {
    let (pa, docs, buf) = res?; // Propaga erro se houver
    acc.0.extend(pa);
    acc.1.extend(docs);
    acc.2.extend(buf);
    Ok(acc)
}

/// Funde os dados de duas threads (Reduce).
fn fundir_resultados(
    mut a: (Vec<NaiveDate>, Vec<DocsFiscais>, Vec<u8>),
    b: (Vec<NaiveDate>, Vec<DocsFiscais>, Vec<u8>),
) -> EFDResult<(Vec<NaiveDate>, Vec<DocsFiscais>, Vec<u8>)> {
    a.0.extend(b.0);
    a.1.extend(b.1);
    a.2.extend(b.2);
    Ok(a)
}

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
    args: &Arguments,
    database: &[DocsFiscais],
    print_table: bool,
    write: &mut dyn Write,
) -> EFDResult<(
    Vec<consolidacao_cst::ConsolidacaoCST>,
    Vec<analise_dos_creditos::AnaliseDosCreditos>,
)> {
    /*
    let (cst, nat) = rayon::join(
        || consolidacao::consolidar_operacoes_por_cst(database),
        || consolidacao::consolidar_natureza_da_base_de_calculo(database),
    );
    */

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

    let mut cst = (String::new(), Vec::new());
    let mut nat = (String::new(), String::new(), Vec::new());

    // This creates the scope for the threads
    thread::scope(|s| {
        s.spawn(|| -> EFDResult<()> {
            cst = consolidacao_cst::consolidar_operacoes_por_cst(database)?;
            Ok(())
        });

        s.spawn(|| -> EFDResult<()> {
            nat = analise_dos_creditos::consolidar_natureza_da_base_de_calculo(args, database)?;
            Ok(())
        });
    });

    let (tabela_cst, consolidacao_cst) = cst;
    let (tabela_nat, tabela_rb, consolidacao_nat) = nat;

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

fn print_csv_file(all_lines: &[DocsFiscais], write: &mut dyn Write) -> EFDResult<()> {
    let mut csv_file: PathBuf = [OUTPUT_DIRECTORY, OUTPUT_FILENAME].iter().collect();
    csv_file.set_extension("csv");

    writeln!(write, "Write csv file: {:?}\n", csv_file.display())?;

    if let Err(err) = write_csv(all_lines, &csv_file) {
        panic!(
            "Erro ao criar o arquivo {:?} com a função write_csv.\n'{err:?}'",
            csv_file.display()
        );
    }

    Ok(())
}

// https://docs.rs/csv/1.0.0/csv/tutorial/index.html
// https://github.com/andrewleverette/rust_csv_examples/blob/master/src/bin/csv_write_serde.rs
fn write_csv(data: &[DocsFiscais], path: &PathBuf) -> EFDResult<()> {
    // Open a file in write-only mode, returns `io::Result<File>`
    let file = match fs::File::create(path) {
        Ok(file) => file,
        Err(error) => panic!("Couldn't create {:?}: {error}", path.display()),
    };

    let mut writer = csv::WriterBuilder::new()
        .delimiter(DELIMITER_CHAR as u8)
        .has_headers(true) // write the header
        .quote_style(csv::QuoteStyle::NonNumeric) // Necessário para polars carregar arquivos csv corretamente!
        .from_writer(file);

    // When writing records without Serde, the header record is written just like any other record.
    let header_names = DocsFiscais::get_headers();
    writer.write_record(&header_names)?;

    for docs_fiscais in data {
        writer.write_record(docs_fiscais.get_values())?;
        //writer.serialize(docs_fiscais)?;
    }

    writer.flush()?;

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

    // cargo test -- --help
    // cargo test -- --nocapture
    // cargo test -- --show-output

    #[test]
    fn test_executar_programa() -> EFDResult<()> {
        // cargo test -- --show-output test_executar_programa

        let filename: &str = "examples/efd_data_random";
        let arquivo: PathBuf = PathBuf::from(filename);

        let mut write_buffer: Vec<u8> = vec![];
        let mut write: Box<&mut dyn Write> = Box::new(&mut write_buffer);

        let args = Arguments {
            all_files: vec![arquivo.clone()],
            clear_terminal: false,
            excluir_saidas: false,
            excluir_cst_49: false,
            find: false,
            generator: None,
            print_csv: true,
            range: Some(vec![0]),
            operacoes_de_creditos: false,
        };

        let all_lines = executar_programa(&args, &mut write)?;
        println!("all_lines:\n{all_lines:#?}");

        let output_name = [&args.get_app_name(), "-output.txt"].concat();
        let output_file: PathBuf = PathBuf::from(output_name);

        my_print(&write_buffer, output_file.clone())?;

        let csv_file: PathBuf = [
            OUTPUT_DIRECTORY,
            "Info do Contribuinte EFD Contribuicoes.csv",
        ]
        .iter()
        .collect();

        let arq_file_hash = blake3_hash(&arquivo)?;
        let out_file_hash = blake3_hash(&output_file)?;
        let csv_file_hash = blake3_hash(&csv_file)?;

        assert_eq!(all_lines[0].descr_item, "MANTER DE 50ºC À 90ºC");
        assert_eq!(
            all_lines[1].descr_item,
            "“ASPAS”, SÍMBOLO EUROPEU (€) E TRAÇOS FANTASIA (– E —)"
        );
        assert_eq!(all_lines[22].num_linha_efd, Some(121));
        assert_eq!(all_lines[22].registro, "C170");
        assert_eq!(
            all_lines[22].chave_doc,
            "74-3014-23.125.825/8364-49-12-016-867.204.387-416.648.086-8"
        );
        assert_eq!(all_lines[22].valor_item.unwrap(), 27256.0);
        assert_eq!(all_lines[45].num_linha_efd, Some(193));
        assert_eq!(all_lines[45].valor_item.unwrap(), 58051.0);
        assert_eq!(
            "2612118ea298d2365a30808e9e2227a7c210d9e91e6580ec3efc6ef875ca35c7",
            arq_file_hash
        );
        assert_eq!(
            "4a4155b9c1b89ec310dbf9da0bc29ecf59407caa8a94bc3f927fa5f2d8dfb7c2",
            out_file_hash
        );
        assert_eq!(
            "b467412ecee458d1dac439a3d4af507d09455ecb9054301ed6e78f8e1b11fe20",
            csv_file_hash
        );

        Ok(())
    }
}
