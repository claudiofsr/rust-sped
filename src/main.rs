mod mylib;

extern crate csv;
extern crate encoding_rs;
extern crate encoding_rs_io;

use std::fs::File;
use std::io::{Error, Read, Write};

use glob::glob_with; // https://docs.rs/glob/0.3.0/glob/
use glob::MatchOptions;

use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;

use std::collections::HashMap;
use std::time::Instant;

use regex::{Regex,Captures};

//use std::thread; // https://doc.rust-lang.org/book/ch16-01-threads.html

//Rayon: https://rustysurfer.me/processing-millions-of-files-in-parallel-with-rust-and-rayon
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

// #[allow(dead_code)]
fn main() -> Result<(), Error> {
    let start = Instant::now();
    let registros_efd = mylib::registros_efd(); // tabela de registros ; executar esta função uma única vez
    let arquivos_efd = procurar_arquivos_efd();
    let metodo_de_analise = "sequencial";

    if metodo_de_analise == "paralelo" {
        println!("Análise em Paralelo usando rayon:");
        arquivos_efd.par_iter().for_each(|arquivo| {
            analisar_efd(&registros_efd, arquivo).ok();
        });
    } else {
        println!("Análise em sequência:");
        //thread::spawn(move || {
        for (i, arquivo) in arquivos_efd.iter().enumerate() {
            println!("In position {} we have file {:?}", i + 1, arquivo);
            analisar_efd(&registros_efd, arquivo).ok();
        }
        //});
    }

    let path = "arquivo_novo.txt";

    let mut output = File::create(path)?;
    write!(&mut output, "Arquivo novo criado\nFinal\n")?;

    let fib = fibonacci(1, 1, 10);

    for (i, f) in fib.iter().enumerate() {
        println!("fib[{}] = {:?}", i + 1, f);
    }

    println!("Tempo de Execução Total: {:?}", start.elapsed());

    Ok(())
}

fn analisar_efd(registros_efd: &std::collections::HashMap<&str, std::collections::HashMap<&str, &str>>, arquivo: &std::string::String) -> Result<(), Error> {
    println!("Arquivo {:?}", arquivo);

    let file = File::open(arquivo)?; // ? or .expect("file not found!") or .unwrap()

    let mut reader = DecodeReaderBytesBuilder::new()
        .encoding(Some(WINDOWS_1252))
        .build(file);

    let mut buffer_read = String::new();
    reader.read_to_string(&mut buffer_read)?;

    // https://users.rust-lang.org/t/hashmap-of-string-fn/25071/2
    // https://stackoverflow.com/questions/36390665/how-do-you-pass-a-rust-function-as-a-parameter
    // https://stackoverflow.com/questions/51372702/how-do-i-make-a-dispatch-table-in-rust
    // https://doc.rust-lang.org/stable/rust-by-example/fn/closures/input_functions.html
    // https://doc.rust-lang.org/stable/rust-by-example/fn/closures/input_parameters.html
    let mut dispatch_table = HashMap::<&str, &dyn Fn(&str, &mut HashMap<&str, String>, &mut HashMap<String, String>)>::new();
    dispatch_table.insert("0000", &ler_registro_0000);
    dispatch_table.insert("0110", &ler_registro_0110);
    dispatch_table.insert("0111", &ler_registro_0111);
    dispatch_table.insert("0140", &ler_registro_0140);

    let mut info_retidas : HashMap<String, String> = HashMap::new();
    info_retidas.insert("arquivo_da_efd".to_string(), arquivo.to_string());

    // Read the file line by line using the lines() iterator from std::io::BufRead.
    for (index, line) in buffer_read.lines().enumerate() {
        let campos = line.split("|");
        let mut vec: Vec<String> = campos.map(|s| s.to_owned()).collect(); // vec[1].to_uppercase(): correção do registo c491 --> C491
        let registro = vec[1].to_uppercase(); // type String
        let linha_num: usize = index + 1;
        let linha_string = linha_num.to_string();

        if !registros_efd.contains_key(&registro.as_str()) {
            println!("\n\t registro {} não definido em mylib.rs \n", &registro);
        }

        let mut valores: HashMap<&str, String> = HashMap::new();
        valores.insert("linha_da_efd", linha_string);
        valores.insert("nivel", registros_efd[registro.as_str()]["00"].to_string()); // type String to type &str
        valores.insert("REG", registro.clone());

        println!("linha n° {} ; registro {} ; vetor {:?}", linha_num, &registro, vec);
        obter_valores(&registros_efd, &mut valores, &registro, &mut vec);

        if dispatch_table.contains_key(&registro.as_str()) {
            dispatch_table[&registro.as_str()](&registro, &mut valores, &mut info_retidas);
        }

        if linha_num >= 10 || registro == "9999" {
            break;
        }
    }

    for (k, v) in info_retidas {
        println!("info_retidas[{}] = '{}'", k, v);
    }

    Ok(())
}

fn ler_registro_0000(registro: &str, valores: &mut HashMap<&str, String>, info_retidas: &mut HashMap<String, String>) {
    // https://stackoverflow.com/questions/24158114/what-are-the-differences-between-rusts-string-and-str?rq=1
    // Use String if you need owned string data (like passing strings to other threads, or building them at runtime), and use &str if you only need a view of a string.
    // This is identical to the relationship between a vector Vec<T> and a slice &[T], and is similar to the relationship between by-value T and by-reference &T for general types.
    println!("ler_registro_0000 --> registro: {} ; valores = {:?}", registro, valores["REG"]);
    info_retidas.insert("CNPJ".to_string(),    valores["CNPJ"  ].to_string());
    info_retidas.insert("DT_INI".to_string(),  valores["DT_INI"].to_string()); // 01042018
    info_retidas.insert("DT_FIN".to_string(),  valores["DT_FIN"].to_string());
    info_retidas.insert("NOME".to_string(),    valores["NOME"  ].to_string());

    // https://github.com/rust-lang/regex
    let re = Regex::new(r"^(\d{2})(\d{2})(\d{4})$").unwrap();

    for caps in re.captures_iter(&valores["DT_INI"]) {
        // Note that all of the unwraps are actually OK for this regex
        // because the only way for the regex to match is if all of the
        // capture groups match. This is not true in general though!
        let dia = caps.get(1).unwrap().as_str();
        let mes = caps.get(2).unwrap().as_str();
        let ano = caps.get(3).unwrap().as_str();

        // https://stackoverflow.com/questions/30154541/how-do-i-concatenate-strings
        let mut pa_do_credito = mes.to_string();
        pa_do_credito.push_str(&ano); // mesano: 042018

        //info_retidas.insert("dia", dia.to_string());
        info_retidas.insert("mes".to_string(), mes.to_string());
        info_retidas.insert("ano".to_string(), ano.to_string());
        info_retidas.insert("pa_do_credito".to_string(), pa_do_credito.to_string());

        println!("dia: {} , mes: {}, ano: {}, pa_do_credito: {}", &dia, &mes, &ano, &pa_do_credito);
    }

    let cnpj = Regex::new(r"^(\d{8})(\d{6})$").unwrap(); // exemplo 22.333.444/0001-55 --> 22333444000155 --> cnpj_base = 22333444
    for caps in cnpj.captures_iter(&valores["CNPJ"]) {
        let cnpj_base = caps.get(1).unwrap().as_str();
        info_retidas.insert("cnpj_base".to_string(), cnpj_base.to_string());
    }
}

fn ler_registro_0110(registro: &str, valores: &mut HashMap<&str, String>, info_retidas: &mut HashMap<String, String>) {
    println!("ler_registro_0110 --> registro: {} ; valores = {:?}", registro, valores["REG"]);
    info_retidas.insert("IND_APRO_CRED".to_string(),  valores["IND_APRO_CRED"].to_string());
}

fn ler_registro_0111(registro: &str, valores: &mut HashMap<&str, String>, info_retidas: &mut HashMap<String, String>) {
    println!("ler_registro_0111 --> registro: {} ; valores = {:?}", registro, valores["REG"]);
    info_retidas.insert("REC_BRU_NCUM_TRIB_MI".to_string(),  valores["REC_BRU_NCUM_TRIB_MI"].to_string());
    info_retidas.insert("REC_BRU_NCUM_NT_MI".to_string(),    valores["REC_BRU_NCUM_NT_MI"  ].to_string());
    info_retidas.insert("REC_BRU_NCUM_EXP".to_string(),      valores["REC_BRU_NCUM_EXP"    ].to_string());
    info_retidas.insert("REC_BRU_CUM".to_string(),           valores["REC_BRU_CUM"         ].to_string());
    info_retidas.insert("REC_BRU_TOTAL".to_string(),         valores["REC_BRU_TOTAL"       ].to_string()); 
}

// Registro 0140: Tabela de Cadastro de Estabelecimentos
// O Registro 0140 tem por objetivo relacionar e informar os estabelecimentos da pessoa jurídica.
fn ler_registro_0140(_registro: &str, valores: &mut HashMap<&str, String>, info_retidas: &mut HashMap<String, String>) {
    
    let re_cnpj_de_14_digitos = Regex::new(r"^\d{14}$").unwrap();
    let cnpj_do_estabelecimento = &valores["CNPJ"];
    let nome_do_estabelecimento = &valores["NOME"];
    
    if re_cnpj_de_14_digitos.is_match(&cnpj_do_estabelecimento) {
        info_retidas.insert(cnpj_do_estabelecimento.to_string(), nome_do_estabelecimento.to_string());
    }
}

// opção <'a> : https://doc.rust-lang.org/stable/rust-by-example/scope/lifetime/explicit.html
#[allow(dead_code)]
fn obter_valores<'a>(registros_efd: &std::collections::HashMap<&str, std::collections::HashMap<&'a str, &'a str>>, valores: &mut std::collections::HashMap<&'a str, String>, registro: &str, vec: &mut Vec<String>) {

    let re_val  = Regex::new(r"^VL_|^REC_|^SLD_").unwrap();
    let re_aliq = Regex::new(r"^ALIQ_").unwrap();

    for (index, valor) in vec.iter().enumerate() {

        // index 0: nivel ; index 1: REG registro corrigido para uppercase
        if index >= 2 && index < (vec.len() - 1) {
            let idx = format!("{:02}",index); // String
            let campo = registros_efd[registro][idx.as_str()];

            let mut new_valor = valor.clone();
            //let mut new_valor = valor.to_owned(); //owned string - this is yours

            if re_val.is_match(campo) {
                //new_valor = re_val.replace_all(&valor, ".").to_string();
                new_valor = new_valor.replace(",",".");

                let my_number: f64 = new_valor.trim().parse().unwrap();
                new_valor = format!("{:0.2}", my_number).to_string();
            }
            if re_aliq.is_match(campo) {
                //new_valor = re_val.replace_all(&valor, ".").to_string();
                new_valor = new_valor.replace(",",".");

                let my_number: f64 = new_valor.trim().parse().unwrap();
                new_valor = format!("{:0.4}", my_number).to_string();
            }

            valores.insert(campo, new_valor.to_string());

            println!("registro {} ; index {:2} ; campo {:20} --> valor '{}'", &registro, &index, &campo, &valor);
        } else {
            continue;
        }
    }

    for (k, v) in valores {
        println!("valores[{}] = '{}'", k, v);
    }

}

fn procurar_arquivos_efd() -> std::vec::Vec<std::string::String> {
    let mut arquivos = Vec::new();
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    for entry in glob_with("**/PISCOFINS_[0-9][0-9]*_[0-9][0-9]*.txt", options).unwrap() {
        if let Ok(path) = entry {
            let my_path = path.display().to_string();
            //println!("File name was {:?}", my_path);
            arquivos.push(my_path);
        }
    }
    arquivos
}

fn fibonacci(mut val1: i64, mut val2: i64, total: i64) -> Vec<i64> {
    let mut result = Vec::new();
    for _i in 1..(total + 1) {
        let soma: i64 = val1 + val2;
        val1 = val2;
        val2 = soma;
        result.push(soma);
    }
    result
}
