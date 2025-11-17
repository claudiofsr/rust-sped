use std::{
    borrow::Cow,
    collections::HashMap,
    fs::File,
    io::BufReader,
    num::ParseFloatError,
    ops::Deref,
    path::{Path, PathBuf},
    str,
};

use indicatif::{MultiProgress, ProgressBar};

use chrono::{Datelike, NaiveDate};
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::{DecodeReaderBytes, DecodeReaderBytesBuilder};

use claudiofsr_lib::{
    CST_ALL, FileExtension, StrExtension, get_naive_date, get_style, num_digits, open_file,
};

// use memmap2::Mmap;

use crate::{
    ALIQ_BASICA_COF, ALIQ_BASICA_PIS, DECIMAL_ALIQ, DECIMAL_VALOR, DocsFiscais, EFDError,
    EFDLineIterator, EFDResult, Info, Informacoes, REGEX_REMOVE_NON_DIGITS, Tipo, cred_presumido,
    obter_cod_da_natureza_da_bc, obter_modelo_do_documento_fiscal, obter_tipo_do_item,
    registros_de_operacoes,
};

// Tipo utilizado em fn make_dispatch_table()
type FuncaoLerRegistro = fn(&mut Info, HashMap<String, String>) -> EFDResult<()>;

/// Analyzes a single EFD file and extracts all relevant information.
pub fn analyze_one_file(
    registros_efd: &HashMap<&'static str, HashMap<u16, (&'static str, Tipo)>>,
    dispatch_table: &HashMap<&str, FuncaoLerRegistro>,
    multiprogressbar: &MultiProgress,
    arquivo: &Path,
    index: usize,
    total: usize,
) -> EFDResult<Informacoes> {
    // Step 1: Read and initially process file lines, accumulating basic info.
    let mut info: Info = read_and_process_file_lines(
        registros_efd,
        dispatch_table,
        multiprogressbar,
        arquivo,
        index,
        total,
    )?;

    // Step 2: Further parse the collected information into a structured format (`DocsFiscais`).
    let vec_docs_fiscais: Vec<DocsFiscais> = parse_file_info(&mut info)?;

    // Return the aggregated results.
    Ok((
        info.cnpj_base,
        info.pa.ok_or(EFDError::InvalidPA)?,
        info.messages,
        vec_docs_fiscais,
    ))
}

/// Reads and processes an EFD (Escrituração Fiscal Digital) file line by line.
///
/// This function handles:
/// - Opening the file.
/// - Initializing and updating a progress bar.
/// - Iterating through lines using `EFDLineIterator`.
/// - Dispatching each valid record to its specific handler function.
/// - Accumulating high-level information in the `Info` struct.
///
/// Critical errors will halt processing and be propagated.
///
/// # Arguments
/// * `registros_efd` - A reference to a `HashMap` defining the EFD schema.
/// * `dispatch_table` - A `HashMap` mapping record identifiers to their processing functions.
/// * `multiprogressbar` - A reference to an `indicatif::MultiProgress` instance.
/// * `arquivo` - A reference to the `Path` of the EFD file.
/// * `index` - The 0-based index of this file in a batch.
/// * `total` - The total number of files in the batch.
///
/// # Returns
/// An `EFDResult<Info>`:
/// * `Ok(info)`: Contains `Info` structure with aggregated data if successful.
/// * `Err(e)`: An `EFDError` indicating a critical issue.
fn read_and_process_file_lines(
    registros_efd: &HashMap<&'static str, HashMap<u16, (&'static str, Tipo)>>,
    dispatch_table: &HashMap<&str, FuncaoLerRegistro>,
    multiprogressbar: &MultiProgress,
    arquivo: &Path,
    index: usize,
    total: usize,
) -> EFDResult<Info> {
    let file_number = index + 1;
    let num_len = num_digits(total);

    let mut info = Info::new(arquivo);
    let mut progressbar = initialize_progressbar(multiprogressbar, index, arquivo)?;

    // Flag to ensure the progress bar message is set only once
    // when the period of 'apuracao' becomes available.
    let mut empty_msg_for_progressbar: bool = true;

    let file = File::open(arquivo)?; // Propagates any I/O error.
    let reader = BufReader::new(file);

    let mut efd_lines = EFDLineIterator::new(reader, arquivo, registros_efd);

    // Iterate over valid, processed lines, dispatching them for specific record handling.
    // `try_for_each` stops on the first `Err` and propagates it.
    efd_lines.try_for_each(|processed_line_result| -> EFDResult<()> {
        let processed_line = processed_line_result?; // Unpack `Ok` or propagate `EFDError`.

        let registro: &str = processed_line.fields[0].as_str();

        // If a handler function exists for the current record type, call it.
        if let Some(&ler_registro) = dispatch_table.get(registro) {
            // Extract and format field values into a HashMap.
            let valores: HashMap<String, String> = obter_valores(
                registros_efd,
                &processed_line.fields,
                processed_line.line_number,
                arquivo,
            )?;
            // Dispatch to the specific record processing function.
            ler_registro(&mut info, valores)?;
        }

        // Update the progress bar after processing each line.
        update_progressbar(
            &info,
            &mut progressbar,
            file_number,
            num_len,
            &mut empty_msg_for_progressbar,
        );

        Ok(())
    })?; // The `?` here propagates the first `EFDError` from `try_for_each`.

    progressbar.finish(); // Finalize the progress bar.
    Ok(info) // Return the accumulated information.
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
    let mut file: File = open_file(arquivo)?;
    let number_of_lines: u64 = file.count_lines()?;

    let progressbar: ProgressBar =
        multiprogressbar.insert(index, ProgressBar::new(number_of_lines));
    let style = get_style(0, 0, 35).map_err(|_| EFDError::InvalidStyle)?;
    progressbar.set_style(style);

    Ok(progressbar)
}

/// Updates the progress bar message with formatted file and period information.
/// Sets `empty_msg_flag` to `false` once the period of apuração is available.
///
/// # Arguments
/// * `info` - The `Info` struct containing file processing data.
/// * `progressbar` - The mutable `ProgressBar` to update.
/// * `file_number` - The 1-based index of the current file.
/// * `num_len` - The number of digits needed to format `total` files (for alignment).
/// * `empty_msg_flag` - A mutable boolean to track if the message has been set.
fn update_progressbar(
    info: &Info,
    progressbar: &mut ProgressBar,
    file_number: usize,
    num_len: usize,
    empty_msg: &mut bool,
) {
    if let (true, Some(periodo_de_apuracao)) = (&empty_msg, info.pa) {
        // Extract month and year from PeriodoDeApuracao
        let mes: u32 = periodo_de_apuracao.month();
        let ano: i32 = periodo_de_apuracao.year();

        // Format the progress bar message
        let msg: String =
            format!("EFD Contributions nº {file_number:>num_len$} de {mes:02}/{ano:04}");
        // Update the progress bar message
        progressbar.set_message(msg);
        // Set empty_msg to false
        *empty_msg = false;
    }
    // Increment the progress bar's count for this line.
    progressbar.inc(1);
}

/// Better to use [get_string_utf8()] function as it has been found in some files that
/// sometimes one line is encoded in UTF8 but another line is encoded in windows_1252!
///
/// <https://stackoverflow.com/questions/64040851/how-can-i-read-a-non-utf8-file-line-by-line-in-rust>
#[allow(dead_code)]
pub fn get_bufreader(file: File) -> BufReader<DecodeReaderBytes<File, Vec<u8>>> {
    BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(WINDOWS_1252))
            .build(file),
    )
}

/// Converts a slice of raw fields into a `HashMap` of field names to formatted values.
///
/// This function validates record and field existence against the EFD schema,
/// and applies decimal formatting based on `Tipo` (Valor, Aliquota, C, N).
///
/// # Type Parameters
/// * `T` - Any type that can be dereferenced to `str` and is `Debug` (e.g., `String`, `&str`).
///
/// # Arguments
/// * `registros_efd` - The EFD schema definition.
/// * `campos` - A slice of raw field strings from a line.
/// * `num_line` - The 1-based line number for error reporting.
/// * `arquivo` - The path to the file for error reporting.
///
/// # Returns
/// An `EFDResult<HashMap<String, String>>` on success, or an `EFDError` if:
/// * The record type is not found in `registros_efd`.
/// * A field index for the record is undefined.
/// * A numeric value fails to parse (handled by `formatar_casas_decimais`).
pub fn obter_valores<T>(
    registros_efd: &HashMap<&'static str, HashMap<u16, (&'static str, Tipo)>>,
    campos: &[T],
    num_line: usize,
    arquivo: &Path,
) -> EFDResult<HashMap<String, String>>
where
    T: Deref<Target = str> + std::fmt::Debug,
{
    // The first field `campos[0]` is the record identifier.
    let registro: &str = &campos[0];

    // Retrieve the field definitions for the current record type from the schema.
    let registros = registros_efd
        .get(registro)
        .ok_or_else(|| EFDError::Other(format!("Registro {registro} não encontrado")))?;

    // Initialize the HashMap with a pre-allocated capacity for efficiency.
    let mut valores: HashMap<String, String> = HashMap::with_capacity(campos.len() + 2);
    valores.insert("linha_da_efd".to_string(), num_line.to_string());
    valores.insert("nivel".to_string(), registros[&0].0.to_string());

    for (index, valor) in campos.iter().enumerate() {
        let index = (index + 1) as u16;

        let Some(&(campo, tipo)) = registros.get(&index) else {
            return Err(EFDError::Other(format!(
                "Erro: Index {index} do registro '{registro}' não definido"
            )));
        };

        // Clean up multiple whitespaces and then format decimal values as required.
        let valor_cleaned: String = valor.replace_multiple_whitespaces();
        let valor_formatted: String =
            formatar_casas_decimais(valor_cleaned.into(), tipo, num_line, arquivo)?;

        valores.insert(campo.to_string(), valor_formatted);
    }

    // println!("valores: {:#?}\n", &valores);

    Ok(valores)
}

/**
Formatar casas decimais de campos que contêm valores ou alíquotas.

- Tipo "C" corresponde ao tipo de Campo alfanumérico

- Tipo "Valor" e "Aliquota" correspondem ao tipo de Campo numérico

- Tipo "Valor" será formatado com duas casas decimais

- Tipo "Aliquota" será formatado com quatro casas decimais

Formats decimal numbers according to their `Tipo` (Valor, Aliquota, C, N).

If `tipo` is `Tipo::Valor` or `Tipo::Aliquota`, the string `valor` is parsed
as an `f64`, cleaned (removing thousands separators, changing decimal comma to point),
and then formatted to a fixed number of decimal places.
Non-numeric fields or fields not needing specific decimal formatting are returned as-is.

### Arguments
* `valor` - The string representation of the value (can be borrowed or owned).
* `tipo` - The `Tipo` of the field, indicating if it's a value, aliquot, or alphanumeric.
* `num_line` - The line number for error reporting.
* `arquivo` - The path to the file for error reporting.

### Returns
A `Result<String, EFDError>`:
* `Ok(String)`: The formatted string (or original if no formatting needed).
* `Err(EFDError::ParseFloatError)`: If parsing a numeric value fails.
*/
fn formatar_casas_decimais<'a>(
    valor: Cow<'a, str>, // Accept Cow<str> for flexibility (owned or borrowed)
    tipo: Tipo,
    num_line: usize,
    arquivo: &Path,
) -> Result<String, EFDError> {
    if !valor.contains_some_digits() {
        return Ok(valor.into_owned()); // Convert to owned String if returning early
    }

    let decimal: Option<usize> = match tipo {
        Tipo::C => None,
        Tipo::N => None,
        Tipo::Valor => Some(DECIMAL_VALOR),
        Tipo::Aliquota => Some(DECIMAL_ALIQ),
        //Tipo::DataMMYYYY => Some(d),
        //Tipo::DataDDMMYYYY => Some(data),
    };

    if let Some(dec) = decimal {
        let parsed_value: Result<f64, ParseFloatError> = valor
            .replace('.', "") // remover separadores de milhar (se houver)
            .replace(',', ".") // alterar separador decimal de vírgula (",") para ponto (".")
            .parse::<f64>();

        match parsed_value {
            Ok(number) => Ok(format!("{number:0.dec$}")), // Return directly
            Err(source_error) => Err(EFDError::ParseFloatError {
                source: source_error,
                valor_str: valor.into_owned(), // Convert to owned for the error struct
                arquivo: arquivo.to_path_buf(), // Converte &Path para PathBuf
                linha_num: num_line,
            }),
        }
    } else {
        Ok(valor.into_owned()) // If no decimal formatting, return owned
    }
}

/// Parses the accumulated `Info` from a single EFD file into a vector of `DocsFiscais`.
///
/// This function iterates through all records collected in `info.completa` and transforms
/// them into a more structured `DocsFiscais` representation, applying business logic
/// and validation specific to fiscal documents.
///
/// # Arguments
/// * `info` - A mutable reference to the `Info` struct containing raw parsed data.
///
/// # Returns
/// An `EFDResult<Vec<DocsFiscais>>` containing the processed fiscal documents or an `EFDError`.
pub fn parse_file_info(info: &mut Info) -> EFDResult<Vec<DocsFiscais>> {
    let nome_do_cnpj_base = info.obter_nome_do_cnpj_base(); // método implementado
    let arquivo_efd = &info.global["arquivo_efd"];

    let result_database: EFDResult<Vec<DocsFiscais>> = info
        .completa
        .iter()
        .filter(|&(_num_linha_efd, hashmap)| !hashmap.is_empty())
        .map(|(&num_linha_efd, hashmap)| -> EFDResult<DocsFiscais> {
            /*
            // Verificar alguma linha específica:
            if num_linha_efd == 26302 {
                eprintln!("hashmap: {:#?}", hashmap);
                process::exit(1);
            }
            */

            // Obter o Período de Apuração do campo PER_APU_CRED no caso dos Registros 1100 e 1500
            let periodo_de_apuracao =
                obter_periodo_de_apuracao(info.pa, hashmap, info.path.clone(), num_linha_efd)?;

            let estab_cnpj = hashmap.get("estab_cnpj").ok_or(EFDError::InvalidCNPJ(
                arquivo_efd.to_string(),
                num_linha_efd,
            ))?;

            let estab_nome = hashmap.get("estab_nome").ok_or(EFDError::InvalidName(
                arquivo_efd.to_string(),
                num_linha_efd,
            ))?;

            let registro = hashmap.get("REG").unwrap();
            let cfop = hashmap.get("CFOP").and_then(|v| v.parse::<u16>().ok());
            let cst_cofins = hashmap
                .get("CST_COFINS")
                .and_then(|v| v.parse::<u16>().ok());
            let cod_credito = hashmap.get("COD_CRED").and_then(|v| v.parse::<u16>().ok());

            //if !(cst_cofins >= Some(1) && cst_cofins <= Some(99)) && registros_de_operacoes(registro) {
            let is_cst_valid = cst_cofins.is_some_and(|v| CST_ALL.binary_search(&v).is_ok());
            if !is_cst_valid && registros_de_operacoes(registro) {
                let msg = format!(
                    "CST necessariamente deve ser um número inteiro entre 01 e 99.\n\
                     Erro encontrado no campo 'CST_COFINS' do registro: {registro}\n\
                     Arquivo: {arquivo_efd}\n\
                     nº da linha: {num_linha_efd}\n\
                     Info: {hashmap:#?}\n",
                );
                info.messages.push_str(&msg);
            }

            let tipo_de_operacao: Option<u16> = obter_tipo_de_operacao(hashmap, cst_cofins);

            let natureza_bc: Option<u16> = hashmap
                .get("NAT_BC_CRED")
                .and_then(|v| v.parse::<u16>().ok()) // Aplica a conversão para u16
                .or(obter_cod_da_natureza_da_bc(&cfop, cst_cofins));

            // Indicador da origem do crédito: 0 – Operação no Mercado Interno ; 1 – Operação de Importação
            let indicador_de_origem: Option<u16> = hashmap
                .get("IND_ORIG_CRED")
                .and_then(|v| v.parse::<u16>().ok()) // Aplica a conversão para u16
                .or({
                    // Convert a bool to an integer, true will be 1 and false will be 0.
                    let bool = cfop >= Some(3000) && cfop <= Some(3999);
                    Some(u16::from(bool))
                });

            // Data da Emissão do Documento Fiscal
            // data_doc: Esta coluna necessariamente deve possuir informação de data
            let data_doc: &str = hashmap.get("DT_DOC").map_or("01011900", |data| data);
            let data_emissao: Option<NaiveDate> = get_naive_date(data_doc);

            // Data da Entrada / Aquisição / Execução ou da Saída / Prestação / Conclusão
            // dt_lan: Esta coluna não necessariamente possui informação de data
            let dt_lan: &str = hashmap.get("dt_lan").map_or("", |data| data);
            let data_lancamento: Option<NaiveDate> = get_naive_date(dt_lan);

            let cod_part = hashmap.get("COD_PART").map_or("", |v| v);
            let mut part_cnpj = info
                .participantes
                .get(cod_part)
                .map_or("", |hash| hash.get("CNPJ").map_or("", |v| v));
            let mut part_cpf = info
                .participantes
                .get(cod_part)
                .map_or("", |hash| hash.get("CPF").map_or("", |v| v));
            let mut part_nome = info
                .participantes
                .get(cod_part)
                .map_or("", |hash| hash.get("NOME").map_or("", |v| v));

            // O campo CNPJ_CPF_PART está presente nos registro C191 e C195
            if let Some(cnpj_cpf_part) = hashmap.get("CNPJ_CPF_PART") {
                // analisar o campo CNPJ_CPF_PART: pessoa jurídica ou pessoa física?
                if cnpj_cpf_part.contains_num_digits(14) {
                    let cnpj_base = &cnpj_cpf_part[0..8];
                    let nome = nome_do_cnpj_base.get(cnpj_base).map_or("", |v| v);

                    part_cnpj = cnpj_cpf_part;
                    part_nome = info.nome_do_cnpj.get(part_cnpj).map_or(nome, |v| v);
                } else if cnpj_cpf_part.contains_num_digits(11) {
                    part_cpf = cnpj_cpf_part;
                    part_nome = info.nome_do_cpf.get(part_cpf).map_or("", |v| v);
                }
            }

            // Returns None if the option is None, otherwise calls f with the wrapped value and returns the result.
            let num_doc = hashmap
                .get("NUM_DOC")
                .and_then(|v| v.select_first_digits().parse::<usize>().ok());
            let chave_doc = hashmap.get("CHV_NFE").map_or("", |v| v);
            let cod_modelo = hashmap.get("COD_MOD").map_or("", |v| v);

            let num_item = hashmap
                .get("NUM_ITEM")
                .and_then(|v| v.select_first_digits().parse::<u32>().ok());

            let cod_item = hashmap.get("COD_ITEM").map_or("", |v| v);

            let cod_tipo: &str = info
                .produtos
                .get(cod_item)
                .and_then(|hash| hash.get("TIPO_ITEM"))
                .map_or("", |s| s.as_str());

            let descr_item: &str = info
                .produtos
                .get(cod_item)
                .and_then(|hash| hash.get("DESCR_ITEM"))
                .map_or("", |s| s.as_str());

            let cod_ncm: &str = info
                .produtos
                .get(cod_item)
                .and_then(|hash| hash.get("COD_NCM"))
                .map_or("", |s| s.as_str());

            let nat_operacao = match hashmap.get("COD_NAT") {
                Some(cod_nat) => info.nat_operacao.get(cod_nat).map_or("", |v| v),
                None => "",
            };

            let mut info_complementar = match hashmap.get("COD_INF") {
                Some(cod_inf) => info.complementar.get(cod_inf).map_or("", |v| v).to_string(),
                None => "".to_string(),
            };

            // adicionar Descr_Complementar em info_complementar.
            let descr_compl = hashmap.get("Descr_Complementar").map_or("", |v| v);

            if !descr_compl.is_empty() {
                if !info_complementar.is_empty() {
                    info_complementar = info_complementar + " & " + descr_compl;
                } else {
                    info_complementar = descr_compl.to_string();
                }
            }

            let cod_conta = hashmap.get("COD_CTA").map_or("", |v| v);

            let nome_da_conta = info
                .contabil
                .get(cod_conta)
                .map_or("", |hash| hash.get("NOME_CTA").map_or("", |v| v));

            let valor_item = hashmap.get("VL_ITEM").and_then(|v| v.parse::<f64>().ok());
            let valor_bc = hashmap
                .get("VL_BC_COFINS")
                .and_then(|v| v.parse::<f64>().ok());
            let aliq_pis = hashmap.get("ALIQ_PIS").and_then(|v| v.parse::<f64>().ok());
            let aliq_cofins = hashmap
                .get("ALIQ_COFINS")
                .and_then(|v| v.parse::<f64>().ok());
            let valor_pis = hashmap.get("VL_PIS").and_then(|v| v.parse::<f64>().ok());
            let valor_cofins = hashmap.get("VL_COFINS").and_then(|v| v.parse::<f64>().ok());
            let valor_iss = hashmap.get("VL_ISS").and_then(|v| v.parse::<f64>().ok());
            let valor_bc_icms = hashmap
                .get("VL_BC_ICMS")
                .and_then(|v| v.parse::<f64>().ok());
            let aliq_icms = hashmap.get("ALIQ_ICMS").and_then(|v| v.parse::<f64>().ok());
            let valor_icms = hashmap.get("VL_ICMS").and_then(|v| v.parse::<f64>().ok());

            let tipo_de_credito = determinar_tipo_de_credito(
                cst_cofins,
                aliq_pis,
                aliq_cofins,
                cod_credito,
                indicador_de_origem,
            );

            let mut docs_fiscais = DocsFiscais {
                linhas: 2, // contador, será posteriormente atualizado
                arquivo_efd: arquivo_efd.to_string(),
                num_linha_efd: Some(num_linha_efd),
                estabelecimento_cnpj: estab_cnpj.to_string(),
                estabelecimento_nome: estab_nome.to_uppercase(),
                periodo_de_apuracao: Some(periodo_de_apuracao),
                ano: Some(periodo_de_apuracao.year()),
                trimestre: Some(periodo_de_apuracao.month().div_ceil(3)),
                mes: Some(periodo_de_apuracao.month()),
                tipo_de_operacao,
                indicador_de_origem,
                cod_credito,
                tipo_de_credito,
                registro: registro.to_string(),
                cst: cst_cofins,
                cfop,
                natureza_bc,
                particante_cnpj: part_cnpj.to_string(),
                particante_cpf: part_cpf.to_string(),
                particante_nome: part_nome.to_string(),
                num_doc,
                chave_doc: chave_doc.to_string(),
                modelo_doc_fiscal: obter_modelo_do_documento_fiscal(cod_modelo),
                num_item,
                tipo_item: obter_tipo_do_item(cod_tipo),
                descr_item: descr_item.to_uppercase(),
                cod_ncm: cod_ncm.to_string(),
                nat_operacao: nat_operacao.to_string(),
                complementar: info_complementar,
                nome_da_conta: nome_da_conta.to_string(),
                data_emissao,
                data_lancamento,
                valor_item,
                valor_bc,
                aliq_pis,
                aliq_cofins,
                valor_pis,
                valor_cofins,
                valor_iss,
                valor_bc_icms,
                aliq_icms,
                valor_icms,
            };

            docs_fiscais.format();

            Ok(docs_fiscais)
        })
        .collect();

    result_database
}

/// Determines the type of operation (Entrada/Saída) based on:
/// 1. The "tipo_de_operacao" field from the hashmap, if available and parsable.
/// 2. A fallback rule using the `cst_cofins` value if the primary field is missing or invalid.
///
/// ### Arguments
/// * `hashmap` - A reference to the `HashMap` containing the parsed record fields.
/// * `cst_cofins` - An `Option<u16>` representing the CST_COFINS value.
///
/// ### Returns
/// An `Option<u16>`: `Some(1)` for Entrada, `Some(2)` for Saída, or `None` if
/// neither source can determine the operation type.
#[allow(clippy::unnecessary_lazy_evaluations)]
pub fn obter_tipo_de_operacao(
    hashmap: &HashMap<String, String>,
    cst_cofins: Option<u16>,
) -> Option<u16> {
    // Attempt to get the value for "tipo_de_operacao" and parse it as u16.
    // get() returns Option<&String>.
    // and_then() applies the closure only if get() returned Some.
    // v.parse::<u16>().ok() attempts parsing; returns Some(u16) on success, None on ParseIntError.
    // The whole chain results in Some(u16) if the key exists AND parsing succeeded, otherwise None.
    hashmap
        .get("tipo_de_operacao")
        .and_then(|v| v.parse::<u16>().ok())
        // If the primary attempt above resulted in None, evaluate the closure
        // to get a fallback Option<u16> value.
        .or_else(|| {
            // Determine the fallback value based on the provided cst_cofins Option.
            match cst_cofins {
                Some(1..=49) => Some(2),  // "Saída"
                Some(50..=99) => Some(1), // "Entrada"
                _ => None,                // Any other CST range
            }
        })
}

/// Determina o Período de Apuração (PA) a partir do campo `PER_APU_CRED`
/// no hashmap ou do `periodo_de_apuracao_da_efd` geral do arquivo.
fn obter_periodo_de_apuracao(
    periodo_de_apuracao_da_efd: Option<NaiveDate>,
    hashmap: &HashMap<String, String>,
    arquivo: PathBuf,
    line_number: usize,
) -> EFDResult<NaiveDate> {
    // Obter o Período de Apuração do campo PER_APU_CRED no caso dos Registros 1100 e 1500
    let key = "PER_APU_CRED";
    match hashmap.get(key) {
        // PER_APU_CRED : Período de Apuração de Origem do Crédito, formato: MMYYYY
        Some(pa_origem) => {
            let pa_numeric = REGEX_REMOVE_NON_DIGITS.replace_all(pa_origem, "");

            // Formatar string para "01MMYYYY" para facilitar o parsing como data
            let periodo_com_dia = format!("01{}", pa_numeric);

            NaiveDate::parse_from_str(&periodo_com_dia, "%d%-m%Y").map_err(|e| {
                EFDError::ParseDateError {
                    source: e,
                    data_str: pa_origem.to_string(),
                    campo_nome: key.to_string(),
                    arquivo,
                    line_number,
                }
            })
        }
        // Se PER_APU_CRED não for encontrado, tenta usar o PA da EFD ou retorna NotFound
        None => periodo_de_apuracao_da_efd.ok_or(EFDError::KeyNotFound(key.to_string())),
    }
}

/// Ver comentários do Registro M100 do Guia Prático da EFD Contribuições:
///
/// Os códigos dos tipos de créditos são definidos a partir das informações de CST e
/// Alíquota constantes nos documentos e operações registrados nos blocos A, C, D e F.
///
/// O código 109 (atividade imobiliária) é obtido diretamente dos registros F205 e F210,
/// bem como os códigos relativos ao estoque de abertura (104, 204 e 304), os quais
/// são obtidos diretamente do registro F150 (NAT_BC_CRED = 18).
///
/// Determines the `tipo_de_credito` (type of credit) based on various factors
/// from EFD records, aligning with SPED Contribuições practical guide rules.
///
/// This involves checking CST, PIS/COFINS aliquots, credit code, and origin indicator.
///
/// ### Arguments
/// * `cst_cofins` - CST (Código de Situação Tributária) for COFINS.
/// * `aliq_pis` - PIS aliquot value.
/// * `aliq_cofins` - COFINS aliquot value.
/// * `cod_credito` - Specific credit code from the record.
/// * `indicador_de_origem` - Indicator of credit origin (0 for Internal Market, 1 for Importation).
///
/// ### Returns
/// An `Option<u16>` representing the determined credit type code.
fn determinar_tipo_de_credito(
    cst_cofins: Option<u16>,
    aliq_pis: Option<f64>,
    aliq_cofins: Option<f64>,
    cod_credito: Option<u16>,
    indicador_de_origem: Option<u16>,
) -> Option<u16> {
    let mut tipo_de_credito = match (aliq_pis, aliq_cofins) {
        // Filtrar alíquotas de PIS/PASEP e de COFINS não nulas
        (Some(aliq_pis_valor), Some(aliq_cof_valor))
            if (aliq_pis_valor > 0.0 || aliq_cof_valor > 0.0) =>
        {
            // Indicador da origem do crédito: 0 – Operação no Mercado Interno ; 1 – Operação de Importação
            match indicador_de_origem {
                Some(0) => {
                    match cst_cofins {
                        Some(50..=56) => {
                            if aliq_pis_valor == ALIQ_BASICA_PIS
                                && aliq_cof_valor == ALIQ_BASICA_COF
                            {
                                Some(1) // Alíquota Básica
                            } else {
                                Some(2) // Alíquotas Diferenciadas
                            }
                        }
                        Some(60..=66) => {
                            if cred_presumido(aliq_pis_valor, aliq_cof_valor) {
                                Some(6) // Presumido da Agroindústria
                            } else {
                                Some(7) // Outros Créditos Presumidos
                            }
                        }
                        _ => None,
                    }
                }

                Some(1) => Some(8), // Importação

                _ => None,
            }
        }

        _ => None,
    };

    tipo_de_credito = match cod_credito {
        // capturar os dois últimos dígitos dos três digitos, '101' --> '01' ou '308' --> '08'.
        // Remainder operator (%): 101 % 100 = 1 and 308 % 100 = 8
        Some(codigo) => Some(codigo % 100),
        None => tipo_de_credito,
    };

    tipo_de_credito
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

/// Run tests with:
/// cargo test -- --show-output analize_one_tests_v1
#[cfg(test)]
mod analize_one_tests_v1 {
    use super::*;
    use crate::{
        DELIMITER_CHAR, NEWLINE_BYTE, SplitLine, get_string_utf8, make_dispatch_table,
        padronizar_registro, sped_efd,
    };
    use rayon::prelude::*;
    use std::io::BufRead;
    use std::path::PathBuf;
    use std::{fs, io::Write};

    // cargo test -- --help
    // cargo test -- --nocapture
    // cargo test -- --show-output

    const ALL_LINES: &str = "\
| 0000 |006|0|||01112020|30112020|Empresa Teste LTDA|12345678000106|SP|1234567||00|9|
 | 0001 | 0 |
|0100|Fulãno de Tal|12345678901|1SP123456-12|12345678000123|12345678|Rua Sem Nome|123|APTO 12345|Jardim Sem Capim|1234567890|1234567890|fulano.de.tal@rust.org|1234567|
|0110|1|2|2||

|0111|12345,12|0234501,98|0,00|0,00|0134567,89|
 |
 |   |
|0140|123|Empresa Teste LTDA|12345678000123|SP|120388877777|3456789|22233||
|0150|ABC000000387|Empresa Fornecedora de Insumos Teste LTDA|01234|98765432100123|||3456789||Rua dos Bobos|0||Condomínio Vinícius de Moraes|
   |  C001  | 0 |
|C010 | 12345678000123 | 2 |
|C100|0|1|ABC000000387|55|00|1|921|35201101234567890123450010010001234567890123|17112020|17112020|150,00|1|0,00|0,00|150,00|3|0,00|0,00|0,00|0,00|0,00|0,00|0,00|0,00|0,00|0,00|0,00|0,00|
 | C170 | 1|7898954094196||1,00000|6|160,00|0,00|0|060|1653|2006|0,00|0,00|0,00|0,00|0,00|0,00||||0,00|0,00|0,00|50|0,00|0,0000|||0,00|50|0,00|0,0000|||||
|C100|0|1|ABC000000387|55|00|1|929|35201101234567890123450010020001234567890123|30112020|30112020|150,00|1|0,00|0,00|150,00|3|0,00|0,00|0,00|0,00|0,00|0,00|0,00|0,00|0,00|0,00|0,00|0,00|
  | C170 |1|7898954094196||1,00000|6|170,00|0,00|0|060|1653|2006|0,00|0,00|0,00|0,00|0,00|0,00||||0,00|0,00|0,00|55|0,00|0,0000|||0,00|55|0,00|0,0000|||0,00|591|
    |  C100   | 0 |1 |ABC000000516|55|00|22|22137|35201101234567890123450010030001234567890123|30112020|30112020|23430,00|2|0,00|0,00|23430,00|0|||||||||||||
| C170|1|8525||1000,00000|6|23430,00|0,00|0|040|1202|2094|0,00|0,00|0,00|0,00|0,00|0,00|0|49||0,00|0,00|0,00|70|0,00|0,0000|||0,00|70|0,00|0,0000|||0,00|761|
|9999|732|
";

    fn make_file(filename: &str, all_lines: &str) -> EFDResult<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(filename)?;
        file.write_all(all_lines.as_bytes())?;
        Ok(())
    }

    #[test]
    fn unzip_tuples() {
        // cargo test -- --show-output unzip_tuples
        let vec_of_tuples: Vec<(i32, Vec<f64>)> = vec![
            (1, vec![1.1, 1.2, 1.3, 1.4, 1.5]),
            (2, vec![2.1, 2.2, 2.3, 2.4, 2.5]),
            (3, vec![3.1, 3.2, 3.3, 3.4, 3.5]),
        ];

        println!("vec_of_tuples: {vec_of_tuples:?}");

        // Collect vec of tuples Vec<(T, U)> into two vecs Vec<T> and Vec<U>
        let (vec_i32, vec_f64): (Vec<i32>, Vec<Vec<f64>>) = vec_of_tuples.into_par_iter().unzip();

        println!("vec_i32: {vec_i32:?}");
        println!("vec_f64: {vec_f64:?}");

        assert_eq!(vec![1, 2, 3], vec_i32,);

        assert_eq!(
            vec![
                vec![1.1, 1.2, 1.3, 1.4, 1.5],
                vec![2.1, 2.2, 2.3, 2.4, 2.5],
                vec![3.1, 3.2, 3.3, 3.4, 3.5],
            ],
            vec_f64,
        );
    }

    #[test]
    fn flatten_nested_vec() {
        // cargo test -- --show-output flatten_nested_vec
        let nested_vec: Vec<Vec<f64>> = vec![
            vec![1.1, 1.2, 1.3, 1.4, 1.5],
            vec![2.1, 2.2, 2.3, 2.4, 2.5],
            vec![3.1, 3.2, 3.3, 3.4, 3.5],
        ];
        println!("nested_vec: {nested_vec:?}");

        // // Flatten a Vec<Vec<T>> to a Vec<T>
        let flat_vec: Vec<f64> = nested_vec.into_par_iter().flatten().collect();
        println!("flat_vec: {flat_vec:?}");

        assert_eq!(
            vec![
                1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1, 3.2, 3.3, 3.4, 3.5,
            ],
            flat_vec,
        );
    }

    #[test]
    fn analisar_linhas() -> EFDResult<()> {
        // cargo test -- --show-output analisar_linhas
        let registros_efd = sped_efd::registros(); // tabela de registros
        let arquivo = PathBuf::from("teste");
        let num_line = 1;

        let line: &str =
            "| m210|  01  teste  |11890046,5|11890046,5| 1,65 |0||196185,7|1| 2|3|4|196185,77 |";
        println!("line: '{line}'");

        let mut campos: Vec<String> = line.split_line();
        println!("campos: {campos:?}");

        padronizar_registro(&mut campos);
        println!("campos: {campos:?}");

        let registro: &str = campos[0].as_str();
        println!("registro: {registro}");

        let valores: HashMap<String, String> =
            obter_valores(&registros_efd, &campos, num_line, &arquivo)?;
        println!("valores: {valores:#?}\n");

        assert_eq!(campos.len(), 13);
        assert_eq!(registro, "M210_antigo");
        assert_eq!(valores["REG"], "M210_antigo");
        assert_eq!(valores["VL_CONT_PER"], "196185.77");
        assert_eq!(valores["ALIQ_PIS"], "1.6500");
        assert_eq!(valores["COD_CONT"], "01 teste");

        let line: String = "| m210 |  01   teste  |25066,45|25066,45|0,00|0,00|25066,45|1,65|| |413,62|0,00|0,00|0,00|0,00| 413,6 | ".to_string();
        println!("line: '{line}'");

        let mut campos: Vec<String> = line.split_line();
        println!("campos: {campos:?}");

        padronizar_registro(&mut campos);
        println!("campos: {campos:?}");

        let registro: &str = campos[0].as_str();
        println!("registro: {registro}");

        let valores: HashMap<String, String> =
            obter_valores(&registros_efd, &campos, num_line, &arquivo)?;
        println!("valores: {valores:#?}");

        assert_eq!(campos.len(), 16);
        assert_eq!(registro, "M210");
        assert_eq!(valores["REG"], "M210");
        assert_eq!(valores["VL_CONT_PER"], "413.60");
        assert_eq!(valores["ALIQ_PIS"], "1.6500");
        assert_eq!(valores["COD_CONT"], "01 teste");

        Ok(())
    }

    #[test]
    fn strip_delimiter() {
        // cargo test -- --show-output strip_delimiter
        let line: &str = " | m210|  01  teste  |11890046,5|11890046,5| 1,65 |0||196185,7|0| 0|0|0|196185,77 |||  ";
        println!("line: '{line}'");

        let campos: Vec<String> = line.split_line();

        for (index, campo) in campos.iter().enumerate() {
            println!("campo[{index:2}]: '{campo}'");
        }

        assert_eq!(campos.len(), 15);
        assert_eq!(campos[0], "m210");
        assert_eq!(campos[12], "196185,77");
        assert_eq!(campos[14], "");

        let line: String =
            " y y z | campo0| campo1| campo2 | campo3 |campo4 | campo5 || campo7 |||||| xxx "
                .to_string();
        println!("\nline: '{line}'");

        let campos: Vec<String> = line.split_line();

        for (index, campo) in campos.iter().enumerate() {
            println!("campo[{index:2}]: '{campo}'");
        }

        assert_eq!(campos.len(), 13);
        assert_eq!(campos[0], "campo0");
        assert_eq!(campos[7], "campo7");
        assert_eq!(campos[12], "");

        println!("O delimitador '{DELIMITER_CHAR}' é o byte nº 124");
        for strings in ["|1|2|", "|ç|ã|"] {
            let bytes = strings.bytes();
            println!("'{strings}' --> bytes iter: '{bytes:?}'");
        }
    }

    #[test]
    fn test_analyze_one_file() -> EFDResult<()> {
        // cargo test -- --show-output test_analyze_one_file

        let registros_efd = sped_efd::registros(); // tabela de registros
        let dispatch_table = make_dispatch_table()?;
        let multiprogressbar: MultiProgress = MultiProgress::new();

        let filename: &str = "efd_contribuicoes-test1.txt";
        make_file(filename, ALL_LINES)?;

        let arquivo = PathBuf::from(filename);

        let info: Info = read_and_process_file_lines(
            &registros_efd,
            &dispatch_table,
            &multiprogressbar,
            &arquivo,
            0,
            1,
        )?;

        println!("info:\n{info:#?}");

        assert_eq!(info.completa[&14]["REG"], "C170");
        assert_eq!(info.completa[&14]["VL_ITEM"], "160.00");
        assert_eq!(
            info.completa[&14]["CHV_NFE"],
            "35201101234567890123450010010001234567890123"
        );
        assert_eq!(info.completa[&16]["VL_ITEM"], "170.00");
        assert_eq!(info.completa[&18]["VL_ITEM"], "23430.00");

        Ok(())
    }

    #[test]
    fn open_windows_1252_encoding_file_v1() -> EFDResult<()> {
        // cargo test -- --show-output open_windows_1252_encoding_file
        // Melhor usar este procedimento, pois foi verificado em alguns
        // arquivos que às vezes uma linha está codificada em UTF8,
        // porém outra linha está codificada em windows_1252!

        let filename: &str = "examples/efd_data_random";
        let path = PathBuf::from(filename);
        let file = File::open(path.clone())?;

        let result_lines: EFDResult<Vec<(usize, Vec<String>)>> = BufReader::new(file)
            .split(NEWLINE_BYTE)
            .zip(1..) // changing the initial value from zero to one
            .map(
                |(line_bytes_result, line_number)| -> EFDResult<(usize, Vec<String>)> {
                    let line_bytes = line_bytes_result?;
                    let trimmed_bytes = line_bytes.trim_ascii();
                    let line = get_string_utf8(trimmed_bytes, line_number, &path)?;
                    let campos = line.split_line();
                    Ok((line_number, campos))
                },
            )
            //.map_while(Result::ok)
            .collect();

        let lines = result_lines?;

        println!("line 33: {:?}", &lines[32]);
        println!("line 34: {:?}", &lines[33]);
        println!("line 40: {:?}\n", &lines[39]);

        let data_01 = &lines[32].1[2];
        let data_02 = &lines[33].1[2];
        let data_03 = &lines[39].1[2];

        println!("row 33: {data_01:?}");
        println!("row 34: {data_02:?}");
        println!("row 40: {data_03:?}");

        assert_eq!(data_01, "Manter de 50ºC à 90ºC");
        assert_eq!(
            data_02,
            "“aspas”, símbolo europeu (€) e traços fantasia (– e —)"
        );
        assert_eq!(
            data_03,
            "Caracteres do Windows-1252: “aspas”, o símbolo europeu (€) e traços fantasia (– e —)"
        );

        Ok(())
    }

    #[test]
    fn open_windows_1252_encoding_file_v2() -> EFDResult<()> {
        // cargo test -- --show-output open_windows_1252_encoding_file

        let filename: &str = "examples/efd_data_random";
        let path = PathBuf::from(filename);
        let file = File::open(path)?;

        // enumerate non-zero initial value using zip
        // https://users.rust-lang.org/t/force-enumerate-to-u64
        let iter1 = 1..;
        let iter2 = get_bufreader(file).lines().map_while(Result::ok);

        let lines: Vec<(usize, Vec<String>)> = iter1
            .zip(iter2)
            .map(|(_line_number, line)| (_line_number, line.split_line()))
            .collect();

        println!("line 33: {:?}", &lines[32]);
        println!("line 34: {:?}", &lines[33]);
        println!("line 40: {:?}\n", &lines[39]);

        let data_01 = &lines[32].1[2];
        let data_02 = &lines[33].1[2];
        let data_03 = &lines[39].1[2];

        println!("row 33: {data_01:?}");
        println!("row 34: {data_02:?}");
        println!("row 40: {data_03:?}");

        assert_eq!(data_01, "Manter de 50ºC à 90ºC");
        assert_eq!(
            data_02,
            "“aspas”, símbolo europeu (€) e traços fantasia (– e —)"
        );
        assert_eq!(
            data_03,
            "Caracteres do Windows-1252: “aspas”, o símbolo europeu (€) e traços fantasia (– e —)"
        );

        Ok(())
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
/// cargo test -- --show-output analize_one_tests_v2
#[cfg(test)]
mod analize_one_tests_v2 {
    use super::*;
    use crate::{EFDError, Info}; // Assuming Info is in crate root
    use claudiofsr_lib::svec;

    // Mock functions and data for testing analyze_one.rs
    // This is a minimal setup; a real test might require more elaborate mocks
    // or a test fixture for the entire EFD processing environment.

    // A minimal dummy for `Tipo` and `sped_efd` if they are not in scope or fully defined
    // for `cfg(test)` within `analyze_one.rs` itself.
    // If they are accessible from the crate root as `crate::Tipo`, `crate::sped_efd`,
    // then these can be removed.
    // Assuming `Tipo` is an enum and `sped_efd::registros_antigos` exists.

    // Helper for dummy Info
    fn create_dummy_info(pa: Option<NaiveDate>) -> Info {
        let mut info = Info::new(Path::new("dummy.efd"));
        info.pa = pa;
        info.global
            .insert("arquivo_efd".to_string(), "dummy.efd".to_string());
        info
    }

    // Helper to create a dummy `registros_efd`
    fn create_dummy_registros_efd() -> HashMap<&'static str, HashMap<u16, (&'static str, Tipo)>> {
        let mut map = HashMap::new();

        let reg_0000: HashMap<u16, (&str, Tipo)> = HashMap::from([
            (0, ("0", Tipo::N)), // "0": nivel hierárquico
            (1, ("REG", Tipo::C)),
            (2, ("COD_VER", Tipo::C)),
            (3, ("DT_INI", Tipo::C)),
        ]);
        map.insert("0000", reg_0000);

        let mut reg_1100 = HashMap::new();
        reg_1100.insert(0, ("1", Tipo::N)); // "1": nivel hierárquico
        reg_1100.insert(1, ("REG", Tipo::C));
        reg_1100.insert(2, ("PER_APU_CRED", Tipo::N)); // For PA
        map.insert("1100", reg_1100);

        let mut reg_c100 = HashMap::new();
        reg_c100.insert(0, ("2", Tipo::N)); // "2": nivel hierárquico
        reg_c100.insert(1, ("REG", Tipo::C));
        reg_c100.insert(2, ("IND_OPER", Tipo::C));
        reg_c100.insert(3, ("DT_DOC", Tipo::N)); // For date parsing
        reg_c100.insert(4, ("CNPJ_CPF_PART", Tipo::C)); // For participant
        reg_c100.insert(5, ("VL_ITEM", Tipo::Valor)); // For float parsing
        reg_c100.insert(6, ("ALIQ_PIS", Tipo::Aliquota)); // For float parsing
        reg_c100.insert(7, ("CST_COFINS", Tipo::N)); // For tipo_de_operacao
        reg_c100.insert(8, ("NUM_DOC", Tipo::C));
        map.insert("C100", reg_c100);

        let reg_9999: HashMap<u16, (&str, Tipo)> = HashMap::from([
            (0, ("0", Tipo::N)),
            (1, ("REG", Tipo::C)),
            (2, ("QTD_LIN", Tipo::C)),
        ]);
        map.insert("9999", reg_9999);

        map
    }

    // Helper to create a dummy `dispatch_table`
    fn create_dummy_dispatch_table() -> HashMap<&'static str, FuncaoLerRegistro> {
        let mut map: HashMap<&str, FuncaoLerRegistro> = HashMap::new();
        // A minimal mock function for record processing
        fn mock_read_record(info: &mut Info, valores: HashMap<String, String>) -> EFDResult<()> {
            let path = PathBuf::from(".");
            let pa_default = NaiveDate::from_ymd_opt(1973, 2, 15);
            let periodo_de_apuracao = obter_periodo_de_apuracao(pa_default, &valores, path, 1)?;
            info.pa = Some(periodo_de_apuracao);

            if let Some(_reg) = valores.get("REG") {
                info.completa.insert(
                    valores.get("linha_da_efd").unwrap().parse().unwrap(),
                    valores,
                );
            }
            Ok(())
        }
        map.insert("0000", mock_read_record);
        map.insert("C100", mock_read_record);
        map.insert("1100", mock_read_record);
        map
    }

    #[test]
    /// cargo test -- --show-output file_lines_success
    fn test_read_and_process_file_lines_success() -> EFDResult<()> {
        let registros = create_dummy_registros_efd();
        println!("registros: {registros:#?}");
        println!("registro[0000]: {:?}", registros["0000"]);
        println!("registro[1100]: {:?}", registros["1100"]);
        println!("registro[C100]: {:?}", registros["C100"]);

        let dispatch_table = create_dummy_dispatch_table();
        let multiprogressbar = MultiProgress::new();
        let arquivo = Path::new("test.efd");
        std::fs::write(
            arquivo,
            "|0000|001|062005|\n|1100|012023|\n|C100|0|20230101||100,50|0,0165|50|67|\n|9999|7380|\n",
        )?;

        let result = read_and_process_file_lines(
            &registros,
            &dispatch_table,
            &multiprogressbar,
            arquivo,
            0,
            1,
        );
        println!("result: {result:#?}");
        assert!(result.is_ok());
        let info = result.unwrap();
        println!("info.completa: {:#?}", info.completa);
        assert!(info.pa.is_some());
        assert_eq!(info.pa, NaiveDate::from_ymd_opt(1973, 2, 15));
        assert!(!info.completa.is_empty());
        assert_eq!(info.completa[&3]["DT_DOC"], "20230101");
        std::fs::remove_file(arquivo)?;

        Ok(())
    }

    #[test]
    /// cargo test -- --show-output invalid_record
    fn test_read_and_process_file_lines_invalid_record_type_error() -> EFDResult<()> {
        let registros = create_dummy_registros_efd();
        println!("registros: {registros:#?}");
        println!("registro[0000]: {:?}", registros["0000"]);
        let dispatch_table = create_dummy_dispatch_table();
        let multiprogressbar = MultiProgress::new();
        let arquivo = Path::new("test_invalid_record.efd");
        std::fs::write(
            arquivo,
            "|0000|123|032025\n|UNKNOWN|FIELD|\n|C100|0|20230101||100,50|0,0165|50|35|\n|9999|\n",
        )?;

        let result = read_and_process_file_lines(
            &registros,
            &dispatch_table,
            &multiprogressbar,
            arquivo,
            0,
            1,
        );
        println!("result: {result:#?}");
        assert!(result.is_err());
        if let Err(EFDError::UndefinedRecord { record, .. }) = result {
            assert_eq!(record, "UNKNOWN");
        } else {
            panic!("Expected UndefinedRecord error");
        }
        std::fs::remove_file(arquivo)?;

        Ok(())
    }

    #[test]
    /// cargo test -- --show-output obter_valores_success
    fn test_obter_valores_success() {
        let registros = create_dummy_registros_efd();
        println!("registros: {registros:#?}");
        let campos = svec!["C100", "0", "20230101", "", "100,50", "0,0165", "50", "35"];
        let path = Path::new("test.efd");
        let num_line = 1;

        let result = obter_valores(&registros, &campos, num_line, path);
        println!("result: {result:#?}");
        assert!(result.is_ok());
        let valores = result.unwrap();
        assert_eq!(valores["REG"], "C100");
        assert_eq!(valores["VL_ITEM"], "100.50");
        assert_eq!(valores["ALIQ_PIS"], "0.0165");
        assert_eq!(valores["NUM_DOC"], "35");
        assert_eq!(valores["linha_da_efd"], "1");
    }

    #[test]
    /// cargo test -- --show-output field_index
    fn test_obter_valores_undefined_field_index() {
        let registros = create_dummy_registros_efd();
        println!("registros: {registros:#?}");
        // C100 only has fields up to index 8 in dummy registros. This uses index 9.
        let campos = svec![
            "C100",
            "0",
            "20230101",
            "",
            "100,50",
            "0,0165",
            "50",
            "35",
            "EXTRA_FIELD"
        ];
        let path = Path::new("test.efd");
        let num_line = 1;

        let result = obter_valores(&registros, &campos, num_line, path);
        println!("result: {result:#?}");
        assert!(result.is_err());
        if let Err(EFDError::Other(msg)) = result {
            assert!(msg.contains("Index 9 do registro 'C100' não definido"));
        } else {
            panic!("Expected an 'Other' error for undefined field index");
        }
    }

    #[test]
    fn test_formatar_casas_decimais_valor() {
        let path = Path::new("test.efd");
        let result = formatar_casas_decimais(Cow::Borrowed("1.234,56"), Tipo::Valor, 1, path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1234.56");
    }

    #[test]
    fn test_formatar_casas_decimais_aliquota() {
        let path = Path::new("test.efd");
        let result = formatar_casas_decimais(Cow::Borrowed("0,012345"), Tipo::Aliquota, 1, path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0.0123");
    }

    #[test]
    fn test_formatar_casas_decimais_non_numeric() {
        let path = Path::new("test.efd");
        let result = formatar_casas_decimais(Cow::Borrowed("ABC"), Tipo::C, 1, path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "ABC");
    }

    #[test]
    fn test_formatar_casas_decimais_parse_error() {
        let path = Path::new("test.efd");
        let result = formatar_casas_decimais(Cow::Borrowed("100,XX"), Tipo::Valor, 1, path);
        assert!(result.is_err());
        if let Err(EFDError::ParseFloatError { valor_str, .. }) = result {
            assert_eq!(valor_str, "100,XX");
        } else {
            panic!("Expected ParseFloatError");
        }
    }

    #[test]
    /// cargo test -- --show-output parte_file_info
    fn test_parse_file_info_success() {
        let mut info = create_dummy_info(NaiveDate::from_ymd_opt(2023, 1, 1));
        let mut line_data = HashMap::new();
        line_data.insert("REG".to_string(), "C133".to_string());
        line_data.insert("estab_cnpj".to_string(), "12345678000199".to_string());
        line_data.insert("estab_nome".to_string(), "TESTE EMPRESA".to_string());
        line_data.insert("DT_DOC".to_string(), "15012023".to_string());
        line_data.insert("DT_E_S".to_string(), "18012023".to_string());
        line_data.insert("CST_COFINS".to_string(), "50".to_string());
        line_data.insert("VL_DOC".to_string(), "123,4567".to_string());
        info.completa.insert(1, line_data);
        println!("info: {info:#?}");

        let result = parse_file_info(&mut info);
        assert!(result.is_ok());
        let docs = result.unwrap();
        println!("docs: {docs:#?}");
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].estabelecimento_cnpj, "12.345.678/0001-99");
        assert_eq!(docs[0].data_emissao, NaiveDate::from_ymd_opt(2023, 1, 15));
        assert_eq!(docs[0].tipo_de_operacao.unwrap(), 1); // 50 -> Entrada
    }

    #[test]
    fn test_parse_file_info_missing_cnpj_error() {
        let mut info = create_dummy_info(Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()));
        let mut line_data = HashMap::new();
        line_data.insert("REG".to_string(), "C100".to_string());
        line_data.insert("estab_nome".to_string(), "TESTE EMPRESA".to_string()); // CNPJ missing
        info.completa.insert(1, line_data);

        let result = parse_file_info(&mut info);
        assert!(result.is_err());
        if let Err(EFDError::InvalidCNPJ(_, line_num)) = result {
            assert_eq!(line_num, 1);
        } else {
            panic!("Expected InvalidCNPJ error");
        }
    }

    #[test]
    fn test_obter_tipo_de_operacao_from_hashmap() {
        let mut hashmap = HashMap::new();
        hashmap.insert("tipo_de_operacao".to_string(), "1".to_string());
        assert_eq!(obter_tipo_de_operacao(&hashmap, Some(50)), Some(1));
    }

    #[test]
    fn test_obter_tipo_de_operacao_from_cst_saida() {
        let hashmap = HashMap::new(); // No "tipo_de_operacao" in hashmap
        assert_eq!(obter_tipo_de_operacao(&hashmap, Some(40)), Some(2));
    }

    #[test]
    fn test_obter_tipo_de_operacao_from_cst_entrada() {
        let hashmap = HashMap::new(); // No "tipo_de_operacao" in hashmap
        assert_eq!(obter_tipo_de_operacao(&hashmap, Some(70)), Some(1));
    }

    #[test]
    fn test_obter_tipo_de_operacao_none() {
        let hashmap = HashMap::new();
        assert_eq!(obter_tipo_de_operacao(&hashmap, None), None);
        assert_eq!(obter_tipo_de_operacao(&hashmap, Some(0)), None); // Invalid CST range
    }

    #[test]
    fn test_obter_periodo_de_apuracao_from_pa_cred() -> EFDResult<()> {
        let mut valores = HashMap::new();
        valores.insert("PER_APU_CRED".to_string(), "032023".to_string());
        let path = PathBuf::from(".");
        let periodo_de_apuracao =
            obter_periodo_de_apuracao(NaiveDate::from_ymd_opt(2022, 1, 1), &valores, path, 1)?;

        assert_eq!(
            periodo_de_apuracao,
            NaiveDate::from_ymd_opt(2023, 3, 1).unwrap()
        );

        Ok(())
    }

    #[test]
    fn test_obter_periodo_de_apuracao_from_efd_default() -> EFDResult<()> {
        let valores = HashMap::new(); // No PER_APU_CRED
        let path = PathBuf::from(".");
        let periodo_de_apuracao =
            obter_periodo_de_apuracao(NaiveDate::from_ymd_opt(2022, 1, 1), &valores, path, 1)?;

        assert_eq!(
            periodo_de_apuracao,
            NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()
        );

        Ok(())
    }

    #[test]
    /// cargo test -- --show-output invalid_format
    fn test_obter_periodo_de_apuracao_invalid_format() -> EFDResult<()> {
        let mut valores = HashMap::new();
        valores.insert("PER_APU_CRED".to_string(), "03-22023".to_string());
        let path = PathBuf::from("test_file.efd"); // Usar um nome de arquivo para o PathBuf
        let result = obter_periodo_de_apuracao(None, &valores, path.clone(), 1); // Clone path

        println!("result: {result:#?}");
        assert!(result.is_err());

        if let Err(EFDError::ParseDateError {
            source,
            data_str,
            campo_nome,
            arquivo: err_arquivo,
            line_number: err_linha_num,
        }) = result
        {
            let chrono_err = source;
            let error_message = chrono_err.to_string();
            eprintln!("error_message: {error_message}");
            assert!(
                error_message.contains("trailing input"),
                "Chrono error message did not indicate an invalid month or too long input. Actual: {}",
                error_message
            );

            assert_eq!(data_str, "03-22023");
            assert_eq!(campo_nome, "PER_APU_CRED".to_string());
            assert_eq!(err_arquivo, path);
            assert_eq!(err_linha_num, 1);
        } else {
            panic!("Expected EFDError::ParseDateError, but got: {result:#?}");
        }

        Ok(())
    }

    #[test]
    fn test_obter_periodo_de_apuracao_is_none() -> EFDResult<()> {
        let valores = HashMap::new();
        let path = PathBuf::from(".");
        let result = obter_periodo_de_apuracao(None, &valores, path, 1);

        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_determinar_tipo_de_credito_aliquota_basica_interna() {
        // Temporarily override the constants for this test scope if they are global
        // This is complex with true constants; for testing, it's often better
        // to pass these as arguments or use a test-specific configuration.
        // Assuming ALIQ_BASICA_PIS and ALIQ_BASICA_COF are accessible.
        // For actual test, I will replace ALIQ_BASICA_PIS and ALIQ_BASICA_COF with MOCK_ALIQ_BASICA_PIS and MOCK_ALIQ_BASICA_COF
        // if they are not global constants.

        let result = determinar_tipo_de_credito(
            Some(50),
            Some(ALIQ_BASICA_PIS), // Use actual ALIQ_BASICA_PIS
            Some(ALIQ_BASICA_COF), // Use actual ALIQ_BASICA_COF
            None,
            Some(0),
        );
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_determinar_tipo_de_credito_aliquota_diferenciada_interna() {
        let result = determinar_tipo_de_credito(Some(50), Some(0.01), Some(0.05), None, Some(0));
        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_determinar_tipo_de_credito_importacao() {
        let result = determinar_tipo_de_credito(Some(50), Some(0.0165), Some(0.076), None, Some(1));
        assert_eq!(result, Some(8));
    }

    #[test]
    fn test_determinar_tipo_de_credito_from_cod_credito() {
        // cod_credito (e.g., 101) should take precedence and be modulo 100 (01)
        let result =
            determinar_tipo_de_credito(Some(50), Some(0.01), Some(0.05), Some(101), Some(0));
        assert_eq!(result, Some(1)); // 101 % 100 = 1
    }

    #[test]
    fn test_determinar_tipo_de_credito_none() {
        let result = determinar_tipo_de_credito(None, None, None, None, None);
        assert_eq!(result, None);
    }
}
