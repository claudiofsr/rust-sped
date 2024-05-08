use chrono::{NaiveDate, Datelike};

use std::{
    process, // process::exit(1)
    error::Error,
    collections::HashMap,
};

use crate::{
    structures::info::{
        Info,
        Tributos::{self, Pis, Cofins},
    },
    DECIMAL_VALOR,
    obter_grupo_de_contas,
    obter_aliquota_correlacionada_de_pis,
    DESCRICAO_DO_TIPO_DE_RATEIO,
    DESCRICAO_DO_TIPO_DE_CREDITO,
    CODIGO_TIPO_DE_CREDITO,
};

use claudiofsr_lib::{
    StrExtension,
    get_naive_date,
    thousands_separator,
};

type FuncaoLerRegistro = fn(&mut Info, HashMap<String, String>) -> Result<(), Box<dyn Error>>;

pub fn make_dispatch_table() -> Result<HashMap<&'static str, FuncaoLerRegistro>, Box<dyn Error>> {

    let pairs: [(&str, FuncaoLerRegistro); 80] = [
        ("0000", ler_registro_0000),
        ("0110", ler_registro_0110),
        ("0111", ler_registro_0111),
        ("0140", ler_registro_0140),
        ("0150", ler_registro_0150),
        ("0190", ler_registro_0190),
        ("0200", ler_registro_0200),
        ("0400", ler_registro_0400),
        ("0450", ler_registro_0450),
        ("0500", ler_registro_0500),
        ("A010", ler_registro_a010),
        ("A100", ler_registro_a100),
        ("A170", ler_registro_a170),
        ("C010", ler_registro_c010),
        ("C100", ler_registro_c100),
        ("C170", ler_registro_c170),
        ("C175", ler_registro_c175),
        ("C180", ler_registro_c180),
        ("C181", ler_registro_c181),
        ("C185", ler_registro_c185),
        ("C190", ler_registro_c190),
        ("C191", ler_registro_c191),
        ("C195", ler_registro_c195),
        ("C199", ler_registro_c199),
        ("C380", ler_registro_c380),
        ("C381", ler_registro_c381),
        ("C385", ler_registro_c385),
        ("C395", ler_registro_c395),
        ("C396", ler_registro_c396),
        ("C400", ler_registro_c400),
        ("C405", ler_registro_c405),
        ("C481", ler_registro_c481),
        ("C485", ler_registro_c485),
        ("C490", ler_registro_c490),
        ("C491", ler_registro_c491),
        ("C495", ler_registro_c495),
        ("C500", ler_registro_c500),
        ("C501", ler_registro_c501),
        ("C505", ler_registro_c505),
        ("C600", ler_registro_c600),
        ("C601", ler_registro_c601),
        ("C605", ler_registro_c605),
        ("C860", ler_registro_c860),
        ("C870", ler_registro_c870),
        ("C880", ler_registro_c880),
        ("D010", ler_registro_d010),
        ("D100", ler_registro_d100),
        ("D101", ler_registro_d101),
        ("D105", ler_registro_d105),
        ("D200", ler_registro_d200),
        ("D201", ler_registro_d201),
        ("D205", ler_registro_d205),
        ("D300", ler_registro_d300),
        ("D350", ler_registro_d350),
        ("D500", ler_registro_d500),
        ("D501", ler_registro_d501),
        ("D505", ler_registro_d505),
        ("D600", ler_registro_d600),
        ("D601", ler_registro_d601),
        ("D605", ler_registro_d605),
        ("F010", ler_registro_f010),
        ("F100", ler_registro_f100),
        ("F120", ler_registro_f120),
        ("F130", ler_registro_f130),
        ("F150", ler_registro_f150),
        ("F200", ler_registro_f200),
        ("F205", ler_registro_f205),
        ("F210", ler_registro_f210),
        ("F500", ler_registro_f500),
        ("F510", ler_registro_f510),
        ("F550", ler_registro_f550),
        ("F560", ler_registro_f560),
        ("I010", ler_registro_i010),
        ("I100", ler_registro_i100),
        ("M100", ler_registro_m100),
        ("M105", ler_registro_m105),
        ("M500", ler_registro_m500),
        ("M505", ler_registro_m505),
        ("1100", ler_registro_de_controle_de_creditos_fiscais), // registro 1100
        ("1500", ler_registro_de_controle_de_creditos_fiscais), // registro 1500
    ];

    let dispatch_table: HashMap<&str, FuncaoLerRegistro> = HashMap::from(pairs);

    Ok(dispatch_table)
}

fn ler_registro_0000(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    // Período de Apuração do Crédito na EFD: ddmmyyyy
    info.pa = get_naive_date(valores["DT_INI"].as_ref());

    if info.pa.is_none() {
        let arquivo_efd = &info.global["arquivo_efd"];
        eprintln!("fn ler_registro_0000()");
        eprintln!("arquivo: '{arquivo_efd}'");
        eprintln!("Registro: '{}'", valores["REG"]);
        eprintln!("DT_INI: '{}'", valores["DT_INI"]);
        panic!("Não foi possível obter o Período de Apuração da SPED EFD!");
    }

    let cnpj_base: u32 = if valores["CNPJ"].contains_num_digits(14) {
        valores["CNPJ"][..8].parse::<u32>().expect("CNPJ Base deve conter 8 dígitos!")
    } else {
        let arquivo_efd = &info.global["arquivo_efd"];
        eprintln!("fn ler_registro_0000()");
        eprintln!("arquivo: '{arquivo_efd}'");
        eprintln!("Registro: '{}'", valores["REG"]);
        eprintln!("CNPJ: '{}'", valores["CNPJ"]);
        eprintln!("O campo CNPJ deve possuir exatamente 14 dígitos numéricos!");
        process::exit(1);
    };

    info.cnpj_base = cnpj_base;

    // https://stackoverflow.com/questions/44575380/is-there-any-way-to-insert-multiple-entries-into-a-hashmap-at-once-in-rust
    // Insert multiple pairs into a HashMap at once:
    let pairs: [(String, String); 4] = [
        ("NOME".to_string(),   valores["NOME"].to_string()),
        ("CNPJ".to_string(),   valores["CNPJ"].to_string()),
        ("DT_INI".to_string(), valores["DT_INI"].to_string()),
        ("DT_FIN".to_string(), valores["DT_FIN"].to_string()),
    ];

    info.global.extend(pairs);

    Ok(())
}

fn ler_registro_0110(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.global.insert("IND_APRO_CRED".to_string(), valores["IND_APRO_CRED"].to_string());
    Ok(())
}

fn ler_registro_0111(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let pairs: [(String, String); 5] = [
        ("REC_BRU_NCUM_TRIB_MI".to_string(), valores["REC_BRU_NCUM_TRIB_MI"].to_string()),
        ("REC_BRU_NCUM_NT_MI".to_string(),   valores["REC_BRU_NCUM_NT_MI"].to_string()),
        ("REC_BRU_NCUM_EXP".to_string(),     valores["REC_BRU_NCUM_EXP"].to_string()),
        ("REC_BRU_CUM".to_string(),          valores["REC_BRU_CUM"].to_string()),
        ("REC_BRU_TOTAL".to_string(),        valores["REC_BRU_TOTAL"].to_string()),
    ];

    info.global.extend(pairs);
    Ok(())
}

// Registro 0140: Tabela de Cadastro de Estabelecimentos
// O Registro 0140 tem por objetivo relacionar e informar os estabelecimentos da pessoa jurídica.
fn ler_registro_0140(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let cnpj_do_estabelecimento = valores["CNPJ"].to_string();
    let nome_do_estabelecimento = valores["NOME"].to_string();

    if cnpj_do_estabelecimento.contains_num_digits(14) && !nome_do_estabelecimento.is_empty() {
        info.estabelecimentos.insert(cnpj_do_estabelecimento, nome_do_estabelecimento);
    }
    Ok(())
}

// Registro 0150: Tabela de Cadastro do Participante
// Atribuir NOME ao CNPJ ou ao CPF
fn ler_registro_0150(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let codigo_do_participante = valores["COD_PART"].to_string();

    let nome_do_participante = valores["NOME"].to_string();
    let cnpj_do_participante = valores["CNPJ"].to_string();
    let cpf_do_participante  = valores["CPF"].to_string();

    let pairs: [(String, String); 3] = [
        ("NOME".to_string(), nome_do_participante.clone()),
        ("CNPJ".to_string(), cnpj_do_participante.clone()),
        ("CPF".to_string(),  cpf_do_participante.clone()),
    ];

    info.participantes.entry(codigo_do_participante).or_default().extend(pairs);

    if !nome_do_participante.is_empty() {
        if cnpj_do_participante.contains_num_digits(14) {
            info.nome_do_cnpj.insert(cnpj_do_participante, nome_do_participante);
        }
        else if cpf_do_participante.contains_num_digits(11) {
            info.nome_do_cpf.insert(cpf_do_participante, nome_do_participante);
        }
    }
    Ok(())
}

// Registro 0190: Identificação das Unidades de Medida
fn ler_registro_0190(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let cod_unidade = valores["UNID"].to_string();
    let descr = valores["DESCR"].to_string();

    if !cod_unidade.is_empty() && !descr.is_empty() {
        info.unidade_de_medida.insert(cod_unidade, descr);
    }
    Ok(())
}

// Registro 0200: Tabela de Identificação do Item (Produtos e Serviços)
fn ler_registro_0200(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let cod_item = valores["COD_ITEM"].to_string();

    if !cod_item.is_empty() {

        let pairs: [(String, String); 5] = [
            ("DESCR_ITEM".to_string(), valores["DESCR_ITEM"].to_string()),
            ("TIPO_ITEM".to_string(),  valores["TIPO_ITEM"].to_string()),
            ("COD_NCM".to_string(),    valores["COD_NCM"].to_string()),
            ("COD_GEN".to_string(),    valores["COD_GEN"].to_string()),
            ("COD_LST".to_string(),    valores["COD_LST"].to_string()),
        ];

        info.produtos.entry(cod_item).or_default().extend(pairs);
    }
    Ok(())
}

// Registro 0400: Tabela de Natureza da Operação/Prestação
fn ler_registro_0400(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let cod_nat = valores["COD_NAT"].to_string();
    let descr_nat = valores["DESCR_NAT"].to_string();

    if !cod_nat.is_empty() && !descr_nat.is_empty() {
        info.nat_operacao.insert(cod_nat, descr_nat);
    }
    Ok(())
}

// Registro 0450: Tabela de Informação Complementar do Documento Fiscal
fn ler_registro_0450(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let cod_info = valores["COD_INF"].to_string();
    let txt = valores["TXT"].to_string();

    if !cod_info.is_empty() && !txt.is_empty() {
        info.complementar.insert(cod_info, txt);
    }
    Ok(())
}

// Registro 0500: Plano de Contas Contábeis
// Este registro tem o objetivo de identificar as contas contábeis utilizadas pelo contribuinte em sua Escrituração
// Contábil, relacionadas às operações representativas de receitas, tributadas ou não, e dos créditos apurados.
fn ler_registro_0500(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let cod_conta = valores["COD_CTA"].to_string();

    if !cod_conta.is_empty() {
        let grupo_de_contas = obter_grupo_de_contas(&valores["COD_NAT_CC"]);
        let nome_da_conta = valores["NOME_CTA"].to_uppercase();

        let conta_contabil = if nome_da_conta.is_empty() {
            grupo_de_contas
        } else if grupo_de_contas.is_empty() {
            nome_da_conta
        } else {
            [&grupo_de_contas, ": ", &nome_da_conta].concat()
        };

        let pairs: [(String, String); 2] = [
            ("COD_NAT_CC".to_string(), valores["COD_NAT_CC"].to_string()),
            (  "NOME_CTA".to_string(), conta_contabil),
        ];

        info.contabil.entry(cod_conta).or_default().extend(pairs);
    }
    Ok(())
}

fn inserir_cnpj_do_estabelecimento(info: &Info, hmap: &mut HashMap<String, String>) {

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    let estabelecimento_cnpj = info.cnpj_do_estabelecimento.to_string();
    let estabelecimento_nome = match info.estabelecimentos.get(&estabelecimento_cnpj) {
        Some(nome) => nome,
        None => {
            let arquivo_efd = &info.global["arquivo_efd"];
            println!("Arquivo EFD: '{arquivo_efd}'");
            println!("Registro 0140: Tabela de Cadastro de Estabelecimentos");
            println!("Ausência do Nome do Estabelecimento do CNPJ: {estabelecimento_cnpj}!\n");
            ""
        },
    };

    hmap.extend([
        ("estab_cnpj".to_string(), estabelecimento_cnpj.to_string()),
        ("estab_nome".to_string(), estabelecimento_nome.to_string()),
    ]);
}

fn inserir_cnpj_da_matriz(info: &Info, hmap: &mut HashMap<String, String>) {

    // adicionar CNPJ da matriz do contribuinte
    hmap.extend([
        ("estab_cnpj".to_string(), info.global["CNPJ"].to_string()),
        ("estab_nome".to_string(), info.global["NOME"].to_string()),
    ]);
}

// Registro A010: Identificação do Estabelecimento
fn ler_registro_a010(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.cnpj_do_estabelecimento = valores["CNPJ"].to_string();
    Ok(())
}

fn ler_registro_a100(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.reg_a100 = valores; // reter info para posterior uso por reg A170
    Ok(())
}

fn ler_registro_a170(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    let pairs = [
        ("CHV_NFE".to_string(), info.reg_a100["CHV_NFSE"].to_string()),
        ( "DT_DOC".to_string(), info.reg_a100["DT_DOC"].to_string()),
        ( "dt_lan".to_string(), info.reg_a100["DT_EXE_SERV"].to_string()),
    ];

    let mut hmap = HashMap::from(pairs);

    // adicionar info do reg A100 em hmap
    hmap.extend(info.reg_a100.clone());

    // adicionar info do reg A170 em hmap
    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

// Registro C010: Identificação do Estabelecimento
fn ler_registro_c010(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.cnpj_do_estabelecimento = valores["CNPJ"].to_string();
    Ok(())
}

fn ler_registro_c100(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.reg_c100 = valores; // reter info para posterior uso por reg C170 ou C175
    Ok(())
}

fn ler_registro_c170(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    let mut hmap = HashMap::from([
        ("Descr_Complementar".to_string(), valores["DESCR_COMPL"].to_string()),
        ("dt_lan".to_string(), info.reg_c100["DT_E_S"].to_string()),
    ]);

    // adicionar info do reg C100 em hmap
    hmap.extend(info.reg_c100.clone());

    // adicionar info do reg C170 hmap
    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_c175(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    let mut hmap = HashMap::from([
        ("Descr_Complementar".to_string(), valores["INFO_COMPL"].to_string()),
        ("VL_ITEM".to_string(), valores["VL_OPR"].to_string()),
    ]);

    // adicionar info do reg C100 em hmap
    hmap.extend(info.reg_c100.clone());

    // adicionar info do reg C175 hmap
    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

// --- Correlacionar as alíquotas de PIS/PASEP com as alíquotas de COFINS --- //
// ---                               START                                --- //

fn obter_chave_de_correlacao_fraca(contrib: Tributos, valores: &HashMap<String, String>) -> String {
    match contrib {
        Pis    => format!("{}_{}", valores["CST_PIS"],    valores["VL_ITEM"]),
        Cofins => format!("{}_{}", valores["CST_COFINS"], valores["VL_ITEM"]),
    }
}

fn obter_chave_de_correlacao_forte(chave: &str, valores: &HashMap<String, String>) -> String {

    let mut chave_forte: String = chave.to_string();

    if let Some(cfop) = valores.get("CFOP") {
        let add = ["_", cfop].concat();
        chave_forte.push_str(&add);
    }

    if let Some(participante) = valores.get("CNPJ_CPF_PART") {
        let add = ["_", participante].concat();
        chave_forte.push_str(&add);
    }

    chave_forte
}

/*
Em alguns casos, nos registros C191 e C195 ocorrem erros nas informações correlatas.
Os campos CFOP, CNPJ_CPF_PART e (VL_BC_PIS, VL_BC_COFINS) são campos correlacionados.
Primeiramente tentar correlacionar os registros C191 e C195 com a chave forte.
Em caso de falhas nas correlações entre as alíquotas, usar a chave fraca.

|C190|55|15022018|15022018|776973|04022120||1234567,04|
|C191|23456789012341|66|1101|123002,54||123002,54|0,825|||1343,61|012345|
|C191|12345678901234|66|2101|123001,76||123001,76|0,825|||1338,75|012345|
|C195|23456789012341|66|1101|123002,54||123002,54|3,8|||6188,74|012345|
|C195|12345678901234|66|1101|123001,76||123001,76|3,8|||6166,36|012345|

Observe acima que a informação de CFOP está incorreta,
Não ocorreu correlação entre '2101|123001,76' e '1101|123001,76'.
Neste caso, usar a chave fraca.
*/

/// Primeiramente, tentar correlacionar as alíquotas de PIS e COFINS utilizando a chave forte.
///
/// Se a chave forte falhar ao tentar correlacionar as alíquotas, utilizar a chave fraca.
fn correlacionar_aliquotas(info: &mut Info, valores: &mut HashMap<String, String>, linha_da_efd: usize) {

    let chave_fraca: String = obter_chave_de_correlacao_fraca(Cofins, valores);
    let chave_forte: String = obter_chave_de_correlacao_forte(&chave_fraca, valores);

    match info.correlacao.get(&chave_forte) {
        Some(array) => {
            valores.insert("ALIQ_PIS".to_string(), array[0].to_string());
            valores.insert("VL_PIS".to_string(),   array[1].to_string());
        },
        None => {
            let arquivo_efd = &info.global["arquivo_efd"];
            let registro = &valores["REG"];
            let aliq_cofins = &valores["ALIQ_COFINS"];
            let cfop: &str = valores.get("CFOP").map_or("", |v| v);
            let participante: &str = valores.get("CNPJ_CPF_PART").map_or("", |v| v);

            let msg01 = "Erro encontrado ao executar a função correlacionar_aliquotas()!!!".to_string();
            let msg02 = format!("Arquivo: {arquivo_efd}");
            let msg03 = format!("Número da linha: {linha_da_efd}");
            let msg04 = format!("Registro: {registro}");
            let msg05 = format!("Ausência de correlação entre as alíquotas de PIS/PASEP e COFINS ({aliq_cofins})!");
            let msg06 = "chave_forte: CST_COFINS_VL_ITEM_CFOP_CNPJ_CPF_PART tal que:".to_string();
            let msg07 = format!("CST_COFINS: {}", valores["CST_COFINS"]);
            let msg08 = format!("VL_ITEM: {}", valores["VL_ITEM"]);
            let msg09 = format!("CFOP: {cfop}");
            let msg10 = format!("CNPJ_CPF_PART: {participante}");
            let msg11 = "Tentativa de utilizar a chave_fraca".to_string();
            let msg12 = "chave_fraca: CST_COFINS_VL_ITEM\n\n".to_string();

            let msg = [
                msg01, msg02, msg03, msg04, msg05,
                msg06, msg07, msg08, msg09, msg10,
                msg11, msg12,
            ].join("\n");
            info.messages.push_str(&msg);

            match info.correlacao.get(&chave_fraca) {
                Some(array) => {
                    valores.insert("ALIQ_PIS".to_string(), array[0].to_string());
                    valores.insert("VL_PIS".to_string(),   array[1].to_string());
                },
                None => {
                    let msg01 = "Tentativa de utilizar a chave_fraca para correlacionar as alíquotas falhou.".to_string();
                    let msg02 = "Erro encontrado na escrituração da EFD Contribuições!!!\n\n".to_string();

                    let msg = [msg01, msg02].join("\n");
                    info.messages.push_str(&msg);
                },
            }
        },
    };
}

// ---                               FINAL                                --- //
// --- Correlacionar as alíquotas de PIS/PASEP com as alíquotas de COFINS --- //

fn ler_registro_c180(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.reg_c180 = valores; // reter info para posterior uso por reg C185
    info.correlacao = HashMap::new();
    Ok(())
}

fn ler_registro_c181(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let array: [String; 2] = [valores["ALIQ_PIS"].clone(), valores["VL_PIS"].clone()];

    let chave_fraca: String = obter_chave_de_correlacao_fraca(Pis, &valores);
    let chave_forte: String = obter_chave_de_correlacao_forte(&chave_fraca, &valores);

    info.correlacao.insert(chave_fraca, array.clone());
    info.correlacao.insert(chave_forte, array);

    Ok(())
}

fn ler_registro_c185(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(), info.reg_c180["DT_DOC_INI"].to_string());

    hmap.extend(info.reg_c180.clone());
    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // correlacionar aliquota de PIS com aliquota de COFINS
    correlacionar_aliquotas(info, &mut hmap, linha_da_efd);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_c190(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.reg_c190 = valores; // reter info para posterior uso por reg C195
    info.correlacao = HashMap::new();
    info.linhas_inseridas = Vec::new();
    Ok(())
}

fn ler_registro_c191(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let array: [String; 2] = [valores["ALIQ_PIS"].clone(), valores["VL_PIS"].clone()];

    let chave_fraca: String = obter_chave_de_correlacao_fraca(Pis, &valores);
    let chave_forte: String = obter_chave_de_correlacao_forte(&chave_fraca, &valores);

    info.correlacao.insert(chave_fraca, array.clone());
    info.correlacao.insert(chave_forte, array);

    Ok(())
}

fn ler_registro_c195(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(), info.reg_c190["DT_REF_INI"].to_string());

    hmap.extend(info.reg_c190.clone());
    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // correlacionar aliquota de PIS com aliquota de COFINS
    correlacionar_aliquotas(info, &mut hmap, linha_da_efd);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);

    // reter o nº das linhas deste registro para uso no registro_c199
    info.linhas_inseridas.push(linha_da_efd);
    Ok(())
}

/*
Procedimento adotado para registros especiais.
O registro pai C190 possui os registros filhos: C191, C195, C198 e C199.
Os registros C198 e C199 são posteriores aos registros registros C191 e C195.
Objetivo: reter informações dos registros C198 e C199 e transmiti-las aos registros C191 e C195.

Mesmo procedimento para C499, pois os registros C499 são posteriores aos registros C491 e C495.
Mesmo procedimento para D609, pois os registros D609 são posteriores aos registros D601 e D605.
*/

fn ler_registro_c199(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    // adicionar info do registro_c199 no registro_c195, para isto
    // é necessário obter o nº da linha_da_efd do registro_c195

    for &linha_da_efd in &info.linhas_inseridas {

        let mut hmap = HashMap::new();
        hmap.extend(valores.clone());

        // Adicionar a informação do número do doc de importação em Descr_Complementar:
        let num_doc = ["Número do documento de Importação: ", &valores["NUM_DOC_IMP"]].concat();

        hmap.insert("Descr_Complementar".to_string(), num_doc);

        // Dado o nº da linha_da_efd, obter o HashMap correspondente
        let hmap_c195 = match info.completa.get(&linha_da_efd) {
            Some(h) => h,
            None => continue,
        };

        match hmap_c195.get("REG") {
            Some(registro) => {
                // Verificar tratar-se do registro C195
                if registro == "C195" {
                    // transmitir informações do registro C195 a hmap
                    hmap.extend(hmap_c195.clone());
                    // sobrescrever hmap em info.completa
                    info.completa.insert(linha_da_efd, hmap);
                }
            },
            None => continue,
        };
    }
    Ok(())
}

fn ler_registro_c380(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.reg_c380 = valores; // reter info para posterior uso por reg C385
    info.correlacao = HashMap::new();
    Ok(())
}

fn ler_registro_c381(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let chave = obter_chave_de_correlacao_fraca(Pis, &valores);
    let array = [valores["ALIQ_PIS"].clone(), valores["VL_PIS"].clone()];
    info.correlacao.insert(chave, array);
    Ok(())
}

fn ler_registro_c385(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(), info.reg_c380["DT_DOC_INI"].to_string());

    hmap.extend(info.reg_c380.clone());
    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // correlacionar aliquota de PIS com aliquota de COFINS
    correlacionar_aliquotas(info, &mut hmap, linha_da_efd);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_c395(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.reg_c395 = valores; // reter info para posterior uso por reg C396
    info.correlacao = HashMap::new();
    Ok(())
}

fn ler_registro_c396(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    hmap.extend(info.reg_c395.clone());
    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_c400(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.reg_c400 = valores; // reter info para posterior uso por reg C485
    info.correlacao = HashMap::new();
    Ok(())
}

fn ler_registro_c405(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.reg_c405 = valores; // reter info para posterior uso por reg C485
    Ok(())
}

fn ler_registro_c481(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let chave = obter_chave_de_correlacao_fraca(Pis, &valores);
    let array = [valores["ALIQ_PIS"].clone(), valores["VL_PIS"].clone()];
    info.correlacao.insert(chave, array);
    Ok(())
}

fn ler_registro_c485(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    hmap.extend(info.reg_c400.clone());
    hmap.extend(info.reg_c405.clone());
    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // correlacionar aliquota de PIS com aliquota de COFINS
    correlacionar_aliquotas(info, &mut hmap, linha_da_efd);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_c490(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.reg_c490 = valores; // reter info para posterior uso por reg C495
    info.correlacao = HashMap::new();
    Ok(())
}

fn ler_registro_c491(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let chave = obter_chave_de_correlacao_fraca(Pis, &valores);
    let array = [valores["ALIQ_PIS"].clone(), valores["VL_PIS"].clone()];
    info.correlacao.insert(chave, array);
    Ok(())
}

fn ler_registro_c495(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(), info.reg_c490["DT_DOC_INI"].to_string());

    hmap.extend(info.reg_c490.clone());
    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // correlacionar aliquota de PIS com aliquota de COFINS
    correlacionar_aliquotas(info, &mut hmap, linha_da_efd);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_c500(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.reg_c500 = valores; // reter info para posterior uso por reg C505
    info.correlacao = HashMap::new();
    Ok(())
}

fn ler_registro_c501(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let chave = obter_chave_de_correlacao_fraca(Pis, &valores);
    let array = [valores["ALIQ_PIS"].clone(), valores["VL_PIS"].clone()];
    info.correlacao.insert(chave, array);
    Ok(())
}

fn ler_registro_c505(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    let chv_doc = info.reg_c500.get("CHV_DOCe").map_or("", |v| v);
    hmap.insert("CHV_NFE".to_string(), chv_doc.to_string());
    hmap.insert("dt_lan".to_string(), info.reg_c500["DT_ENT"].to_string());

    hmap.extend(info.reg_c500.clone());
    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // correlacionar aliquota de PIS com aliquota de COFINS
    correlacionar_aliquotas(info, &mut hmap, linha_da_efd);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_c600(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.reg_c600 = valores; // reter info para posterior uso por reg C605
    info.correlacao = HashMap::new();
    Ok(())
}

fn ler_registro_c601(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let chave = obter_chave_de_correlacao_fraca(Pis, &valores);
    let array = [valores["ALIQ_PIS"].clone(), valores["VL_PIS"].clone()];
    info.correlacao.insert(chave, array);
    Ok(())
}

fn ler_registro_c605(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    hmap.extend(info.reg_c600.clone());
    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // correlacionar aliquota de PIS com aliquota de COFINS
    correlacionar_aliquotas(info, &mut hmap, linha_da_efd);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_c860(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.reg_c860 = valores; // reter info para posterior uso por reg C870
    Ok(())
}

fn ler_registro_c870(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    hmap.extend(info.reg_c860.clone());
    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_c880(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

// Registro D010: Identificação do Estabelecimento
fn ler_registro_d010(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.cnpj_do_estabelecimento = valores["CNPJ"].to_string();
    Ok(())
}

fn ler_registro_d100(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.reg_d100 = valores; // reter info para posterior uso por reg D105
    info.correlacao = HashMap::new();
    Ok(())
}

fn ler_registro_d101(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let chave = obter_chave_de_correlacao_fraca(Pis, &valores);
    let array = [valores["ALIQ_PIS"].clone(), valores["VL_PIS"].clone()];
    info.correlacao.insert(chave, array);
    Ok(())
}

fn ler_registro_d105(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("CHV_NFE".to_string(), info.reg_d100["CHV_CTE"].to_string());
    hmap.insert( "dt_lan".to_string(), info.reg_d100["DT_A_P"].to_string() );

    hmap.extend(info.reg_d100.clone());
    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // correlacionar aliquota de PIS com aliquota de COFINS
    correlacionar_aliquotas(info, &mut hmap, linha_da_efd);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_d200(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.reg_d200 = valores; // reter info para posterior uso por reg D205
    info.correlacao = HashMap::new();
    Ok(())
}

fn ler_registro_d201(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    hmap.extend(info.reg_d200.clone()); // copiar CFOP
    hmap.extend(valores);

    let array: [String; 2] = [hmap["ALIQ_PIS"].clone(), hmap["VL_PIS"].clone()];

    let chave_fraca: String = obter_chave_de_correlacao_fraca(Pis, &hmap);
    let chave_forte: String = obter_chave_de_correlacao_forte(&chave_fraca, &hmap);

    info.correlacao.insert(chave_fraca, array.clone());
    info.correlacao.insert(chave_forte, array);

    Ok(())
}

fn ler_registro_d205(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert( "DT_DOC".to_string(), info.reg_d200["DT_REF"].to_string());
    hmap.insert("NUM_DOC".to_string(), info.reg_d200["NUM_DOC_INI"].to_string());

    hmap.extend(info.reg_d200.clone()); // copiar CFOP
    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // correlacionar aliquota de PIS com aliquota de COFINS
    correlacionar_aliquotas(info, &mut hmap, linha_da_efd);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_d300(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("VL_ITEM".to_string(), valores["VL_DOC"].to_string());
    hmap.insert( "DT_DOC".to_string(), valores["DT_REF"].to_string());
    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_d350(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("VL_ITEM".to_string(), valores["VL_BRT"].to_string());
    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_d500(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.reg_d500 = valores; // reter info para posterior uso por reg D505
    info.correlacao = HashMap::new();
    Ok(())
}

fn ler_registro_d501(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let chave = obter_chave_de_correlacao_fraca(Pis, &valores);
    let array = [valores["ALIQ_PIS"].clone(), valores["VL_PIS"].clone()];
    info.correlacao.insert(chave, array);
    Ok(())
}

fn ler_registro_d505(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // adicionar informações de alguns campos do reg D500 em hmap
    hmap.insert("dt_lan".to_string(), info.reg_d500["DT_A_P"].to_string());

    hmap.extend(info.reg_d500.clone());
    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // correlacionar aliquota de PIS com aliquota de COFINS
    correlacionar_aliquotas(info, &mut hmap, linha_da_efd);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_d600(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.reg_d600 = valores; // reter info para posterior uso por reg D605
    info.correlacao = HashMap::new();
    Ok(())
}

fn ler_registro_d601(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let chave = obter_chave_de_correlacao_fraca(Pis, &valores);
    let array = [valores["ALIQ_PIS"].clone(), valores["VL_PIS"].clone()];
    info.correlacao.insert(chave, array);
    Ok(())
}

fn ler_registro_d605(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    hmap.extend(info.reg_d600.clone());
    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // adicionar informações de alguns campos do reg D600 em hmap
    hmap.insert("DT_DOC".to_string(), info.reg_d600["DT_DOC_INI"].to_string());

    // correlacionar aliquota de PIS com aliquota de COFINS
    correlacionar_aliquotas(info, &mut hmap, linha_da_efd);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

// Registro F010: Identificação do Estabelecimento
fn ler_registro_f010(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.cnpj_do_estabelecimento = valores["CNPJ"].to_string();
    Ok(())
}

fn ler_registro_f100(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(),  valores["DT_OPER"].to_string());
    hmap.insert("VL_ITEM".to_string(), valores["VL_OPER"].to_string());
    hmap.insert("Descr_Complementar".to_string(), valores["DESC_DOC_OPER"].to_string());

    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_f120(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(), info.global["DT_INI"].to_string());
    hmap.insert("VL_ITEM".to_string(),            valores["VL_OPER_DEP"].to_string());
    hmap.insert("Descr_Complementar".to_string(), valores["DESC_BEM_IMOB"].to_string());

    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_f130(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(),  info.global["DT_INI"].to_string());
    hmap.insert("VL_ITEM".to_string(), valores["VL_BC_CRED"].to_string());
    hmap.insert("Descr_Complementar".to_string(), valores["DESC_BEM_IMOB"].to_string());

    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_f150(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(),  info.global["DT_INI"].to_string());
    hmap.insert("VL_ITEM".to_string(), valores["VL_TOT_EST"].to_string());
    hmap.insert("Descr_Complementar".to_string(), valores["DESC_EST"].to_string());

    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_f200(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(),             valores["DT_OPER"].to_string());
    hmap.insert("VL_ITEM".to_string(),            valores["VL_TOT_REC"].to_string());
    hmap.insert("Descr_Complementar".to_string(), valores["INF_COMP"].to_string());
    hmap.insert("CNPJ_CPF_PART".to_string(),      valores["CPF_CNPJ_ADQU"].to_string());

    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_f205(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(), info.global["DT_INI"].to_string());
    hmap.insert("VL_ITEM".to_string(),            valores["VL_CUS_INC_PER_ESC"].to_string());
    hmap.insert("Descr_Complementar".to_string(), valores["INF_COMP"].to_string());

    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_f210(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(),  info.global["DT_INI"].to_string());
    hmap.insert("VL_ITEM".to_string(), valores["VL_CUS_ORC"].to_string());
    hmap.insert("Descr_Complementar".to_string(), valores["INF_COMP"].to_string());

    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_f500(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    //  inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(),  info.global["DT_INI"].to_string());
    hmap.insert("VL_ITEM".to_string(), valores["VL_REC_CAIXA"].to_string());
    hmap.insert("Descr_Complementar".to_string(), valores["INFO_COMPL"].to_string());

    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_f510(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    //  inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(),  info.global["DT_INI"].to_string());
    hmap.insert("VL_ITEM".to_string(), valores["VL_REC_CAIXA"].to_string());
    hmap.insert("Descr_Complementar".to_string(), valores["INFO_COMPL"].to_string());

    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_f550(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(),  info.global["DT_INI"].to_string());
    hmap.insert("VL_ITEM".to_string(), valores["VL_REC_COMP"].to_string());
    hmap.insert("Descr_Complementar".to_string(), valores["INFO_COMPL"].to_string());

    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

fn ler_registro_f560(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(),  info.global["DT_INI"].to_string());
    hmap.insert("VL_ITEM".to_string(), valores["VL_REC_COMP"].to_string());
    hmap.insert("Descr_Complementar".to_string(), valores["INFO_COMPL"].to_string());

    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

// Registro I010: Identificação da Pessoa Juridica/Estabelecimento
fn ler_registro_i010(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    info.cnpj_do_estabelecimento = valores["CNPJ"].to_string();
    Ok(())
}

fn ler_registro_i100(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(),  info.global["DT_INI"].to_string());
    hmap.insert("VL_ITEM".to_string(), valores["VL_REC"].to_string());
    hmap.insert("Descr_Complementar".to_string(), valores["INFO_COMPL"].to_string());

    hmap.extend(valores);

    // adicionar informações do CNPJ do estabelecimento do contribuinte em hmap
    inserir_cnpj_do_estabelecimento(info, &mut hmap);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

/*
|M100|106|0|561234,72|0,5775|0||3275,36|0|0|0|3275,36|0|3275,36|0|
|M105|02|66|22446688|0|22446688|561234,72||0||
|M100|106|0|654321,6|0,99|0||4937,69|0|0|0|4937,69|0|4937,69|0|
|M105|02|66|33336666,46|0|33336666,46|654321,6||0||
|M100|206|0|8317541,95|0,5775|0||48033,8|0,01|0|0|48033,81|0|48033,81|0|
|M105|02|66|22446688|0|22446688|8317541,95||0||
|M100|206|0|7777777,05|0,99|0||72412,11|0|0,01|0|72412,1|0|72412,1|0|
|M105|02|66|33336666,46|0|33336666,46|7777777,05||0||
*/

// Registro M100: Crédito de PIS/Pasep Relativo ao Período
fn ler_registro_m100(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    info.reg_m100.clone_from(&valores); // reter info para posterior uso no reg M105

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    let mut ajuste_acres: f64 = valores["VL_AJUS_ACRES"].parse::<f64>().unwrap_or(0.0);
    let mut ajuste_reduc: f64 = valores["VL_AJUS_REDUC"].parse::<f64>().unwrap_or(0.0);
    let mut cred_descont: f64 = valores["VL_CRED_DESC" ].parse::<f64>().unwrap_or(0.0);

    ajuste_acres =        ajuste_acres.abs();
    ajuste_reduc = -1.0 * ajuste_reduc.abs();
    cred_descont = -1.0 * cred_descont.abs();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(), info.global["DT_INI"].to_string());

    //Adicionar valor da Base de Cálculo do PIS na coluna de COFINS (coluna padrão adotada)
    hmap.insert("VL_BC_COFINS".to_string(), valores["VL_BC_PIS"].to_string());

    hmap.extend(valores);

    if codigo_do_tipo_de_credito(info, &mut hmap, linha_da_efd).is_none() {
        return Ok(());
    }

    // println!("linha_da_efd: {linha_da_efd} ; ajuste_acres: {ajuste_acres:12} ; ajuste_reduc: {ajuste_reduc:12} ; cred_descont: {cred_descont:12}");

    // adicionar CNPJ da matriz do contribuinte
    inserir_cnpj_da_matriz(info, &mut hmap);

    // Para cada tipo de ajuste (acres ou reduc) inserir uma linha em info.completa
    // VL_CRED_DESC: Valor do Crédito disponível, descontado da contribuição apurada no próprio período.

    adicionar_ajuste_ou_desconto_em_info(info, &hmap, linha_da_efd, 3, ajuste_acres); // 3: Ajuste de Acréscimo
    adicionar_ajuste_ou_desconto_em_info(info, &hmap, linha_da_efd, 4, ajuste_reduc); // 4: Ajuste de Redução
    adicionar_ajuste_ou_desconto_em_info(info, &hmap, linha_da_efd, 5, cred_descont); // 5: Desconto da Contribuição Apurada no Próprio Período

    Ok(())
}

fn ler_registro_m105(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();

    hmap.extend(info.reg_m100.clone());
    hmap.extend(valores);

    let chave = obter_chave_de_correlacao_codcred(Pis, &hmap);
    let array = [hmap["ALIQ_PIS"].clone(), hmap["VL_BC_PIS"].clone()];
    info.correlacao.insert(chave, array);
    Ok(())
}

fn codigo_do_tipo_de_credito(info: &mut Info, hmap: &mut HashMap<String, String>, linha_da_efd: usize) -> Option<u16> {

    // De acordo com 4.3.6 – Tabela Código de Tipo de Crédito
    // Remainder operator (%): 101 % 100 = 1 and 308 % 100 = 8

    let cod_cred:    Option<u16> = hmap["COD_CRED"].parse::<u16>().ok(); // valor inteiro entre 101 e 499
    let cod_rateio:  Option<u16> = cod_cred.map(|cod| cod / 100);        // valor inteiro entre 1 e 4
    let cod_credito: Option<u16> = cod_cred.map(|cod| cod % 100);        // valor inteiro entre 1 e 9 ou 99

    if cod_cred.is_some() &&
       DESCRICAO_DO_TIPO_DE_RATEIO .get(&cod_rateio ).is_some() &&
       DESCRICAO_DO_TIPO_DE_CREDITO.get(&cod_credito).is_some()
    {
        if cod_credito == Some(8) {
            // Indicador da origem do crédito: 0 – Operação no Mercado Interno ; 1 – Operação de Importação
            hmap.insert("IND_ORIG_CRED".to_string(), "1".to_string());
        }
        cod_cred
    } else {
        let arquivo_efd = &info.global["arquivo_efd"];

        let msg01 = "Erro encontrado!!!".to_string();
        let msg02 = format!("Arquivo: {arquivo_efd}");
        let msg03 = format!("Número da linha: {linha_da_efd}");
        let msg04 = "COD_CRED inválido!\n\n".to_string();

        let msg = [msg01, msg02, msg03, msg04].join("\n");
        info.messages.push_str(&msg);

        None
    }
}

fn adicionar_ajuste_ou_desconto_em_info(info: &mut Info, hmap: &HashMap<String, String>, linha_da_efd: usize, tipo: u16, valor: f64) {

    // Em alguns casos, no registro M100 ou M500 ocorre simultaneamente a informação de ajuste de acréscimo, ajuste de redução e desconto
    // Nestes casos, procurar número de linha anterior não utilizada

    let mut linha_vazia = linha_da_efd;

    // Check if key exists in HashMap
    while info.completa.contains_key(&linha_vazia) {
        linha_vazia -= 1;
    }

    if valor.abs() > 0.0 {
        let mut hmap_ajuste_ou_desconto = hmap.clone();
        // Adicionar valor com sinal negativo em caso de ajuste de redução ou desconto
        hmap_ajuste_ou_desconto.insert("VL_ITEM".to_string(), valor.to_string());
        hmap_ajuste_ou_desconto.insert("tipo_de_operacao".to_string(), tipo.to_string());
        info.completa.insert(linha_vazia, hmap_ajuste_ou_desconto);
    }
}

// --- Correlacionar as alíquotas de PIS/PASEP com as alíquotas de COFINS --- //
// ---                               START                                --- //

fn obter_chave_de_correlacao_codcred(contrib: Tributos, valores: &HashMap<String, String>) -> String {

    let mut chave = String::new();

    if contrib == Pis {
        chave = format!("{}_{}_{}_{}", valores["COD_CRED"], valores["CST_PIS"], valores["NAT_BC_CRED"], &valores["VL_BC_PIS"]);
        //println!("chave_pis: {chave}");
    } else if contrib == Cofins {
        chave = format!("{}_{}_{}_{}", valores["COD_CRED"], valores["CST_COFINS"], valores["NAT_BC_CRED"], &valores["VL_BC_COFINS"]);
        //println!("chave_cof: {chave}");
    }

    chave
}

fn correlacionar_aliquotas_codcred(info: &mut Info, hmap: &mut HashMap<String, String>, linha_da_efd: usize) {

    match obter_aliquota_correlacionada_de_pis(&hmap["ALIQ_COFINS"]) {
        Some(aliq_pis) => {
            //println!("procedimento 1 ; aliq_cofins: {:06} ; aliq_pis: {:06}", &hmap["ALIQ_COFINS"], aliq_pis);
            hmap.insert("ALIQ_PIS".to_string(), aliq_pis);
        },
        None => {
            let chave = obter_chave_de_correlacao_codcred(Cofins, hmap);
            match info.correlacao.get(&chave) {
                Some(array) => {
                    let aliq_pis = array[0].to_string();
                    //println!("procedimento 2 ; aliq_cofins: {:06} ; aliq_pis: {:06}", &hmap["ALIQ_COFINS"], aliq_pis);
                    hmap.insert("ALIQ_PIS".to_string(), aliq_pis);
                }
                None => {
                    let arquivo_efd = &info.global["arquivo_efd"];
                    let registro = &hmap["REG"];
                    let aliq_cofins = &hmap["ALIQ_COFINS"];

                    let msg01 = "fn correlacionar_aliquotas_codcred(). Erro encontrado!!!".to_string();
                    let msg02 = format!("Arquivo: {arquivo_efd}");
                    let msg03 = format!("Número da linha: {linha_da_efd}");
                    let msg04 = format!("Registro {registro}.");
                    let msg05 = format!("Ausência de correlação entre as alíquotas de PIS/PASEP e COFINS ({aliq_cofins})!\n\n");

                    let msg = [msg01, msg02, msg03, msg04, msg05].join("\n");
                    info.messages.push_str(&msg);
                },
            };
        },
    };
}

// ---                               FINAL                                --- //
// --- Correlacionar as alíquotas de PIS/PASEP com as alíquotas de COFINS --- //

/// Registro M500: Crédito de COFINS Relativo ao Período
fn ler_registro_m500(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    info.reg_m500.clone_from(&valores); // reter info para posterior uso no reg M505

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    let mut ajuste_acres: f64 = valores["VL_AJUS_ACRES"].parse::<f64>().unwrap_or(0.0);
    let mut ajuste_reduc: f64 = valores["VL_AJUS_REDUC"].parse::<f64>().unwrap_or(0.0);
    let mut cred_descont: f64 = valores["VL_CRED_DESC" ].parse::<f64>().unwrap_or(0.0);

    ajuste_acres =        ajuste_acres.abs();
    ajuste_reduc = -1.0 * ajuste_reduc.abs();
    cred_descont = -1.0 * cred_descont.abs();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(), info.global["DT_INI"].to_string());

    hmap.extend(valores);

    if codigo_do_tipo_de_credito(info, &mut hmap, linha_da_efd).is_none() {
        return Ok(());
    }

    // println!("linha_da_efd: {linha_da_efd} ; ajuste_acres: {ajuste_acres:12} ; ajuste_reduc: {ajuste_reduc:12} ; cred_descont: {cred_descont:12}");

    // adicionar CNPJ da matriz do contribuinte
    inserir_cnpj_da_matriz(info, &mut hmap);

    // Para cada tipo de ajuste (acres ou reduc) inserir uma linha em info.completa
    // VL_CRED_DESC: Valor do Crédito disponível, descontado da contribuição apurada no próprio período.

    adicionar_ajuste_ou_desconto_em_info(info, &hmap, linha_da_efd, 3, ajuste_acres); // 3: Ajuste de Acréscimo
    adicionar_ajuste_ou_desconto_em_info(info, &hmap, linha_da_efd, 4, ajuste_reduc); // 4: Ajuste de Redução
    adicionar_ajuste_ou_desconto_em_info(info, &hmap, linha_da_efd, 5, cred_descont); // 5: Desconto da Contribuição Apurada no Próprio Período
    Ok(())
}

/// Registro M505: Detalhamento da Base de Calculo do Crédito Apurado no Período – Cofins
fn ler_registro_m505(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(), info.global["DT_INI"].to_string());

    // adicionar info do reg M500 em hmap
    hmap.extend(info.reg_m500.clone());

    // adicionar info do reg M505 hmap
    hmap.extend(valores);

    if codigo_do_tipo_de_credito(info, &mut hmap, linha_da_efd).is_none() {
        return Ok(());
    }

    // adicionar CNPJ da matriz do contribuinte
    inserir_cnpj_da_matriz(info, &mut hmap);

    hmap.insert("tipo_de_operacao".to_string(), "7".to_string()); // 7: Detalhamento

    // correlacionar aliquota de PIS com aliquota de COFINS
    correlacionar_aliquotas_codcred(info, &mut hmap, linha_da_efd);

    // finalmente, adicionar hmap em info.completa
    info.completa.insert(linha_da_efd, hmap);
    Ok(())
}

/*
|1001|0|
|1100|112016|01||201|519,73|0,00|519,73|2,36|188,78|0,00|328,59|0,00|0,00|0,00|0,00|0,00|328,59|
|1100|112019|01||101|125,12|0,00|125,12|114,29|0,00|0,00|10,83|0,00|0,00|0,00|0,00|0,00|10,83|
|1100|122020|01||101|250,51|0,00|250,51|0,00|0,00|0,00|250,51|33,03|0,00|0,00|0,00|0,00|217,48|

Registro 1500, campo 13: VL_CRED_DESC_EFD 'Valor do Crédito descontado neste período de escrituração'.
Somar apenas para Valor do Crédito Descontado, Apurado em Período de Apuração Anterior (recuperado do campo 13 do Registro 1500)
'PER_APU_CRED'    : Período de Apuração de Origem do Crédito MMAAAA
'VL_CRED_DESC_EFD': Valor do Crédito descontado neste período de escrituração
*/

/// Controle de Créditos Fiscais
///
/// Registro 1100: Controle de Créditos Fiscais – PIS/Pasep
///
/// Registro 1500: Controle de Créditos Fiscais – Cofins
fn ler_registro_de_controle_de_creditos_fiscais(info: &mut Info, valores: HashMap<String, String>) -> Result<(), Box<dyn Error>> {

    let contribuicao: Tributos = match valores["REG"].as_ref() {
        "1100" => Pis,
        "1500" => Cofins,
        _ => return Ok(()),
    };

    let mut hmap = HashMap::new();
    let linha_da_efd = valores["linha_da_efd"].parse::<usize>().unwrap();

    // inicialmente, padronizar nomes de alguns campos e adicionar em hmap
    hmap.insert("DT_DOC".to_string(), info.global["DT_INI"].to_string());

    let (credito_descontado, codigo_do_credito): (f64, u16) =
    match (valores["VL_CRED_DESC_EFD"].parse::<f64>(), valores["COD_CRED"].parse::<u16>()) {
        (Ok(vl_cred), Ok(cod_cred)) if vl_cred > 0.0 && CODIGO_TIPO_DE_CREDITO.binary_search(&cod_cred).is_ok()
        => (-1.0 * vl_cred.abs(), cod_cred),
        _ => return Ok(()),
    };

    if codigo_do_credito % 100 == 8 {
        // Indicador da origem do crédito: 0 – Operação no Mercado Interno ; 1 – Operação de Importação
        hmap.insert("IND_ORIG_CRED".to_string(), "1".to_string());
    }

    // O controle dos saldos dos créditos é realizado tendo por base o Período de Apuração de origem do crédito.
    // PER_APU_CRED : Período de Apuração de Origem do Crédito MMAAAA

    let pa_de_origem_do_credito: Option<NaiveDate> = if valores["PER_APU_CRED"].contains_num_digits(6) {
        let month = valores["PER_APU_CRED"][..2].parse::<u32>().unwrap();
        let year  = valores["PER_APU_CRED"][2..].parse::<i32>().unwrap();
        NaiveDate::from_ymd_opt(year, month, 1)
    } else {
        return Ok(())
    };

    // Filtrar 'Período de Escrituração Atual' != 'Período de Apuração de Origem do Crédito'

    match (info.pa, pa_de_origem_do_credito) {
        //(Some(pa_atual), Some(pa_origem)) if pa_atual.year() != pa_origem.year() || pa_atual.month() != pa_origem.month() => {
        (Some(pa_atual), Some(pa_origem)) if pa_atual != pa_origem => {
            let valor_formatado = thousands_separator(credito_descontado, DECIMAL_VALOR);
            let msg01 = "Verificado 'Valor do Crédito descontado neste período de escrituração', porém 'Crédito Apurado em Período de Apuração Anterior'.".to_string();
            let msg02 = format!("'Valor do Crédito descontado neste período de escrituração' --> Período de Escrituração Atual  = {:02}/{:04}.", pa_atual.month(),  pa_atual.year());
            let msg03 = format!("'Crédito Apurado em Período de Apuração Anterior' --> Período de Apuração de Origem do Crédito = {:02}/{:04}.", pa_origem.month(), pa_origem.year());
            let msg04 = format!("Código do Crédito: {codigo_do_credito}");
            let msg05 = format!("Valor das Deduções ou Descontos de {contribuicao}: {valor_formatado}\n\n");

            let msg = [msg01, msg02, msg03, msg04, msg05].join("\n");
            info.messages.push_str(&msg);
        },
        _ => return Ok(()),
    }

    hmap.extend(valores);

    // adicionar CNPJ da matriz do contribuinte
    inserir_cnpj_da_matriz(info, &mut hmap);

    adicionar_ajuste_ou_desconto_em_info(info, &hmap, linha_da_efd, 6, credito_descontado); // 6: Desconto Efetuado em Período Posterior
    Ok(())
}
