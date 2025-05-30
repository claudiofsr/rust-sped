use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read},
    num::ParseFloatError,
    ops::Deref,
    path::Path,
    process, str,
};

use indicatif::{MultiProgress, ProgressBar};

use chrono::{Datelike, NaiveDate};
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::{DecodeReaderBytes, DecodeReaderBytesBuilder};

use claudiofsr_lib::{
    BytesExtension, CST_ALL, FileExtension, StrExtension, get_naive_date, get_style, num_digits,
    open_file,
};

// use memmap2::Mmap;

use crate::{
    ALIQ_BASICA_COF, ALIQ_BASICA_PIS, DECIMAL_ALIQ, DECIMAL_VALOR, DocsFiscais, EFDError,
    EFDResult, Info, NEWLINE_BYTE, SplitLine, Tipo, cred_presumido, obter_cod_da_natureza_da_bc,
    obter_modelo_do_documento_fiscal, obter_tipo_do_item, registros_de_operacoes, sped_efd,
};

// Tipo utilizado em fn make_dispatch_table()
type FuncaoLerRegistro = fn(&mut Info, HashMap<String, String>) -> EFDResult<()>;
type Informacoes = (u32, NaiveDate, String, Vec<DocsFiscais>);

pub fn analyze_one_file(
    registros_efd: &HashMap<&str, HashMap<u16, (&str, Tipo)>>,
    dispatch_table: &HashMap<&str, FuncaoLerRegistro>,
    multiprogressbar: &MultiProgress,
    arquivo: &Path,
    index: usize,
    total: usize,
) -> EFDResult<Informacoes> {
    let mut info: Info = get_file_info(
        registros_efd,
        dispatch_table,
        multiprogressbar,
        arquivo,
        index,
        total,
    )?;

    let vec_docs_fiscais: Vec<DocsFiscais> = parse_file_info(&mut info)?;

    Ok((
        info.cnpj_base,
        info.pa.unwrap(),
        info.messages,
        vec_docs_fiscais,
    ))
}

fn get_file_info(
    registros_efd: &HashMap<&str, HashMap<u16, (&str, Tipo)>>,
    dispatch_table: &HashMap<&str, FuncaoLerRegistro>,
    multiprogressbar: &MultiProgress,
    arquivo: &Path,
    index: usize,
    total: usize,
) -> EFDResult<Info> {
    let file_number = index + 1;
    let num_len = num_digits(total); // total.to_string().len()

    let mut info = Info::new(arquivo);
    let mut progressbar: ProgressBar = get_progressbar(multiprogressbar, index, arquivo)?;

    let mut empty_msg: bool = true;

    match File::open(arquivo) {
        Ok(file) => {
            Box::new(BufReader::new(file))
                .split(NEWLINE_BYTE)
                .zip(1..) // changing the initial value from zero to one
                .map(
                    |(result_vec_bytes, line_number)| -> EFDResult<(usize, Vec<String>)> {
                        let vec_bytes = result_vec_bytes?;
                        let line = get_string_utf8(vec_bytes.trim(), line_number, arquivo)?;
                        let campos = line.split_line();
                        Ok((line_number, campos))
                    },
                )
                //.map_while(Result::ok)
                .map_while(is_valid_result)
                .filter(|(_line_number, campos)| campos.len() >= 2)
                .map(|(line_number, mut campos)| {
                    padronizar_registro(&mut campos);
                    (line_number, campos)
                })
                .filter(|(line_number, campos)| {
                    registro_valido(registros_efd, campos, line_number, arquivo)
                })
                .take_while(|(_line_number, campos)| campos[0] != "9999")
                .try_for_each(|(line_number, campos)| -> EFDResult<()> {
                    let registro: &str = campos[0].as_str();

                    if let Some(&ler_registro) = dispatch_table.get(registro) {
                        let valores: HashMap<String, String> =
                            obter_valores(registros_efd, &campos, line_number, arquivo)?;
                        ler_registro(&mut info, valores)?;
                    }

                    atualizar_progressbar(
                        &mut progressbar,
                        &mut empty_msg,
                        &info,
                        file_number,
                        num_len,
                    );

                    progressbar.inc(1);

                    Ok(())
                })?;
        }
        Err(error) => return Err(EFDError::Io(error)),
    }

    progressbar.finish();

    Ok(info)
}

/// Check if the result is valid.
pub fn is_valid_result<T, E>(result: Result<T, E>) -> Option<T>
where
    E: std::fmt::Display,
{
    match result {
        Ok(var) => Some(var),
        Err(why) => {
            eprintln!("fn is_valid_result()");
            eprintln!("Error: {why}");
            //Err(Box::new(EFDError::Error(why)))
            process::exit(1);
        }
    }
}

fn get_progressbar(
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

/// Updates the progress bar message and sets empty_msg to false
fn atualizar_progressbar(
    progressbar: &mut ProgressBar,
    empty_msg: &mut bool,
    info: &Info,
    file_number: usize,
    num_len: usize,
) {
    // Check if the condition is met
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

/**
Converts a slice of bytes to a String, attempting to handle different encodings.

It first tries to decode the bytes as UTF-8. If that fails, it attempts to decode
them using WINDOWS_1252 encoding. If both fail, it returns an error.

### Arguments

* `slice_bytes` - A slice of bytes to convert to a String.
* `line_number` - The line number where these bytes were read from (for error reporting).
* `path` - The path to the file from which these bytes were read (for error reporting).

### Returns

A `Result` containing the decoded String if successful, or an error if decoding fails.

```rust
use efd_contribuicoes::{get_string_utf8, EFDResult};

fn main() -> EFDResult<()> {
    let bytes: &[u8] = "café".as_bytes();
    let path = std::path::Path::new("my_file.txt");
    // Use the ? operator to propagate the error
    let result: String = get_string_utf8(bytes, 1, &path)?;

    assert_eq!(result, "café");
    Ok(())
}
```
*/
pub fn get_string_utf8(slice_bytes: &[u8], line_number: usize, path: &Path) -> EFDResult<String> {
    // Attempt to decode as UTF-8 first
    match str::from_utf8(slice_bytes) {
        Ok(str) => Ok(str.to_string()),
        Err(error1) => {
            // If UTF-8 decoding fails, attempt WINDOWS_1252 decoding
            let mut decoder = DecodeReaderBytesBuilder::new()
                .encoding(Some(WINDOWS_1252))
                .build(slice_bytes);

            let mut buffer = String::new();
            match decoder.read_to_string(&mut buffer) {
                Ok(_number_of_byte) => Ok(buffer),
                // If WINDOWS_1252 decoding also fails, return a detailed error
                Err(error2) => Err(EFDError::Utf8DecodeError(
                    path.to_path_buf(),
                    line_number,
                    error1,
                    error2,
                )),
            }
        }
    }
}

/**
Obter registro padronizado

Registro é o primeiro elemento de campos
```
    use claudiofsr_lib::svec;
    use efd_contribuicoes::padronizar_registro;
    let mut campos: Vec<String> = svec![
        "d101", "0", "15,24", "56", "02",
        "12,13", "1,65", "0,20", "código XYZ"
    ];
    padronizar_registro(&mut campos);
    assert_eq!(campos[0], "D101".to_string());
```
*/
pub fn padronizar_registro(campos: &mut [String]) {
    let mut registro: String = match campos.first() {
        Some(reg) => reg.to_uppercase(),
        None => return,
    };

    // Substituir registro M210 por M210_antigo ou M610 por M610_antigo,
    // se campos.len() == 13
    if sped_efd::registros_antigos(&registro, campos.len()) {
        registro.push_str("_antigo");
    }

    campos[0] = registro;
}

fn registro_valido<T>(
    registros_efd: &HashMap<&str, HashMap<u16, (&str, Tipo)>>,
    campos: &[T],
    num_line: &usize,
    arquivo: &Path,
) -> bool
where
    T: Deref<Target = str> + std::fmt::Debug,
{
    let registro: &str = &campos[0];
    if registros_efd.contains_key(registro) {
        true
    } else {
        eprintln!("\nArquivo: {arquivo:?}.");
        eprintln!("Linha nº {num_line}");
        eprintln!("Dados {campos:?}");
        eprintln!("Registro '{registro}' não definido conforme sped_efd.rs\n");
        false
    }
}

fn obter_valores<T>(
    registros_efd: &HashMap<&str, HashMap<u16, (&str, Tipo)>>,
    campos: &[T],
    num_line: usize,
    arquivo: &Path,
) -> EFDResult<HashMap<String, String>>
where
    T: Deref<Target = str> + std::fmt::Debug,
{
    let mut index: u16 = 1;
    let registro: &str = &campos[0];

    let mut valores: HashMap<String, String> = HashMap::from([
        ("linha_da_efd".to_string(), num_line.to_string()),
        (
            "nivel".to_string(),
            registros_efd[registro][&0].0.to_string(),
        ),
    ]);

    for valor in campos {
        // https://doc.rust-lang.org/rust-by-example/flow_control/let_else.html
        let Some(&(campo, tipo)) = registros_efd[registro].get(&index) else {
            eprintln!("Registro: {registro}");
            eprintln!("Índice: {index}");
            eprintln!("Erro: Índice não definido!");
            process::exit(1);
        };

        let valor_alterado: String = valor.replace_multiple_whitespaces();
        let valor_formatado: String =
            formatar_casas_decimais(valor_alterado, tipo, num_line, arquivo)?;

        //println!("registro {registro} ; index {index:2} ; campo {campo:20} --> valor '{valor_formatado}'");

        valores.insert(campo.to_string(), valor_formatado);

        index += 1;
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
*/
fn formatar_casas_decimais(
    valor: String,
    tipo: Tipo,
    num_line: usize,
    arquivo: &Path,
) -> Result<String, EFDError> {
    if !valor.contains_some_digits() {
        return Ok(valor);
    }

    let mut valor_formatado: String = valor.clone();

    let decimal: Option<usize> = match tipo {
        Tipo::C => None,
        Tipo::N => None,
        Tipo::Valor => Some(DECIMAL_VALOR),
        Tipo::Aliquota => Some(DECIMAL_ALIQ),
    };

    if let Some(dec) = decimal {
        let formatted: Result<f64, ParseFloatError> = valor
            .replace('.', "") // remover separadores de milhar (se houver)
            .replace(',', ".") // alterar separador decimal de vírgula (",") para ponto (".")
            .parse::<f64>();

        valor_formatado = match formatted {
            Ok(number) => {
                format!("{number:0.dec$}")
            }
            Err(error) => {
                eprintln!("fn formatar_casas_decimais()");
                eprintln!("Linha nº {num_line} do arquivo '{arquivo:?}'.");
                eprintln!("Erro na conversão de '{valor}' para Valor Numérico Float64.");
                return Err(EFDError::ParseFloatError(error));
            }
        };
    }

    Ok(valor_formatado)
}

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
            let periodo_de_apuracao = Some(obter_periodo_de_apuracao(info.pa, hashmap)?);

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

            let periodo_de_apuracao_ano: Option<i32> = periodo_de_apuracao.map(|dt| dt.year());
            let periodo_de_apuracao_mes: Option<u32> = periodo_de_apuracao.map(|dt| dt.month());

            let mut docs_fiscais = DocsFiscais {
                linhas: 2, // contador, será posteriormente atualizado
                arquivo_efd: arquivo_efd.to_string(),
                num_linha_efd: Some(num_linha_efd),
                estabelecimento_cnpj: estab_cnpj.to_string(),
                estabelecimento_nome: estab_nome.to_uppercase(),
                periodo_de_apuracao,
                ano: periodo_de_apuracao_ano,
                trimestre: periodo_de_apuracao_mes.map(|m| m.div_ceil(3)),
                mes: periodo_de_apuracao_mes,
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
/// 1. Primarily, parsing "tipo_de_operacao" from the provided hashmap.
/// 2. Secondarily, if the hashmap key is missing or parsing fails,
///    using a fallback rule based on the CST value provided.
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

fn obter_periodo_de_apuracao(
    periodo_de_apuracao_da_efd: Option<NaiveDate>,
    hashmap: &HashMap<String, String>,
) -> Result<NaiveDate, EFDError> {
    // Obter o Período de Apuração do campo PER_APU_CRED no caso dos Registros 1100 e 1500
    match hashmap.get("PER_APU_CRED") {
        Some(pa_origem) => {
            let (month, year) = parse_mmyyyy(pa_origem)?;

            if !(1..=12).contains(&month) || year < 1900 {
                eprintln!("Data inválida: ano: {}, mês: {}", year, month);
                return Err(EFDError::InvalidDate);
            }
            // Tentar criar a data para garantir a validade
            NaiveDate::from_ymd_opt(year as i32, month, 1).ok_or(EFDError::InvalidDate)
        }
        None => periodo_de_apuracao_da_efd.ok_or(EFDError::NotFound),
    }
}

/// PER_APU_CRED : Período de Apuração de Origem do Crédito, formato: MMYYYY
fn parse_mmyyyy(s: &str) -> Result<(u32, u32), EFDError> {
    let dt = s.trim();

    if dt.len() != 6 {
        return Err(EFDError::InvalidFormat);
    }

    let mmyyyy = dt.parse::<u32>().map_err(|e| {
        let msg_1 = "Erro ao executar a função: fn parse_mmyyyy()\n";
        let msg_2 = "Data MMYYYY deve conter 6 dígitos!\n";
        let msg_3 = format!("Data: {dt}\n");
        let msg = [msg_1, msg_2, &msg_3].concat();
        EFDError::ParseIntError(e, msg)
    })?;

    let month = mmyyyy / 10_000;
    let year = mmyyyy % 10_000;

    Ok((month, year))
}

#[allow(dead_code)]
fn obter_periodo_de_apuracao_v2(
    periodo_de_apuracao_da_efd: Option<NaiveDate>,
    hashmap: &HashMap<String, String>,
) -> Option<NaiveDate> {
    // Obter o Período de Apuração do campo PER_APU_CRED no caso dos Registros 1100 e 1500
    match hashmap.get("PER_APU_CRED") {
        // PER_APU_CRED : Período de Apuração de Origem do Crédito, formato: MMYYYY
        Some(pa_origem) => NaiveDate::parse_from_str(pa_origem.trim(), "%-m%Y").ok(),
        None => periodo_de_apuracao_da_efd,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DELIMITER_CHAR, make_dispatch_table};
    use rayon::prelude::*;
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

        let info: Info = get_file_info(
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
                |(result_vec_bytes, line_number)| -> EFDResult<(usize, Vec<String>)> {
                    let vec_bytes = result_vec_bytes?;
                    let line = get_string_utf8(vec_bytes.trim(), line_number, &path)?;
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
