use std::{
    error::Error, fs, io::Write, path::PathBuf, thread
};

use chrono::{NaiveDate, Datelike};
use indicatif::MultiProgress;
use rayon::prelude::*;

use crate::{
    analyze_one_file,
    args::Arguments,
    create_xlsx,
    make_dispatch_table,
    sped_efd,
    structures::analise_dos_creditos,
    structures::consolidacao_cst,
    DocsFiscais,
    DELIMITER_CHAR,
    OUTPUT_DIRECTORY,
    OUTPUT_FILENAME,
    MyResult,
};

type Informacoes = (u32, NaiveDate, String, Vec<DocsFiscais>);

pub fn executar_programa(
    args: &Arguments,
    mut write: &mut dyn Write,
) -> Result<Vec<DocsFiscais>, Box<dyn Error>> {

    if let Err(error) = make_dir_recursively(OUTPUT_DIRECTORY) {
        eprintln!("Não foi possível criar o diretório '{OUTPUT_DIRECTORY}'!");
        eprintln!("Error: {error}");
    }

    let (mut pa_total, all_lines) = analyze_all_files(&args.all_files, &mut write)?;

    let print_table: bool = pa_total.len() > 1;

    if print_table {
        pa_total.sort_by_key(|tuple| (tuple.year(), tuple.month()));
        let last_index = pa_total.len() - 1;
        let (pa_ano_first, pa_mes_first) = (pa_total[0].year(), pa_total[0].month());
        let (pa_ano_last,  pa_mes_last ) = (pa_total[last_index].year(), pa_total[last_index].month());
        write!  (write, "Período de Apuração Total ({} arquivos): ", pa_total.len())?;
        writeln!(write, "{pa_mes_first:02}/{pa_ano_first} a {pa_mes_last:02}/{pa_ano_last}")?;
    }

    let (consolidacao_cst, consolidacao_nat) = consolidar_resultados(&all_lines, print_table, &mut write)?;

    if let Err(error) = create_xlsx(&all_lines, &consolidacao_cst, &consolidacao_nat, &mut write) {
        eprintln!("create_xlsx error: {error}");
    }

    if args.print_csv {
        print_csv_file(&all_lines, &mut write)?;
    }

    Ok(all_lines)
}

// Função para criar diretório
fn make_dir_recursively (dir_name: &str) -> std::io::Result<()> {
    // Recursively create a directory and all of its parent components if they are missing.
    fs::create_dir_all(dir_name)?;
    Ok(())
}

fn analyze_all_files(
    arquivos_efd: &[PathBuf],
    mut write: &mut dyn Write,
) -> Result<(Vec<NaiveDate>, Vec<DocsFiscais>), Box<dyn Error>> {

    print_arquivos_selecionados(arquivos_efd, &mut write)?;

    let registros_efd = sped_efd::registros(); // tabela de registros
    let dispatch_table = make_dispatch_table()?;

    // indicatif ProgressBar + rayon
    let total: usize = arquivos_efd.len();
    let multiprogressbar: MultiProgress = MultiProgress::new();

    let mut all_info: Vec<Informacoes> = arquivos_efd
        .into_par_iter() // rayon: parallel iterator
        .enumerate()
        .map(|(index, arquivo)| {
            match analyze_one_file(
                &registros_efd,
                &dispatch_table,
                &multiprogressbar,
                arquivo,
                index,
                total,
            ) {
                Ok(informacoes) => informacoes,
                Err(error) => {
                    eprintln!("Erro na função analyze_one_file()!");
                    eprintln!("Arquivo: {}", arquivo.display());
                    panic!("Erro: {error}")
                },
            }
        })
        .filter(|(_cnpj_base, _pa, _info_msg, vec_docs_fiscais)| !vec_docs_fiscais.is_empty())
        .collect();

    // Ordenar all_info em função de cnpj_base e pa (periodo_de_apuracao)
    all_info.par_sort_by_key(|(cnpj_base, pa, _, _)| (cnpj_base.to_owned(), pa.year(), pa.month()));

    let (pa_total, vec_all_data): (Vec<NaiveDate>, Vec<Vec<DocsFiscais>>) = all_info
        .into_iter()
        .enumerate()
        .map(|(index, tuple)| {
            let (_cnpj_base, pa, info_msg, vec_docs_fiscais): Informacoes = tuple;

            writeln!(write, "EFD {:02}: {}", index + 1, vec_docs_fiscais[0].arquivo_efd)?;
            writeln!(write, "Período de Apuração: {:02}/{}\n", pa.month(), pa.year())?;
            write!  (write, "{info_msg}")?;

            consolidar_resultados(&vec_docs_fiscais, true, &mut write)?;

            Ok((pa, vec_docs_fiscais))
        })
        .collect::<Result<Vec<(NaiveDate, Vec<DocsFiscais>)>, Box<dyn Error>>>()?
        .into_par_iter()
        .unzip(); // Collect vec of tuples Vec<(T, U)> into two vecs Vec<T> and Vec<U>

    // Flatten a Vec<Vec<T>> to a Vec<T>
    let mut all_data: Vec<DocsFiscais> = vec_all_data
        .into_par_iter() // rayon: parallel iterator
        .flatten()
        .collect();

    update_line_counter(&mut all_data);

    Ok((pa_total, all_data))
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

fn print_arquivos_selecionados(
    arquivos: &[PathBuf],
    write: &mut dyn Write,
) -> Result<(), Box<dyn Error>> {

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
    database: &[DocsFiscais],
    print_table: bool,
    write: &mut dyn Write,
) -> Result<(Vec<consolidacao_cst::ConsolidacaoCST>, Vec<analise_dos_creditos::AnaliseDosCreditos>), Box<dyn Error>> {

    /*
    let (cst, nat) = rayon::join(
        || consolidacao::consolidar_operacoes_por_cst(database),
        || consolidacao::consolidar_natureza_da_base_de_calculo(database),
    );
    */

    /*
    // This creates the scope for the threads
    let (result_cst, result_nat) = thread::scope(|s| {

        let thread_cst = s.spawn(||consolidacao_cst::consolidar_operacoes_por_cst(database).unwrap());

        let thread_nat = s.spawn(||analise_dos_creditos::consolidar_natureza_da_base_de_calculo(database).unwrap());

        // Wait for background thread to complete
        (thread_cst.join(), thread_nat.join())
    });

    let (cst, nat) = match (result_cst, result_nat) {
        (Ok(cst), Ok(nat)) => (cst, nat),
        _ => panic!("Falha na Consolidação!"),
    };
    */

    let mut cst = (String::new(), Vec::new());
    let mut nat = (String::new(), String::new(), Vec::new());

    // This creates the scope for the threads
    thread::scope(|s| {
        s.spawn(|| -> MyResult<()> {
            cst = consolidacao_cst::consolidar_operacoes_por_cst(database)?;
            Ok(())
        });

        s.spawn(|| -> MyResult<()> { 
            nat = analise_dos_creditos::consolidar_natureza_da_base_de_calculo(database)?;
            Ok(())
        });
    });


    let (tabela_cst, consolidacao_cst) = cst;
    let (tabela_nat, tabela_rb, consolidacao_nat) = nat;

    if print_table {
        let title = "Receita Bruta Apurada e Segregada Conforme CST para Fins de Rateio dos Créditos";
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

fn print_csv_file(
    all_lines: &[DocsFiscais],
    write: &mut dyn Write,
) -> Result<(), Box<dyn Error>> {

    let mut csv_file: PathBuf = [OUTPUT_DIRECTORY, OUTPUT_FILENAME].iter().collect();
    csv_file.set_extension("csv");

    writeln!(write, "Write csv file: {:?}\n", csv_file.display())?;

    if let Err(err) = write_csv(all_lines, &csv_file) {
        panic!("Erro ao criar o arquivo {:?} com a função write_csv.\n'{err:?}'", csv_file.display());
    }

    Ok(())
}

// https://docs.rs/csv/1.0.0/csv/tutorial/index.html
// https://github.com/andrewleverette/rust_csv_examples/blob/master/src/bin/csv_write_serde.rs
fn write_csv(data: &[DocsFiscais], path: &PathBuf) -> Result<(), Box<dyn Error>> {

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

#[cfg(test)]
mod tests {
    use super::*;
    use claudiofsr_lib::{my_print, blake3_hash};

    // cargo test -- --help
    // cargo test -- --nocapture
    // cargo test -- --show-output

    #[test]
    fn test_executar_programa() -> Result<(), Box<dyn Error>> {
        // cargo test -- --show-output test_executar_programa

        let filename: &str = "examples/efd_data_random";
        let arquivo: PathBuf = PathBuf::from(filename);

        let mut write_buffer: Vec<u8> = vec!();
        let mut write: Box<&mut dyn Write> = Box::new(&mut write_buffer);

        let args = Arguments {
            all_files: vec![arquivo.clone()],
            clear_terminal: false,
            find: false,
            generator: None,
            print_csv: true,
            range: Some(vec![0]),
        };

        let all_lines = executar_programa(&args, &mut write)?;
        println!("all_lines:\n{all_lines:#?}");

        let output_name = [&args.get_app_name(), "-output.txt"].concat();
        let output_file: PathBuf = PathBuf::from(output_name);

        my_print(&write_buffer, output_file.clone())?;

        let csv_file: PathBuf = [OUTPUT_DIRECTORY, "Info do Contribuinte EFD Contribuicoes.csv"].iter().collect();

        let arq_file_hash = blake3_hash(&arquivo)?;
        let out_file_hash = blake3_hash(&output_file)?;
        let csv_file_hash = blake3_hash(&csv_file)?;

        assert_eq!(all_lines[0].descr_item, "MANTER DE 50ºC À 90ºC");
        assert_eq!(all_lines[1].descr_item, "“ASPAS”, SÍMBOLO EUROPEU (€) E TRAÇOS FANTASIA (– E —)");
        assert_eq!(all_lines[22].num_linha_efd, Some(121));
        assert_eq!(all_lines[22].registro, "C170");
        assert_eq!(all_lines[22].chave_doc, "74-3014-23.125.825/8364-49-12-016-867.204.387-416.648.086-8");
        assert_eq!(all_lines[22].valor_item.unwrap(), 27256.0);
        assert_eq!(all_lines[45].num_linha_efd, Some(193));
        assert_eq!(all_lines[45].valor_item.unwrap(), 58051.0);
        assert_eq!("2612118ea298d2365a30808e9e2227a7c210d9e91e6580ec3efc6ef875ca35c7", arq_file_hash);
        assert_eq!("62d84f3f6811fe651e25b4865a4d82f809fff192c5e03ee9a5aa777a21f6fa42", out_file_hash);
        assert_eq!("96c857355e6f41de233ec4094c89bca06f3b46fe54bbe65a596a8320beb77843", csv_file_hash);

        Ok(())
    }
}
