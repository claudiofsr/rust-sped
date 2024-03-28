use std::collections::{
    HashMap,
    HashSet,
};
use once_cell::sync::Lazy;

use crate::{
    DECIMAL_ALIQ,
    ALIQ_BASICA_PIS,
    ALIQ_BASICA_COF,
};

/// Example:
///
/// <https://docs.rs/once_cell/latest/once_cell/sync/struct.Lazy.html>
static NOME_DO_MES: Lazy<HashMap<Option<u32>, &'static str>> = Lazy::new(|| {
    let tuples = [
        ( 1, "janeiro"),
        ( 2, "fevereiro"),
        ( 3, "março"),
        ( 4, "abril"),
        ( 5, "maio"),
        ( 6, "junho"),
        ( 7, "julho"),
        ( 8, "agosto"),
        ( 9, "setembro"),
        (10, "outubro"),
        (11, "novembro"),
        (12, "dezembro"),
    ];

    let meses = tuples.map(|(n, mes)| (Some(n), mes));

    HashMap::from(meses)
});

/// Convert month number to month name
pub fn month_to_str(num: &Option<u32>) -> &'static str {

    let mes_nome = match NOME_DO_MES.get(num) {
        Some(mes) => mes,
        None => "",
    };

    mes_nome
}

static NUMERO_DO_MES: Lazy<HashMap<&'static str, Option<u32>>> = Lazy::new(|| {
    let tuples = [
        ( "janeiro", 1),
        ( "fevereiro", 2),
        ( "março", 3),
        ( "abril", 4),
        ( "maio", 5),
        ( "junho", 6),
        ( "julho", 7),
        ( "agosto", 8),
        ( "setembro", 9),
        ( "outubro", 10),
        ( "novembro", 11),
        ( "dezembro", 12),
    ];

    let meses = tuples.map(|(mes, n)| (mes, Some(n)));

    HashMap::from(meses)
});

/// Convert month name to month number
pub fn str_to_month(name: &str) -> Option<u32> {
    NUMERO_DO_MES
        .get(name)
        .and_then(|opt_integer| opt_integer.to_owned())
}

static INDICADOR_DE_ORIGEM: Lazy<HashMap<Option<u16>, &'static str>> = Lazy::new(|| {
    HashMap::from([
        (Some(0), "Operação no Mercado Interno"),
        (Some(1), "Operação de Importação"),
    ])
});

pub fn indicador_de_origem_to_str(num: &Option<u16>) -> &'static str {

    let origem = match INDICADOR_DE_ORIGEM.get(num) {
        Some(nome) => nome,
        None => "",
    };

    origem
}

static TIPO_DE_OPERACAO: Lazy<HashMap<Option<u16>, &'static str>> = Lazy::new(|| {
    HashMap::from([
        (Some(1), "Entrada"),
        (Some(2), "Saída"),
        (Some(3), "Ajuste"),   // "Ajuste de Acréscimo"
        (Some(4), "Ajuste"),   // "Ajuste de Redução"
        (Some(5), "Desconto"), // "Desconto da Contribuição Apurada no Próprio Período"
        (Some(6), "Desconto"), // "Desconto Efetuado em Período Posterior"
        (Some(7), "Detalhamento"),
    ])
});

pub fn tipo_de_operacao_to_str(num:&Option<u16>) -> &'static str {

    let tipo_de_operacao = match TIPO_DE_OPERACAO.get(num) {
        Some(tipo) => tipo,
        None => "",
    };

    tipo_de_operacao
}

/// 4.3.6 - Tabela Código de Tipo de Crédito
pub static DESCRICAO_DO_TIPO_DE_RATEIO: Lazy<HashMap<Option<u16>, &'static str>> = Lazy::new(|| {
    HashMap::from([
        (Some(1), "Receita Bruta Não Cumulativa: Tributada no Mercado Interno"),
        (Some(2), "Receita Bruta Não Cumulativa: Não Tributada no Mercado Interno"),
        (Some(3), "Receita Bruta Não Cumulativa: de Exportação"),
        (Some(4), "Receita Bruta Cumulativa"),
    ])
});

#[allow(dead_code)]
pub fn obter_tipo_de_rateio(cod: &Option<u16>) -> String {

    let rateio_descricao = match DESCRICAO_DO_TIPO_DE_RATEIO.get(cod) {
        Some(&descricao) => format!("{} - {}", cod.unwrap(), descricao),
        None => "".to_string(),
    };

    rateio_descricao
}

/// 4.3.6 – Tabela Código de Tipo de Crédito
///
/// 199, 299, 399: códigos encerrado a partir de 31/03/2023
///
/// A equipe da EFD-Contribuições informa a atualização da tabela 4.3.6 –
/// Código de tipo de crédito, com o encerramento de vigência dos
/// códigos 199, 299 e 399 (Outros) a ocorrer a partir de 31/03/2023.
pub const CODIGO_TIPO_DE_CREDITO: [u16; 28] = [
    101, 102, 103, 104, 105, 106, 107, 108, 109, 199,
    201, 202, 203, 204, 205, 206, 207, 208,      299,
    301, 302, 303, 304, 305, 306, 307, 308,      399,
];

pub fn registros_de_operacoes(text: &str) -> bool {
    // Blocos A, B, ..., I contêm registros de operações
    // Bloco M contém registros de apurações

    let first_char: Option<char> = text.chars().next();
    let test1: bool = matches!(first_char, Some('A' ..= 'I'));
    let test2: bool = text[1..].parse::<u32>().is_ok();
    test1 && test2
}

static TIPOS_DOS_ITENS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    // Registro 0200: Tabela de Identificação do Item (Produtos e Serviços)
    // 'TIPO_ITEM' Tipo do item - Atividades Industriais, Comerciais e Serviços:
    HashMap::from([
        ("00", "Mercadoria para Revenda"),
        ("01", "Materia-Prima"),
        ("02", "Embalagem"),
        ("03", "Produto em Processo"),
        ("04", "Produto Acabado"),
        ("05", "Subproduto"),
        ("06", "Produto Intermediario"),
        ("07", "Material de Uso e Consumo"),
        ("08", "Ativo Imobilizado"),
        ("09", "Servicos"),
        ("10", "Outros insumos"),
        ("99", "Outras"),
    ])
});

pub fn obter_tipo_do_item(codigo: &str) -> String {

    let tipo_do_item = match TIPOS_DOS_ITENS.get(codigo) {
        Some(&tipo) => [codigo, " - ", tipo].concat(),
        None => "".to_string(),
    };

    tipo_do_item
}

static GRUPO_DE_CONTAS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    // Registro 0500: Plano de Contas Contábeis
    // 'GRUPO_DE_CONTAS' Código da natureza da conta/grupo de contas:
    HashMap::from([
        ("01", "Contas de Ativo"),
        ("02", "Contas de Passivo"),
        ("03", "Patrimônio Líquido"),
        ("04", "Contas de Resultado"),
        ("05", "Contas de Compensação"),
        ("09", "Outras"),
    ])
});

pub fn obter_grupo_de_contas(codigo: &str) -> String {

    let grupo_de_contas = match GRUPO_DE_CONTAS.get(codigo) {
        Some(&contas) => [codigo, " - ", contas].concat(),
        None => "".to_string(),
    };

    grupo_de_contas
}

static CORRELACAO_DE_ALIQUOTAS: Lazy<HashMap<&'static str, f64>> = Lazy::new(|| {
    // Aliq COFINS --> Aliq PIS/PASEP
    HashMap::from([
        // if DECIMAL_ALIQ = 4
        ( "0.0000", 1.6500), // caso de Alíquotas Diferenciadas
        ( "0.7600", 0.1650),
        ( "1.5200", 0.3300),
        ( "2.6600", 0.5775),
        ( "3.8000", 0.8250),
        ( "4.5600", 0.9900),
        ( "6.0800", 1.3200),
        ( "7.6000", 1.6500),
        ( "8.5400", 1.8600), // venda pelo atacadista ao varejista ou ao consumidor final
        ( "9.6500", 2.1000),
        ("10.6800", 2.3200),
    ])
});

pub fn obter_aliquota_correlacionada_de_pis(aliq_cofins: &str) -> Option<String> {

    match CORRELACAO_DE_ALIQUOTAS.get(&aliq_cofins) {
        Some(&aliq_pis) => {
            // println!("aliq_cof: {aliq_cof} --> aliq_cofins: {aliq_cofins} relacionada aliquota_de_pis: {aliquota_de_pis}");
            let aliquota_de_pis = format!("{aliq_pis:0.DECIMAL_ALIQ$}");
            Some(aliquota_de_pis)
        },
        None => {
            eprintln!("sped_registros.rs");
            eprintln!("fn obter_aliquota_correlacionada_de_pis()");
            eprintln!("Não foi possível obter diretamente a alíquota de PIS/PASEP relacionada à alíquota de COFINS: {aliq_cofins}");
            eprintln!("A correlação entre as alíquotas de PIS/PASEP e COFINS será obtida da função correlacionar_aliquotas_codcred em dispatch_table.rs.\n");
            None
        },
    }
}

static MODELOS_DOCUMENTOS_FISCAIS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    // 4.1.1- Tabela Modelos de Documentos Fiscais
    HashMap::from([
        ("01", "Nota Fiscal"),
        ("1B", "Nota Fiscal Avulsa"),
        ("02", "Nota Fiscal de Venda a Consumidor"),
        ("2D", "Cupom Fiscal emitido por ECF"),
        ("2E", "Bilhete de Passagem emitido por ECF"),
        ("04", "Nota Fiscal de Produtor"),
        ("06", "Nota Fiscal / Conta de Energia Elétrica"),
        ("07", "Nota Fiscal de Serviço de Transporte"),
        ("08", "Conhecimento de Transporte Rodoviário de Cargas"),
        ("8B", "Conhecimento de Transporte de Cargas Avulso"),
        ("09", "Conhecimento de Transporte Aquaviário de Cargas"),
        ("10", "Conhecimento Aéreo"),
        ("11", "Conhecimento de Transporte Ferroviário de Cargas"),
        ("13", "Bilhete de Passagem Rodoviário"),
        ("14", "Bilhete de Passagem Aquaviário"),
        ("15", "Bilhete de Passagem e Nota de Bagagem"),
        ("17", "Despacho de Transporte"),
        ("16", "Bilhete de Passagem Ferroviário"),
        ("18", "Resumo de Movimento Diário"),
        ("20", "Ordem de Coleta de Cargas"),
        ("21", "Nota Fiscal de Serviço de Comunicação"),
        ("22", "Nota Fiscal de Serviço de Telecomunicação"),
        ("23", "GNRE"),
        ("24", "Autorização de Carregamento e Transporte"),
        ("25", "Manifesto de Carga"),
        ("26", "Conhecimento de Transporte Multimodal de Cargas"),
        ("27", "Nota Fiscal de Transporte Ferroviário de Cargas"),
        ("28", "Nota Fiscal / Conta de Fornecimento de Gás Canalizado"),
        ("29", "Nota Fiscal / Conta de Fornecimento de Água Canalizada"),
        ("30", "Bilhete / Recibo do Passageiro"),
        ("55", "Nota Fiscal Eletrônica: NF-e"),
        ("57", "Conhecimento de Transporte Eletrônico: CT-e"),
        ("59", "Cupom Fiscal Eletrônico: CF-e (CF-e-SAT)"),
        ("60", "Cupom Fiscal Eletrônico: CF-e-ECF"),
        ("63", "Bilhete de Passagem Eletrônico: BP-e"),
        ("65", "Nota Fiscal Eletrônica ao Consumidor Final: NFC-e"),
        ("66", "Nota Fiscal de Energia Elétrica Eletrônica: NF3e"),
        ("67", "Conhecimento de Transporte Eletrônico para Outros Serviços: CT-e OS"),
    ])
});

pub fn obter_modelo_do_documento_fiscal(codigo: &str) -> String {

    let modelo_do_documento_fiscal = match MODELOS_DOCUMENTOS_FISCAIS.get(codigo) {
        Some(&modelo) => [codigo, " - ", modelo].concat(),
        None => "".to_string(),
    };

    modelo_do_documento_fiscal
}

static CODIGO_DA_NATUREZA_DA_BC_DOS_CREDITOS: Lazy<HashMap<Option<u16>, u16>> = Lazy::new(|| {
    // Natureza da BC dos Créditos em função do CFOP
    let mut info: HashMap<Option<u16>, u16> = HashMap::new();

    let cfops_de_natureza01: [u16; 21] = [
        1102, 1113, 1117, 1118, 1121, 1159, 1251, 1403,
        1652, 2102, 2113, 2117, 2118, 2121, 2159, 2251,
        2403, 2652, 3102, 3251, 3652
    ];

    let cfops_de_natureza02: [u16; 36] = [
        1101, 1111, 1116, 1120, 1122, 1126, 1128, 1401,
        1407, 1556, 1651, 1653, 2101, 2111, 2116, 2120,
        2122, 2126, 2128, 2401, 2407, 2556, 2651, 2653,
        3101, 3126, 3128, 3556, 3651, 3653, 1135, 2135,
        1132, 2132, 1456, 2456
    ];

    let cfops_de_natureza03:[u16; 6] = [
        1124, 1125, 1933, 2124, 2125, 2933
    ];

    let cfops_de_natureza12: [u16; 24] = [
        1201, 1202, 1203, 1204, 1410, 1411, 1660, 1661,
        1662, 2201, 2202, 2410, 2411, 2660, 2661, 2662,
        1206, 2206, 1207, 2207, 1215, 1216, 2215, 2216
    ];

    let cfops_de_natureza13: [u16; 2] = [
        1922, 2922
    ];

    for cfop in cfops_de_natureza01 {
        info.insert(Some(cfop), 1);
    }
    for cfop in cfops_de_natureza02 {
        info.insert(Some(cfop), 2);
    }
    for cfop in cfops_de_natureza03 {
        info.insert(Some(cfop), 3);
    }
    for cfop in cfops_de_natureza12 {
        info.insert(Some(cfop), 12);
    }
    for cfop in cfops_de_natureza13 {
        info.insert(Some(cfop), 13);
    }

    info
});

/// Obter código da Natureza da Base de Cálculo
///
/// desde que:
///
/// 50 <= CST <= 56 ou 60 <= CST <= 66
///
/// e
///
/// CFOP seja um código de Operações com direito a créditos,
///
/// conforme Tabela “CFOP – Operações Geradoras de Créditos”.
pub fn obter_cod_da_natureza_da_bc(opt_cfop: &Option<u16>, opt_cst: Option<u16>) -> Option<u16> {

    match (CODIGO_DA_NATUREZA_DA_BC_DOS_CREDITOS.get(opt_cfop), opt_cst) {
        // if 50 <= cst <= 56 || 60 <= cst <= 66
        (Some(&cod_nat), Some(50..=56|60..=66)) => {
            Some(cod_nat)
        },
        _ => None
    }
}

static NATUREZA_DA_BASE_DE_CALCULO_DOS_CREDITOS: Lazy<HashMap<Option<u16>, &'static str>> = Lazy::new(|| {
    // array of tuples [T; length]
    // 4.3.7 - Tabela Base de Cálculo do Crédito (versão 1.0.1)
    let natureza_bc: [(u16, &str); 45] = [
        ( 1, "Aquisição de Bens para Revenda"),
        ( 2, "Aquisição de Bens Utilizados como Insumo"),
        ( 3, "Aquisição de Serviços Utilizados como Insumo"),
        ( 4, "Energia Elétrica e Térmica, Inclusive sob a Forma de Vapor"),
        ( 5, "Aluguéis de Prédios"),
        ( 6, "Aluguéis de Máquinas e Equipamentos"),
        ( 7, "Armazenagem de Mercadoria e Frete na Operação de Venda"),
        ( 8, "Contraprestações de Arrendamento Mercantil"),
        ( 9, "Máquinas, Equipamentos ... (Crédito sobre Encargos de Depreciação)"),
        (10, "Máquinas, Equipamentos ... (Crédito com Base no Valor de Aquisição)"),
        (11, "Amortizacao e Depreciação de Edificações e Benfeitorias em Imóveis"),
        (12, "Devolução de Vendas Sujeitas à Incidência Não-Cumulativa"),
        (13, "Outras Operações com Direito a Crédito"),
        (14, "Atividade de Transporte de Cargas - Subcontratação"),
        (15, "Atividade Imobiliária - Custo Incorrido de Unidade Imobiliária"),
        (16, "Atividade Imobiliária - Custo Orçado de Unidade não Concluída"),
        (17, "Atividade de Prestação de Serviços de Limpeza, Conservação e Manutenção"),
        (18, "Estoque de Abertura de Bens"),

        // Ajustes
        (31, "Ajuste de Acréscimo (PIS/PASEP)"),
        (35, "Ajuste de Acréscimo (COFINS)"),
        (41, "Ajuste de Redução (PIS/PASEP)"),
        (45, "Ajuste de Redução (COFINS)"),

        // Descontos
        (51, "Desconto da Contribuição Apurada no Próprio Período (PIS/PASEP)"),
        (55, "Desconto da Contribuição Apurada no Próprio Período (COFINS)"),
        (61, "Desconto Efetuado em Período Posterior (PIS/PASEP)"),
        (65, "Desconto Efetuado em Período Posterior (COFINS)"),

        // Base de Cálculo dos Créditos
        (101, "Base de Cálculo dos Créditos - Alíquota Básica (Soma)"),
        (102, "Base de Cálculo dos Créditos - Alíquotas Diferenciadas (Soma)"),
        (103, "Base de Cálculo dos Créditos - Alíquota por Unidade de Produto (Soma)"),
        (104, "Base de Cálculo dos Créditos - Estoque de Abertura (Soma)"),
        (105, "Base de Cálculo dos Créditos - Aquisição Embalagens para Revenda (Soma)"),
        (106, "Base de Cálculo dos Créditos - Presumido da Agroindústria (Soma)"),
        (107, "Base de Cálculo dos Créditos - Outros Créditos Presumidos (Soma)"),
        (108, "Base de Cálculo dos Créditos - Importação (Soma)"),
        (109, "Base de Cálculo dos Créditos - Atividade Imobiliária (Soma)"),
        (199, "Base de Cálculo dos Créditos - Outros (Soma)"),

        // Valor Total do Crédito Apurado no Período
        (201, "Crédito Apurado no Período (PIS/PASEP)"),
        (205, "Crédito Apurado no Período (COFINS)"),

        // Crédito Disponível após Ajustes
        (211, "Crédito Disponível após Ajustes (PIS/PASEP)"),
        (215, "Crédito Disponível após Ajustes (COFINS)"),

        // Crédito Disponível após Descontos
        (221, "Crédito Disponível após Descontos (PIS/PASEP)"),
        (225, "Crédito Disponível após Descontos (COFINS)"),

        // Saldo de Crédito Passível de Desconto ou Ressarcimento
        (300, "Base de Cálculo dos Créditos - Valor Total (Soma)"),
        (301, "Saldo de Crédito Passível de Desconto ou Ressarcimento (PIS/PASEP)"),
        (305, "Saldo de Crédito Passível de Desconto ou Ressarcimento (COFINS)"),
    ];

    let nat = natureza_bc.map(|(n, t)| (Some(n), t));

    HashMap::from(nat)
});

pub fn obter_descricao_da_natureza_da_bc_dos_creditos(cod: &Option<u16>) -> String {

    let natureza = match NATUREZA_DA_BASE_DE_CALCULO_DOS_CREDITOS.get(cod) {
        Some(&descricao) => {
            if *cod <= Some(18) {
                format!("{:02} - {}", cod.unwrap(), descricao)
            } else {
                //format!("{} - {}", cod.unwrap(), descricao)
                descricao.to_string()
            }
        },
        None => "".to_string(),
    };

    natureza
}

// https://doc.rust-lang.org/rust-by-example/std/hash/alt_key_types.html
#[derive(Debug, PartialEq, Eq, Hash)]
struct Aliquotas {
    pis: String,
    cof: String,
}

static ALIQUOTAS_DE_CRED_PRESUMIDO: Lazy<HashSet<Aliquotas>> = Lazy::new(|| {

    let mut hashset: HashSet<Aliquotas> = HashSet::new();

    let percentuais = [
        0.10, // Lei 12.599, Art. 5o, § 1o  # pis = 0.1650 ; confins = 0.7600 --> crédito presumido - exportação de café, produtos com ncm 0901.1
        0.12, // 3/25
        0.20, // Lei 10.925, Art. 8o, § 3o, inciso V.    # pis = 0.3300 ; confins = 1.5200
        0.35, // Lei 10.925, Art. 8o, § 3o, inciso III.  # pis = 0.5775 ; confins = 2.6600
        0.50, // Lei 10.925, Art. 8o, § 3o, inciso IV.   # pis = 0.8250 ; confins = 3.8000
        0.60, // Lei 10.925, Art. 8o, § 3o, inciso I.    # pis = 0.9900 ; confins = 4.5600
        0.80, // Lei 12.599, Art. 6o, § 2o  # pis = 1.3200 ; confins = 6.0800 --> crédito presumido - industrialização do café, aquisição dos produtos com ncm 0901.1 utilizados na elaboração dos produtos com 0901.2 e 2101.1
    ];

    // geralmente, aliq de cred presumido é um percentual da aliq básica
    for percentual in percentuais {

        let aliquotas = Aliquotas {
            pis: format!("{:0.DECIMAL_ALIQ$}", percentual * ALIQ_BASICA_PIS),
            cof: format!("{:0.DECIMAL_ALIQ$}", percentual * ALIQ_BASICA_COF),
        };

        hashset.insert(aliquotas);
    }

    //println!("hashset: {hashset:#?}");
    hashset
});

pub fn cred_presumido(aliq_pis: f64, aliq_cof: f64) -> bool {

    let aliquotas = Aliquotas {
        pis: format!("{aliq_pis:0.DECIMAL_ALIQ$}"),
        cof: format!("{aliq_cof:0.DECIMAL_ALIQ$}"),
    };

    ALIQUOTAS_DE_CRED_PRESUMIDO.contains(&aliquotas)
}

pub static DESCRICAO_DO_TIPO_DE_CREDITO: Lazy<HashMap<Option<u16>, &'static str>> = Lazy::new(|| {
    // array of tuples [T; length]
    // 4.3.6 - Tabela Código de Tipo de Crédito
    let cred_descricao: [(u16, &str); 11] = [
        (  1, "Alíquota Básica"),
        (  2, "Alíquotas Diferenciadas"),
        (  3, "Alíquota por Unidade de Produto"),
        (  4, "Estoque de Abertura"),
        (  5, "Aquisição Embalagens para Revenda"),
        (  6, "Presumido da Agroindústria"),
        (  7, "Outros Créditos Presumidos"),
        (  8, "Importação"),
        (  9, "Atividade Imobiliária"),
        ( 99, "Outros"), // código encerrado a partir de 31/03/2023
        (100, ""),
    ];

    let cred = cred_descricao.map(|(n, t)| (Some(n), t));

    HashMap::from(cred)
});

pub fn obter_descricao_do_tipo_de_credito(cod: &Option<u16>, formatar: bool) -> String {

    let cred_descricao = match DESCRICAO_DO_TIPO_DE_CREDITO.get(cod) {
        Some(&descricao) => {
            if formatar {
                format!("{:02} - {}", cod.unwrap(), descricao)
            } else {
                descricao.to_string()
            }
        },
        None => "".to_string(),
    };

    cred_descricao
}

static CODIGO_DA_SITUACAO_TRIBUTARIA: Lazy<HashMap<Option<u16>, &'static str>> = Lazy::new(|| {
    // array of tuples [T; length]
    // 4.3.4 - Tabela Código da Situação Tributária (CST)
    let cst_descricao: [(u16, &str); 33] = [
        ( 1, "Operação Tributável com Alíquota Básica"),
        ( 2, "Operação Tributável com Alíquota Diferenciada"),
        ( 3, "Operação Tributável com Alíquota por Unidade de Medida de Produto"),
        ( 4, "Operação Tributável Monofásica - Revenda a Alíquota Zero"),
        ( 5, "Operação Tributável por Substituição Tributária"),
        ( 6, "Operação Tributável a Alíquota Zero"),
        ( 7, "Operação Isenta da Contribuição"),
        ( 8, "Operação sem Incidência da Contribuição"),
        ( 9, "Operação com Suspensão da Contribuição"),
        (49, "Outras Operações de Saída"),
        (50, "Operação com Direito a Crédito - Vinculada Exclusivamente a Receita Tributada no Mercado Interno"),
        (51, "Operação com Direito a Crédito - Vinculada Exclusivamente a Receita Não-Tributada no Mercado Interno"),
        (52, "Operação com Direito a Crédito - Vinculada Exclusivamente a Receita de Exportação"),
        (53, "Operação com Direito a Crédito - Vinculada a Receitas Tributadas e Não-Tributadas no Mercado Interno"),
        (54, "Operação com Direito a Crédito - Vinculada a Receitas Tributadas no Mercado Interno e de Exportação"),
        (55, "Operação com Direito a Crédito - Vinculada a Receitas Não Tributadas no Mercado Interno e de Exportação"),
        (56, "Operação com Direito a Crédito - Vinculada a Receitas Tributadas e Não-Tributadas no Mercado Interno e de Exportação"),
        (60, "Crédito Presumido - Operação de Aquisição Vinculada Exclusivamente a Receita Tributada no Mercado Interno"),
        (61, "Crédito Presumido - Operação de Aquisição Vinculada Exclusivamente a Receita Não-Tributada no Mercado Interno"),
        (62, "Crédito Presumido - Operação de Aquisição Vinculada Exclusivamente a Receita de Exportação"),
        (63, "Crédito Presumido - Operação de Aquisição Vinculada a Receitas Tributadas e Não-Tributadas no Mercado Interno"),
        (64, "Crédito Presumido - Operação de Aquisição Vinculada a Receitas Tributadas no Mercado Interno e de Exportação"),
        (65, "Crédito Presumido - Operação de Aquisição Vinculada a Receitas Não-Tributadas no Mercado Interno e de Exportação"),
        (66, "Crédito Presumido - Operação de Aquisição Vinculada a Receitas Tributadas e Não-Tributadas no Mercado Interno e de Exportação"),
        (67, "Crédito Presumido - Outras Operações"),
        (70, "Operação de Aquisição sem Direito a Crédito"),
        (71, "Operação de Aquisição com Isenção"),
        (72, "Operação de Aquisição com Suspensão"),
        (73, "Operação de Aquisição a Alíquota Zero"),
        (74, "Operação de Aquisição sem Incidência da Contribuição"),
        (75, "Operação de Aquisição por Substituição Tributária"),
        (98, "Outras Operações de Entrada"),
        (99, "Outras Operações")
    ];

    let cst = cst_descricao.map(|(n, t)| (Some(n), t));

    HashMap::from(cst)
});

pub fn obter_descricao_do_cst(cst: &Option<u16>) -> String {

    let cst_descricao = match CODIGO_DA_SITUACAO_TRIBUTARIA.get(cst) {
        Some(&descricao) => format!("{:02} - {}", cst.unwrap(), descricao),
        None => "".to_string(),
    };

    cst_descricao
}

static CFOP_DESCRICAO_RESUMIDA: Lazy<HashMap<Option<u16>, &'static str>> = Lazy::new(|| {
    // array of tuples [T; length]
    // Tabela Código CFOP: Descrição Resumida
    // https://www.confaz.fazenda.gov.br/legislacao/ajustes/sinief/cfop_cvsn_70_vigente
    // ^(\d{4}) - (.*)$       |        $cfop_descricao_resumida{'$1'} = '$2';
    let cfop_descricao: [(u16, &str); 661] = [
        (1000, "ENTRADAS OU AQUISIÇÕES DE SERVIÇOS DO ESTADO"),
        (1100, "COMPRAS PARA INDUSTRIALIZAÇÃO, PRODUÇÃO RURAL, COMERCIALIZAÇÃO OU PRESTAÇÃO DE SERVIÇOS"),
        (1101, "Compra para industrialização ou produção rural"),
        (1102, "Compra para comercialização"),
        (1111, "Compra para industrialização de mercadoria recebida anteriormente em consignação industrial"),
        (1113, "Compra para comercialização, de mercadoria recebida anteriormente em consignação mercantil"),
        (1116, "Compra para industrialização ou produção rural originada de encomenda para recebimento futuro"),
        (1117, "Compra para comercialização originada de encomenda para recebimento futuro"),
        (1118, "Compra de mercadoria para comercialização pelo adquirente originário, entregue pelo vendedor remetente ao destinatário, em venda à ordem"),
        (1120, "Compra para industrialização, em venda à ordem, já recebida do vendedor remetente"),
        (1121, "Compra para comercialização, em venda à ordem, já recebida do vendedor remetente"),
        (1122, "Compra para industrialização em que a mercadoria foi remetida pelo fornecedor ao industrializador sem transitar pelo estabelecimento adquirente"),
        (1124, "Industrialização efetuada por outra empresa"),
        (1125, "Industrialização efetuada por outra empresa quando a mercadoria remetida para utilização no processo de industrialização não transitou pelo estabelecimento adquirente da mercadoria"),
        (1126, "Compra para utilização na prestação de serviço sujeita ao ICMS"),
        (1128, "Compra para utilização na prestação de serviço sujeita ao ISSQN"),
        (1131, "Entrada de mercadoria com previsão de posterior ajuste ou fixação de preço, decorrente de operação de ato cooperativo"),
        (1132, "Fixação de preço de produção do estabelecimento produtor, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço, em ato cooperativo, para comercialização"),
        (1135, "Fixação de preço de produção do estabelecimento produtor, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço, em ato cooperativo, para industrialização"),
        (1150, "TRANSFERÊNCIAS PARA INDUSTRIALIZAÇÃO, PRODUÇÃO RURAL, COMERCIALIZAÇÃO OU PRESTAÇÃO DE SERVIÇOS"),
        (1151, "Transferência para industrialização ou produção rural"),
        (1152, "Transferência para comercialização"),
        (1153, "Transferência de energia elétrica para distribuição"),
        (1154, "Transferência para utilização na prestação de serviço"),
        (1159, "Entrada decorrente do fornecimento de produto ou mercadoria de ato cooperativo"),
        (1200, "DEVOLUÇÕES DE VENDAS DE PRODUÇÃO PRÓPRIA, DE TERCEIROS OU ANULAÇÕES DE VALORES"),
        (1201, "Devolução de venda de produção do estabelecimento"),
        (1202, "Devolução de venda de mercadoria adquirida ou recebida de terceiros"),
        (1203, "Devolução de venda de produção do estabelecimento, destinada à Zona Franca de Manaus ou Áreas de Livre Comércio"),
        (1204, "Devolução de venda de mercadoria adquirida ou recebida de terceiros, destinada à Zona Franca de Manaus ou Áreas de Livre Comércio"),
        (1205, "Anulação de valor relativo à prestação de serviço de comunicação"),
        (1206, "Anulação de valor relativo à prestação de serviço de transporte"),
        (1207, "Anulação de valor relativo à venda de energia elétrica"),
        (1208, "Devolução de produção do estabelecimento, remetida em transferência"),
        (1209, "Devolução de mercadoria adquirida ou recebida de terceiros, remetida em transferência"),
        (1212, "Devolução de venda no mercado interno de mercadoria industrializada e insumo importado sob o Regime Aduaneiro Especial de Entreposto Industrial sob Controle Informatizado do Sistema Público de Escrituração Digital (Recof-Sped)"),
        (1213, "Devolução de remessa de produção do estabelecimento com previsão de posterior ajuste ou fixação de preço, em ato cooperativo"),
        (1214, "Devolução de fixação de preço de produção do estabelecimento produtor, de ato cooperativo"),
        (1250, "COMPRAS DE ENERGIA ELÉTRICA"),
        (1251, "Compra de energia elétrica para distribuição ou comercialização"),
        (1252, "Compra de energia elétrica por estabelecimento industrial"),
        (1253, "Compra de energia elétrica por estabelecimento comercial"),
        (1254, "Compra de energia elétrica por estabelecimento prestador de serviço de transporte"),
        (1255, "Compra de energia elétrica por estabelecimento prestador de serviço de comunicação"),
        (1256, "Compra de energia elétrica por estabelecimento de produtor rural"),
        (1257, "Compra de energia elétrica para consumo por demanda contratada"),
        (1300, "AQUISIÇÕES DE SERVIÇOS DE COMUNICAÇÃO"),
        (1301, "Aquisição de serviço de comunicação para execução de serviço da mesma natureza"),
        (1302, "Aquisição de serviço de comunicação por estabelecimento industrial"),
        (1303, "Aquisição de serviço de comunicação por estabelecimento comercial"),
        (1304, "Aquisição de serviço de comunicação por estabelecimento de prestador de serviço de transporte"),
        (1305, "Aquisição de serviço de comunicação por estabelecimento de geradora ou de distribuidora de energia elétrica"),
        (1306, "Aquisição de serviço de comunicação por estabelecimento de produtor rural"),
        (1350, "AQUISIÇÕES DE SERVIÇOS DE TRANSPORTE"),
        (1351, "Aquisição de serviço de transporte para execução de serviço da mesma natureza"),
        (1352, "Aquisição de serviço de transporte por estabelecimento industrial"),
        (1353, "Aquisição de serviço de transporte por estabelecimento comercial"),
        (1354, "Aquisição de serviço de transporte por estabelecimento de prestador de serviço de comunicação"),
        (1355, "Aquisição de serviço de transporte por estabelecimento de geradora ou de distribuidora de energia elétrica"),
        (1356, "Aquisição de serviço de transporte por estabelecimento de produtor rural"),
        (1360, "Aquisição de serviço de transporte por contribuinte substituto em relação ao serviço de transporte"),
        (1400, "ENTRADAS DE MERCADORIAS SUJEITAS AO REGIME DE SUBSTITUIÇÃO TRIBUTÁRIA"),
        (1401, "Compra para industrialização ou produção rural em operação com mercadoria sujeita ao regime de substituição tributária"),
        (1403, "Compra para comercialização em operação com mercadoria sujeita ao regime de substituição tributária"),
        (1406, "Compra de bem para o ativo imobilizado cuja mercadoria está sujeita ao regime de substituição tributária"),
        (1407, "Compra de mercadoria para uso ou consumo cuja mercadoria está sujeita ao regime de substituição tributária"),
        (1408, "Transferência para industrialização ou produção rural em operação com mercadoria sujeita ao regime de substituição tributária"),
        (1409, "Transferência para comercialização em operação com mercadoria sujeita ao regime de substituição tributária"),
        (1410, "Devolução de venda de produção do estabelecimento em operação com produto sujeito ao regime de substituição tributária"),
        (1411, "Devolução de venda de mercadoria adquirida ou recebida de terceiros em operação com mercadoria sujeita ao regime de substituição tributária"),
        (1414, "Retorno de produção do estabelecimento, remetida para venda fora do estabelecimento em operação com produto sujeito ao regime de substituição tributária"),
        (1415, "Retorno de mercadoria adquirida ou recebida de terceiros, remetida para venda fora do estabelecimento em operação com mercadoria sujeita ao regime de substituição tributária"),
        (1450, "SISTEMAS DE INTEGRAÇÃO"),
        (1451, "Retorno de animal do estabelecimento produtor"),
        (1452, "Retorno de insumo não utilizado na produção"),
        (1500, "ENTRADAS DE MERCADORIAS REMETIDAS PARA FORMAÇÃO DE LOTE OU COM FIM ESPECÍFICO DE EXPORTAÇÃO E EVENTUAIS DEVOLUÇÕES"),
        (1501, "Entrada de mercadoria recebida com fim específico de exportação"),
        (1503, "Entrada decorrente de devolução de produto remetido com fim específico de exportação, de produção do estabelecimento"),
        (1504, "Entrada decorrente de devolução de mercadoria remetida com fim específico de exportação, adquirida ou recebida de terceiros"),
        (1505, "Entrada decorrente de devolução de mercadorias remetidas para formação de lote de exportação, de produtos industrializados ou produzidos pelo próprio estabelecimento"),
        (1506, "Entrada decorrente de devolução de mercadorias, adquiridas ou recebidas de terceiros, remetidas para formação de lote de exportação"),
        (1550, "OPERAÇÕES COM BENS DE ATIVO IMOBILIZADO E MATERIAIS PARA USO OU CONSUMO"),
        (1551, "Compra de bem para o ativo imobilizado"),
        (1552, "Transferência de bem do ativo imobilizado"),
        (1553, "Devolução de venda de bem do ativo imobilizado"),
        (1554, "Retorno de bem do ativo imobilizado remetido para uso fora do estabelecimento"),
        (1555, "Entrada de bem do ativo imobilizado de terceiro, remetido para uso no estabelecimento"),
        (1556, "Compra de material para uso ou consumo"),
        (1557, "Transferência de material para uso ou consumo"),
        (1600, "CRÉDITOS E RESSARCIMENTOS DE ICMS"),
        (1601, "Recebimento, por transferência, de crédito de ICMS"),
        (1602, "Recebimento, por transferência, de saldo credor de ICMS de outro estabelecimento da mesma empresa, para compensação de saldo devedor de ICMS"),
        (1603, "Ressarcimento de ICMS retido por substituição tributária"),
        (1604, "Lançamento do crédito relativo à compra de bem para o ativo imobilizado"),
        (1605, "Recebimento, por transferência, de saldo devedor de ICMS de outro estabelecimento da mesma empresa"),
        (1650, "ENTRADAS DE COMBUSTÍVEIS, DERIVADOS OU NÃO DE PETRÓLEO E LUBRIFICANTES"),
        (1651, "Compra de combustível ou lubrificante para industrialização subseqüente"),
        (1652, "Compra de combustível ou lubrificante para comercialização"),
        (1653, "Compra de combustível ou lubrificante por consumidor ou usuário final"),
        (1658, "Transferência de combustível e lubrificante para industrialização"),
        (1659, "Transferência de combustível e lubrificante para comercialização"),
        (1660, "Devolução de venda de combustível ou lubrificante destinado à industrialização subseqüente"),
        (1661, "Devolução de venda de combustível ou lubrificante destinado à comercialização"),
        (1662, "Devolução de venda de combustível ou lubrificante destinado a consumidor ou usuário final"),
        (1663, "Entrada de combustível ou lubrificante para armazenagem"),
        (1664, "Retorno de combustível ou lubrificante remetido para armazenagem"),
        (1900, "OUTRAS ENTRADAS DE MERCADORIAS OU AQUISIÇÕES DE SERVIÇOS"),
        (1901, "Entrada para industrialização por encomenda"),
        (1902, "Retorno de mercadoria remetida para industrialização por encomenda"),
        (1903, "Entrada de mercadoria remetida para industrialização e não aplicada no referido processo"),
        (1904, "Retorno de remessa para venda fora do estabelecimento"),
        (1905, "Entrada de mercadoria recebida para depósito em depósito fechado ou armazém geral"),
        (1906, "Retorno de mercadoria remetida para depósito fechado ou armazém geral"),
        (1907, "Retorno simbólico de mercadoria remetida para depósito fechado ou armazém geral"),
        (1908, "Entrada de bem por conta de contrato de comodato"),
        (1909, "Retorno de bem remetido por conta de contrato de comodato"),
        (1910, "Entrada de bonificação, doação ou brinde"),
        (1911, "Entrada de amostra grátis"),
        (1912, "Entrada de mercadoria ou bem recebido para demonstração ou mostruário"),
        (1913, "Retorno de mercadoria ou bem remetido para demonstração, mostruário ou treinamento"),
        (1914, "Retorno de mercadoria ou bem remetido para exposição ou feira"),
        (1915, "Entrada de mercadoria ou bem recebido para conserto ou reparo"),
        (1916, "Retorno de mercadoria ou bem remetido para conserto ou reparo"),
        (1917, "Entrada de mercadoria recebida em consignação mercantil ou industrial"),
        (1918, "Devolução de mercadoria remetida em consignação mercantil ou industrial"),
        (1919, "Devolução simbólica de mercadoria vendida ou utilizada em processo industrial, remetida anteriormente em consignação mercantil ou industrial"),
        (1920, "Entrada de vasilhame ou sacaria"),
        (1921, "Retorno de vasilhame ou sacaria"),
        (1922, "Lançamento efetuado a título de simples faturamento decorrente de compra para recebimento futuro"),
        (1923, "Entrada de mercadoria recebida do vendedor remetente, em venda à ordem"),
        (1924, "Entrada para industrialização por conta e ordem do adquirente da mercadoria, quando esta não transitar pelo estabelecimento do adquirente"),
        (1925, "Retorno de mercadoria remetida para industrialização por conta e ordem do adquirente da mercadoria, quando esta não transitar pelo estabelecimento do adquirente"),
        (1926, "Lançamento efetuado a título de reclassificação de mercadoria decorrente de formação de kit ou de sua desagregação"),
        (1931, "Lançamento efetuado pelo tomador do serviço de transporte quando a responsabilidade de retenção do imposto for atribuída ao remetente ou alienante da mercadoria, pelo serviço de transporte realizado por transportador autônomo ou por transportador não inscrito na unidade da Federação onde iniciado o serviço"),
        (1932, "Aquisição de serviço de transporte iniciado em unidade da Federação diversa daquela onde inscrito o prestador"),
        (1933, "Aquisição de serviço tributado pelo ISSQN"),
        (1934, "Entrada simbólica de mercadoria recebida para depósito fechado ou armazém geral"),
        (1949, "Outra entrada de mercadoria ou prestação de serviço não especificada"),
        (2000, "ENTRADAS OU AQUISIÇÕES DE SERVIÇOS DE OUTROS ESTADOS"),
        (2100, "COMPRAS PARA INDUSTRIALIZAÇÃO, PRODUÇÃO RURAL, COMERCIALIZAÇÃO OU PRESTAÇÃO DE SERVIÇOS"),
        (2101, "Compra para industrialização ou produção rural"),
        (2102, "Compra para comercialização"),
        (2111, "Compra para industrialização de mercadoria recebida anteriormente em consignação industrial"),
        (2113, "Compra para comercialização, de mercadoria recebida anteriormente em consignação mercantil"),
        (2116, "Compra para industrialização ou produção rural originada de encomenda para recebimento futuro"),
        (2117, "Compra para comercialização originada de encomenda para recebimento futuro"),
        (2118, "Compra de mercadoria para comercialização pelo adquirente originário, entregue pelo vendedor remetente ao destinatário, em venda à ordem"),
        (2120, "Compra para industrialização, em venda à ordem, já recebida do vendedor remetente"),
        (2121, "Compra para comercialização, em venda à ordem, já recebida do vendedor remetente"),
        (2122, "Compra para industrialização em que a mercadoria foi remetida pelo fornecedor ao industrializador sem transitar pelo estabelecimento adquirente"),
        (2124, "Industrialização efetuada por outra empresa"),
        (2125, "Industrialização efetuada por outra empresa quando a mercadoria remetida para utilização no processo de industrialização não transitou pelo estabelecimento adquirente da mercadoria"),
        (2126, "Compra para utilização na prestação de serviço sujeita ao ICMS"),
        (2128, "Compra para utilização na prestação de serviço sujeita ao ISSQN"),
        (2131, "Entrada de mercadoria com previsão de posterior ajuste ou fixação de preço, decorrente de operação de ato cooperativo"),
        (2132, "Fixação de preço de produção do estabelecimento produtor, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço, em ato cooperativo, para comercialização"),
        (2135, "Fixação de preço de produção do estabelecimento produtor, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço, em ato cooperativo, para industrialização"),
        (2150, "TRANSFERÊNCIAS PARA INDUSTRIALIZAÇÃO, PRODUÇÃO RURAL, COMERCIALIZAÇÃO OU PRESTAÇÃO DE SERVIÇOS"),
        (2151, "Transferência para industrialização ou produção rural"),
        (2152, "Transferência para comercialização"),
        (2153, "Transferência de energia elétrica para distribuição"),
        (2154, "Transferência para utilização na prestação de serviço"),
        (2159, "Entrada decorrente do fornecimento de produto ou mercadoria de ato cooperativo"),
        (2200, "DEVOLUÇÕES DE VENDAS DE PRODUÇÃO PRÓPRIA, DE TERCEIROS OU ANULAÇÕES DE VALORES"),
        (2201, "Devolução de venda de produção do estabelecimento"),
        (2202, "Devolução de venda de mercadoria adquirida ou recebida de terceiros"),
        (2203, "Devolução de venda de produção do estabelecimento, destinada à Zona Franca de Manaus ou Áreas de Livre Comércio"),
        (2204, "Devolução de venda de mercadoria adquirida ou recebida de terceiros, destinada à Zona Franca de Manaus ou Áreas de Livre Comércio"),
        (2205, "Anulação de valor relativo à prestação de serviço de comunicação"),
        (2206, "Anulação de valor relativo à prestação de serviço de transporte"),
        (2207, "Anulação de valor relativo à venda de energia elétrica"),
        (2208, "Devolução de produção do estabelecimento, remetida em transferência"),
        (2209, "Devolução de mercadoria adquirida ou recebida de terceiros, remetida em transferência"),
        (2212, "Devolução de venda no mercado interno de mercadoria industrializada e insumo importado sob o Regime Aduaneiro Especial de Entreposto Industrial sob Controle Informatizado do Sistema Público de Escrituração Digital (Recof-Sped)"),
        (2213, "Devolução de remessa de produção do estabelecimento com previsão de posterior ajuste ou fixação de preço, em ato cooperativo"),
        (2214, "Devolução de fixação de preço de produção do estabelecimento produtor, de ato cooperativo"),
        (2250, "COMPRAS DE ENERGIA ELÉTRICA"),
        (2251, "Compra de energia elétrica para distribuição ou comercialização"),
        (2252, "Compra de energia elétrica por estabelecimento industrial"),
        (2253, "Compra de energia elétrica por estabelecimento comercial"),
        (2254, "Compra de energia elétrica por estabelecimento prestador de serviço de transporte"),
        (2255, "Compra de energia elétrica por estabelecimento prestador de serviço de comunicação"),
        (2256, "Compra de energia elétrica por estabelecimento de produtor rural"),
        (2257, "Compra de energia elétrica para consumo por demanda contratada"),
        (2300, "AQUISIÇÕES DE SERVIÇOS DE COMUNICAÇÃO"),
        (2301, "Aquisição de serviço de comunicação para execução de serviço da mesma natureza"),
        (2302, "Aquisição de serviço de comunicação por estabelecimento industrial"),
        (2303, "Aquisição de serviço de comunicação por estabelecimento comercial"),
        (2304, "Aquisição de serviço de comunicação por estabelecimento de prestador de serviço de transporte"),
        (2305, "Aquisição de serviço de comunicação por estabelecimento de geradora ou de distribuidora de energia elétrica"),
        (2306, "Aquisição de serviço de comunicação por estabelecimento de produtor rural"),
        (2350, "AQUISIÇÕES DE SERVIÇOS DE TRANSPORTE"),
        (2351, "Aquisição de serviço de transporte para execução de serviço da mesma natureza"),
        (2352, "Aquisição de serviço de transporte por estabelecimento industrial"),
        (2353, "Aquisição de serviço de transporte por estabelecimento comercial"),
        (2354, "Aquisição de serviço de transporte por estabelecimento de prestador de serviço de comunicação"),
        (2355, "Aquisição de serviço de transporte por estabelecimento de geradora ou de distribuidora de energia elétrica"),
        (2356, "Aquisição de serviço de transporte por estabelecimento de produtor rural"),
        (2400, "ENTRADAS DE MERCADORIAS SUJEITAS AO REGIME DE SUBSTITUIÇÃO TRIBUTÁRIA"),
        (2401, "Compra para industrialização ou produção rural em operação com mercadoria sujeita ao regime de substituição tributária"),
        (2403, "Compra para comercialização em operação com mercadoria sujeita ao regime de substituição tributária"),
        (2406, "Compra de bem para o ativo imobilizado cuja mercadoria está sujeita ao regime de substituição tributária"),
        (2407, "Compra de mercadoria para uso ou consumo cuja mercadoria está sujeita ao regime de substituição tributária"),
        (2408, "Transferência para industrialização ou produção rural em operação com mercadoria sujeita ao regime de substituição tributária"),
        (2409, "Transferência para comercialização em operação com mercadoria sujeita ao regime de substituição tributária"),
        (2410, "Devolução de venda de produção do estabelecimento em operação com produto sujeito ao regime de substituição tributária"),
        (2411, "Devolução de venda de mercadoria adquirida ou recebida de terceiros em operação com mercadoria sujeita ao regime de substituição tributária"),
        (2414, "Retorno de produção do estabelecimento, remetida para venda fora do estabelecimento em operação com produto sujeito ao regime de substituição tributária"),
        (2415, "Retorno de mercadoria adquirida ou recebida de terceiros, remetida para venda fora do estabelecimento em operação com mercadoria sujeita ao regime de substituição tributária"),
        (2500, "ENTRADAS DE MERCADORIAS REMETIDAS PARA FORMAÇÃO DE LOTE OU COM FIM ESPECÍFICO DE EXPORTAÇÃO E EVENTUAIS DEVOLUÇÕES"),
        (2501, "Entrada de mercadoria recebida com fim específico de exportação"),
        (2503, "Entrada decorrente de devolução de produto remetido com fim específico de exportação, de produção do estabelecimento"),
        (2504, "Entrada decorrente de devolução de mercadoria remetida com fim específico de exportação, adquirida ou recebida de terceiros"),
        (2505, "Entrada decorrente de devolução de mercadorias remetidas para formação de lote de exportação, de produtos industrializados ou produzidos pelo próprio estabelecimento"),
        (2506, "Entrada decorrente de devolução de mercadorias, adquiridas ou recebidas de terceiros, remetidas para formação de lote de exportação"),
        (2550, "OPERAÇÕES COM BENS DE ATIVO IMOBILIZADO E MATERIAIS PARA USO OU CONSUMO"),
        (2551, "Compra de bem para o ativo imobilizado"),
        (2552, "Transferência de bem do ativo imobilizado"),
        (2553, "Devolução de venda de bem do ativo imobilizado"),
        (2554, "Retorno de bem do ativo imobilizado remetido para uso fora do estabelecimento"),
        (2555, "Entrada de bem do ativo imobilizado de terceiro, remetido para uso no estabelecimento"),
        (2556, "Compra de material para uso ou consumo"),
        (2557, "Transferência de material para uso ou consumo"),
        (2600, "CRÉDITOS E RESSARCIMENTOS DE ICMS"),
        (2603, "Ressarcimento de ICMS retido por substituição tributária"),
        (2650, "ENTRADAS DE COMBUSTÍVEIS, DERIVADOS OU NÃO DE PETRÓLEO E LUBRIFICANTES"),
        (2651, "Compra de combustível ou lubrificante para industrialização subseqüente"),
        (2652, "Compra de combustível ou lubrificante para comercialização"),
        (2653, "Compra de combustível ou lubrificante por consumidor ou usuário final"),
        (2658, "Transferência de combustível e lubrificante para industrialização"),
        (2659, "Transferência de combustível e lubrificante para comercialização"),
        (2660, "Devolução de venda de combustível ou lubrificante destinado à industrialização subseqüente"),
        (2661, "Devolução de venda de combustível ou lubrificante destinado à comercialização"),
        (2662, "Devolução de venda de combustível ou lubrificante destinado a consumidor ou usuário final"),
        (2663, "Entrada de combustível ou lubrificante para armazenagem"),
        (2664, "Retorno de combustível ou lubrificante remetido para armazenagem"),
        (2900, "OUTRAS ENTRADAS DE MERCADORIAS OU AQUISIÇÕES DE SERVIÇOS"),
        (2901, "Entrada para industrialização por encomenda"),
        (2902, "Retorno de mercadoria remetida para industrialização por encomenda"),
        (2903, "Entrada de mercadoria remetida para industrialização e não aplicada no referido processo"),
        (2904, "Retorno de remessa para venda fora do estabelecimento"),
        (2905, "Entrada de mercadoria recebida para depósito em depósito fechado ou armazém geral"),
        (2906, "Retorno de mercadoria remetida para depósito fechado ou armazém geral"),
        (2907, "Retorno simbólico de mercadoria remetida para depósito fechado ou armazém geral"),
        (2908, "Entrada de bem por conta de contrato de comodato"),
        (2909, "Retorno de bem remetido por conta de contrato de comodato"),
        (2910, "Entrada de bonificação, doação ou brinde"),
        (2911, "Entrada de amostra grátis"),
        (2912, "Entrada de mercadoria ou bem recebido para demonstração ou mostruário"),
        (2913, "Retorno de mercadoria ou bem remetido para demonstração, mostruário ou treinamento"),
        (2914, "Retorno de mercadoria ou bem remetido para exposição ou feira"),
        (2915, "Entrada de mercadoria ou bem recebido para conserto ou reparo"),
        (2916, "Retorno de mercadoria ou bem remetido para conserto ou reparo"),
        (2917, "Entrada de mercadoria recebida em consignação mercantil ou industrial"),
        (2918, "Devolução de mercadoria remetida em consignação mercantil ou industrial"),
        (2919, "Devolução simbólica de mercadoria vendida ou utilizada em processo industrial, remetida anteriormente em consignação mercantil ou industrial"),
        (2920, "Entrada de vasilhame ou sacaria"),
        (2921, "Retorno de vasilhame ou sacaria"),
        (2922, "Lançamento efetuado a título de simples faturamento decorrente de compra para recebimento futuro"),
        (2923, "Entrada de mercadoria recebida do vendedor remetente, em venda à ordem"),
        (2924, "Entrada para industrialização por conta e ordem do adquirente da mercadoria, quando esta não transitar pelo estabelecimento do adquirente"),
        (2925, "Retorno de mercadoria remetida para industrialização por conta e ordem do adquirente da mercadoria, quando esta não transitar pelo estabelecimento do adquirente"),
        (2931, "Lançamento efetuado pelo tomador do serviço de transporte quando a responsabilidade de retenção do imposto for atribuída ao remetente ou alienante da mercadoria, pelo serviço de transporte realizado por transportador autônomo ou por transportador não inscrito na unidade da Federação onde iniciado o serviço"),
        (2932, "Aquisição de serviço de transporte iniciado em unidade da Federação diversa daquela onde inscrito o prestador"),
        (2933, "Aquisição de serviço tributado pelo ISSQN"),
        (2934, "Entrada simbólica de mercadoria recebida para depósito fechado ou armazém geral"),
        (2949, "Outra entrada de mercadoria ou prestação de serviço não especificado"),
        (3000, "ENTRADAS OU AQUISIÇÕES DE SERVIÇOS DO EXTERIOR"),
        (3100, "COMPRAS PARA INDUSTRIALIZAÇÃO, PRODUÇÃO RURAL, COMERCIALIZAÇÃO OU PRESTAÇÃO DE SERVIÇOS"),
        (3101, "Compra para industrialização ou produção rural"),
        (3102, "Compra para comercialização"),
        (3126, "Compra para utilização na prestação de serviço sujeita ao ICMS"),
        (3127, "Compra para industrialização sob o regime de 'drawback'"),
        (3128, "Compra para utilização na prestação de serviço sujeita ao ISSQN"),
        (3129, "Compra para industrialização sob o Regime Aduaneiro Especial de Entreposto Industrial sob Controle Informatizado do Sistema Público de Escrituração Digital (Recof-Sped)"),
        (3200, "DEVOLUÇÕES DE VENDAS DE PRODUÇÃO PRÓPRIA, DE TERCEIROS OU ANULAÇÕES DE VALORES"),
        (3201, "Devolução de venda de produção do estabelecimento"),
        (3202, "Devolução de venda de mercadoria adquirida ou recebida de terceiros"),
        (3205, "Anulação de valor relativo à prestação de serviço de comunicação"),
        (3206, "Anulação de valor relativo à prestação de serviço de transporte"),
        (3207, "Anulação de valor relativo à venda de energia elétrica"),
        (3211, "Devolução de venda de produção do estabelecimento sob o regime de 'drawback'"),
        (3212, "Devolução de venda no mercado externo de mercadoria industrializada sob o Regime Aduaneiro Especial de Entreposto Industrial sob Controle Informatizado do Sistema Público de Escrituração Digital (Recof-Sped)"),
        (3250, "COMPRAS DE ENERGIA ELÉTRICA"),
        (3251, "Compra de energia elétrica para distribuição ou comercialização"),
        (3300, "AQUISIÇÕES DE SERVIÇOS DE COMUNICAÇÃO"),
        (3301, "Aquisição de serviço de comunicação para execução de serviço da mesma natureza"),
        (3350, "AQUISIÇÕES DE SERVIÇOS DE TRANSPORTE"),
        (3351, "Aquisição de serviço de transporte para execução de serviço da mesma natureza"),
        (3352, "Aquisição de serviço de transporte por estabelecimento industrial"),
        (3353, "Aquisição de serviço de transporte por estabelecimento comercial"),
        (3354, "Aquisição de serviço de transporte por estabelecimento de prestador de serviço de comunicação"),
        (3355, "Aquisição de serviço de transporte por estabelecimento de geradora ou de distribuidora de energia elétrica"),
        (3356, "Aquisição de serviço de transporte por estabelecimento de produtor rural"),
        (3500, "ENTRADAS DE MERCADORIAS REMETIDAS COM FIM ESPECÍFICO DE EXPORTAÇÃO E EVENTUAIS DEVOLUÇÕES"),
        (3503, "Devolução de mercadoria exportada que tenha sido recebida com fim específico de exportação"),
        (3550, "OPERAÇÕES COM BENS DE ATIVO IMOBILIZADO E MATERIAIS PARA USO OU CONSUMO"),
        (3551, "Compra de bem para o ativo imobilizado"),
        (3553, "Devolução de venda de bem do ativo imobilizado"),
        (3556, "Compra de material para uso ou consumo"),
        (3650, "ENTRADAS DE COMBUSTÍVEIS, DERIVADOS OU NÃO DE PETRÓLEO E LUBRIFICANTES"),
        (3651, "Compra de combustível ou lubrificante para industrialização subseqüente"),
        (3652, "Compra de combustível ou lubrificante para comercialização"),
        (3653, "Compra de combustível ou lubrificante por consumidor ou usuário final"),
        (3900, "OUTRAS ENTRADAS DE MERCADORIAS OU AQUISIÇÕES DE SERVIÇOS"),
        (3930, "Lançamento efetuado a título de entrada de bem sob amparo de regime especial aduaneiro de admissão temporária"),
        (3949, "Outra entrada de mercadoria ou prestação de serviço não especificado"),
        (5000, "SAÍDAS OU PRESTAÇÕES DE SERVIÇOS PARA O ESTADO"),
        (5100, "VENDAS DE PRODUÇÃO PRÓPRIA OU DE TERCEIROS"),
        (5101, "Venda de produção do estabelecimento"),
        (5102, "Venda de mercadoria adquirida ou recebida de terceiros"),
        (5103, "Venda de produção do estabelecimento, efetuada fora do estabelecimento"),
        (5104, "Venda de mercadoria adquirida ou recebida de terceiros, efetuada fora do estabelecimento"),
        (5105, "Venda de produção do estabelecimento que não deva por ele transitar"),
        (5106, "Venda de mercadoria adquirida ou recebida de terceiros, que não deva por ele transitar"),
        (5109, "Venda de produção do estabelecimento, destinada à Zona Franca de Manaus ou Áreas de Livre Comércio"),
        (5110, "Venda de mercadoria adquirida ou recebida de terceiros, destinada à Zona Franca de Manaus ou Áreas de Livre Comércio"),
        (5111, "Venda de produção do estabelecimento remetida anteriormente em consignação industrial"),
        (5112, "Venda de mercadoria adquirida ou recebida de terceiros remetida anteriormente em consignação industrial"),
        (5113, "Venda de produção do estabelecimento remetida anteriormente em consignação mercantil"),
        (5114, "Venda de mercadoria adquirida ou recebida de terceiros remetida anteriormente em consignação mercantil"),
        (5115, "Venda de mercadoria adquirida ou recebida de terceiros, recebida anteriormente em consignação mercantil"),
        (5116, "Venda de produção do estabelecimento originada de encomenda para entrega futura"),
        (5117, "Venda de mercadoria adquirida ou recebida de terceiros, originada de encomenda para entrega futura"),
        (5118, "Venda de produção do estabelecimento entregue ao destinatário por conta e ordem do adquirente originário, em venda à ordem"),
        (5119, "Venda de mercadoria adquirida ou recebida de terceiros entregue ao destinatário por conta e ordem do adquirente originário, em venda à ordem"),
        (5120, "Venda de mercadoria adquirida ou recebida de terceiros entregue ao destinatário pelo vendedor remetente, em venda à ordem"),
        (5122, "Venda de produção do estabelecimento remetida para industrialização, por conta e ordem do adquirente, sem transitar pelo estabelecimento do adquirente"),
        (5123, "Venda de mercadoria adquirida ou recebida de terceiros remetida para industrialização, por conta e ordem do adquirente, sem transitar pelo estabelecimento do adquirente"),
        (5124, "Industrialização efetuada para outra empresa"),
        (5125, "Industrialização efetuada para outra empresa quando a mercadoria recebida para utilização no processo de industrialização não transitar pelo estabelecimento adquirente da mercadoria"),
        (5129, "Venda de insumo importado e de mercadoria industrializada sob o amparo do Regime Aduaneiro Especial de Entreposto Industrial sob Controle Informatizado do Sistema Público de Escrituração Digital (Recof-Sped)"),
        (5131, "Remessa de produção do estabelecimento, com previsão de posterior ajuste ou fixação de preço, de ato cooperativo"),
        (5132, "Fixação de preço de produção do estabelecimento, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço de ato cooperativo"),
        (5150, "TRANSFERÊNCIAS DE PRODUÇÃO PRÓPRIA OU DE TERCEIROS"),
        (5151, "Transferência de produção do estabelecimento"),
        (5152, "Transferência de mercadoria adquirida ou recebida de terceiros"),
        (5153, "Transferência de energia elétrica"),
        (5155, "Transferência de produção do estabelecimento, que não deva por ele transitar"),
        (5156, "Transferência de mercadoria adquirida ou recebida de terceiros, que não deva por ele transitar"),
        (5159, "Fornecimento de produção do estabelecimento de ato cooperativo"),
        (5160, "Fornecimento de mercadoria adquirida ou recebida de terceiros de ato cooperativo"),
        (5200, "DEVOLUÇÕES DE COMPRAS PARA INDUSTRIALIZAÇÃO, PRODUÇÃO RURAL, COMERCIALIZAÇÃO OU ANULAÇÕES DE VALORES"),
        (5201, "Devolução de compra para industrialização ou produção rural"),
        (5202, "Devolução de compra para comercialização"),
        (5205, "Anulação de valor relativo a aquisição de serviço de comunicação"),
        (5206, "Anulação de valor relativo a aquisição de serviço de transporte"),
        (5207, "Anulação de valor relativo à compra de energia elétrica"),
        (5208, "Devolução de mercadoria recebida em transferência para industrialização ou produção rural"),
        (5209, "Devolução de mercadoria recebida em transferência para comercialização"),
        (5210, "Devolução de compra para utilização na prestação de serviço"),
        (5213, "Devolução de entrada de mercadoria com previsão de posterior ajuste ou fixação de preço, em ato cooperativo"),
        (5214, "Devolução de fixação de preço de produção do estabelecimento produtor, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço, de ato cooperativo, para comercialização"),
        (5215, "Devolução de fixação de preço de produção do estabelecimento produtor, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço, de ato cooperativo, para industrialização"),
        (5250, "VENDAS DE ENERGIA ELÉTRICA"),
        (5251, "Venda de energia elétrica para distribuição ou comercialização"),
        (5252, "Venda de energia elétrica para estabelecimento industrial"),
        (5253, "Venda de energia elétrica para estabelecimento comercial"),
        (5254, "Venda de energia elétrica para estabelecimento prestador de serviço de transporte"),
        (5255, "Venda de energia elétrica para estabelecimento prestador de serviço de comunicação"),
        (5256, "Venda de energia elétrica para estabelecimento de produtor rural"),
        (5257, "Venda de energia elétrica para consumo por demanda contratada"),
        (5258, "Venda de energia elétrica a não contribuinte"),
        (5300, "PRESTAÇÕES DE SERVIÇOS DE COMUNICAÇÃO"),
        (5301, "Prestação de serviço de comunicação para execução de serviço da mesma natureza"),
        (5302, "Prestação de serviço de comunicação a estabelecimento industrial"),
        (5303, "Prestação de serviço de comunicação a estabelecimento comercial"),
        (5304, "Prestação de serviço de comunicação a estabelecimento de prestador de serviço de transporte"),
        (5305, "Prestação de serviço de comunicação a estabelecimento de geradora ou de distribuidora de energia elétrica"),
        (5306, "Prestação de serviço de comunicação a estabelecimento de produtor rural"),
        (5307, "Prestação de serviço de comunicação a não contribuinte"),
        (5350, "PRESTAÇÕES DE SERVIÇOS DE TRANSPORTE"),
        (5351, "Prestação de serviço de transporte para execução de serviço da mesma natureza"),
        (5352, "Prestação de serviço de transporte a estabelecimento industrial"),
        (5353, "Prestação de serviço de transporte a estabelecimento comercial"),
        (5354, "Prestação de serviço de transporte a estabelecimento de prestador de serviço de comunicação"),
        (5355, "Prestação de serviço de transporte a estabelecimento de geradora ou de distribuidora de energia elétrica"),
        (5356, "Prestação de serviço de transporte a estabelecimento de produtor rural"),
        (5357, "Prestação de serviço de transporte a não contribuinte"),
        (5359, "Prestação de serviço de transporte a contribuinte ou a não contribuinte quando a mercadoria transportada está dispensada de emissão de nota fiscal"),
        (5360, "Prestação de serviço de transporte a contribuinte substituto em relação ao serviço de transporte"),
        (5400, "SAÍDAS DE MERCADORIAS SUJEITAS AO REGIME DE SUBSTITUIÇÃO TRIBUTÁRIA"),
        (5401, "Venda de produção do estabelecimento em operação com produto sujeito ao regime de substituição tributária, na condição de contribuinte substituto"),
        (5402, "Venda de produção do estabelecimento de produto sujeito ao regime de substituição tributária, em operação entre contribuintes substitutos do mesmo produto"),
        (5403, "Venda de mercadoria adquirida ou recebida de terceiros em operação com mercadoria sujeita ao regime de substituição tributária, na condição de contribuinte substituto"),
        (5405, "Venda de mercadoria adquirida ou recebida de terceiros em operação com mercadoria sujeita ao regime de substituição tributária, na condição de contribuinte substituído"),
        (5408, "Transferência de produção do estabelecimento em operação com produto sujeito ao regime de substituição tributária"),
        (5409, "Transferência de mercadoria adquirida ou recebida de terceiros em operação com mercadoria sujeita ao regime de substituição tributária"),
        (5410, "Devolução de compra para industrialização ou produção rural em operação com mercadoria sujeita ao regime de substituição tributária"),
        (5411, "Devolução de compra para comercialização em operação com mercadoria sujeita ao regime de substituição tributária"),
        (5412, "Devolução de bem do ativo imobilizado, em operação com mercadoria sujeita ao regime de substituição tributária"),
        (5413, "Devolução de mercadoria destinada ao uso ou consumo, em operação com mercadoria sujeita ao regime de substituição tributária"),
        (5414, "Remessa de produção do estabelecimento para venda fora do estabelecimento em operação com produto sujeito ao regime de substituição tributária"),
        (5415, "Remessa de mercadoria adquirida ou recebida de terceiros para venda fora do estabelecimento, em operação com mercadoria sujeita ao regime de substituição tributária"),
        (5450, "SISTEMAS DE INTEGRAÇÃO"),
        (5451, "Remessa de animal e de insumo para estabelecimento produtor"),
        (5500, "REMESSAS PARA FORMAÇÃO DE LOTE E COM FIM ESPECÍFICO DE EXPORTAÇÃO E EVENTUAIS DEVOLUÇÕES"),
        (5501, "Remessa de produção do estabelecimento, com fim específico de exportação"),
        (5502, "Remessa de mercadoria adquirida ou recebida de terceiros, com fim específico de exportação"),
        (5503, "Devolução de mercadoria recebida com fim específico de exportação"),
        (5504, "Remessa de mercadorias para formação de lote de exportação, de produtos industrializados ou produzidos pelo próprio estabelecimento"),
        (5505, "Remessa de mercadorias, adquiridas ou recebidas de terceiros, para formação de lote de exportação"),
        (5550, "OPERAÇÕES COM BENS DE ATIVO IMOBILIZADO E MATERIAIS PARA USO OU CONSUMO"),
        (5551, "Venda de bem do ativo imobilizado"),
        (5552, "Transferência de bem do ativo imobilizado"),
        (5553, "Devolução de compra de bem para o ativo imobilizado"),
        (5554, "Remessa de bem do ativo imobilizado para uso fora do estabelecimento"),
        (5555, "Devolução de bem do ativo imobilizado de terceiro, recebido para uso no estabelecimento"),
        (5556, "Devolução de compra de material de uso ou consumo"),
        (5557, "Transferência de material de uso ou consumo"),
        (5600, "CRÉDITOS E RESSARCIMENTOS DE ICMS"),
        (5601, "Transferência de crédito de ICMS acumulado"),
        (5602, "Transferência de saldo credor de ICMS para outro estabelecimento da mesma empresa, destinado à compensação de saldo devedor de ICMS"),
        (5603, "Ressarcimento de ICMS retido por substituição tributária"),
        (5605, "Transferência de saldo devedor de ICMS de outro estabelecimento da mesma empresa"),
        (5606, "Utilização de saldo credor de ICMS para extinção por compensação de débitos fiscais"),
        (5650, "SAÍDAS DE COMBUSTÍVEIS, DERIVADOS OU NÃO DE PETRÓLEO E LUBRIFICANTES"),
        (5651, "Venda de combustível ou lubrificante de produção do estabelecimento destinado à industrialização subseqüente"),
        (5652, "Venda de combustível ou lubrificante de produção do estabelecimento destinado à comercialização"),
        (5653, "Venda de combustível ou lubrificante de produção do estabelecimento destinado a consumidor ou usuário final"),
        (5654, "Venda de combustível ou lubrificante adquirido ou recebido de terceiros destinado à industrialização subseqüente"),
        (5655, "Venda de combustível ou lubrificante adquirido ou recebido de terceiros destinado à comercialização"),
        (5656, "Venda de combustível ou lubrificante adquirido ou recebido de terceiros destinado a consumidor ou usuário final"),
        (5657, "Remessa de combustível ou lubrificante adquirido ou recebido de terceiros para venda fora do estabelecimento"),
        (5658, "Transferência de combustível ou lubrificante de produção do estabelecimento"),
        (5659, "Transferência de combustível ou lubrificante adquirido ou recebido de terceiro"),
        (5660, "Devolução de compra de combustível ou lubrificante adquirido para industrialização subseqüente"),
        (5661, "Devolução de compra de combustível ou lubrificante adquirido para comercialização"),
        (5662, "Devolução de compra de combustível ou lubrificante adquirido por consumidor ou usuário final"),
        (5663, "Remessa para armazenagem de combustível ou lubrificante"),
        (5664, "Retorno de combustível ou lubrificante recebido para armazenagem"),
        (5665, "Retorno simbólico de combustível ou lubrificante recebido para armazenagem"),
        (5666, "Remessa por conta e ordem de terceiros de combustível ou lubrificante recebido para armazenagem"),
        (5667, "Venda de combustível ou lubrificante a consumidor ou usuário final estabelecido em outra unidade da Federação"),
        (5900, "OUTRAS SAÍDAS DE MERCADORIAS OU PRESTAÇÕES DE SERVIÇOS"),
        (5901, "Remessa para industrialização por encomenda"),
        (5902, "Retorno de mercadoria utilizada na industrialização por encomenda"),
        (5903, "Retorno de mercadoria recebida para industrialização e não aplicada no referido processo"),
        (5904, "Remessa para venda fora do estabelecimento"),
        (5905, "Remessa para depósito fechado ou armazém geral"),
        (5906, "Retorno de mercadoria depositada em depósito fechado ou armazém geral"),
        (5907, "Retorno simbólico de mercadoria depositada em depósito fechado ou armazém geral"),
        (5908, "Remessa de bem por conta de contrato de comodato"),
        (5909, "Retorno de bem recebido por conta de contrato de comodato"),
        (5910, "Remessa em bonificação, doação ou brinde"),
        (5911, "Remessa de amostra grátis"),
        (5912, "Remessa de mercadoria ou bem para demonstração, mostruário ou treinamento"),
        (5913, "Retorno de mercadoria ou bem recebido para demonstração ou mostruário"),
        (5914, "Remessa de mercadoria ou bem para exposição ou feira"),
        (5915, "Remessa de mercadoria ou bem para conserto ou reparo"),
        (5916, "Retorno de mercadoria ou bem recebido para conserto ou reparo"),
        (5917, "Remessa de mercadoria em consignação mercantil ou industrial"),
        (5918, "Devolução de mercadoria recebida em consignação mercantil ou industrial"),
        (5919, "Devolução simbólica de mercadoria vendida ou utilizada em processo industrial, recebida anteriormente em consignação mercantil ou industrial"),
        (5920, "Remessa de vasilhame ou sacaria"),
        (5921, "Devolução de vasilhame ou sacaria"),
        (5922, "Lançamento efetuado a título de simples faturamento decorrente de venda para entrega futura"),
        (5923, "Remessa de mercadoria por conta e ordem de terceiros, em venda à ordem ou em operações com armazém geral ou depósito fechado"),
        (5924, "Remessa para industrialização por conta e ordem do adquirente da mercadoria, quando esta não transitar pelo estabelecimento do adquirente"),
        (5925, "Retorno de mercadoria recebida para industrialização por conta e ordem do adquirente da mercadoria, quando aquela não transitar pelo estabelecimento do adquirente"),
        (5926, "Lançamento efetuado a título de reclassificação de mercadoria decorrente de formação de kit ou de sua desagregação"),
        (5927, "Lançamento efetuado a título de baixa de estoque decorrente de perda, roubo ou deterioração"),
        (5928, "Lançamento efetuado a título de baixa de estoque decorrente do encerramento da atividade da empresa"),
        (5929, "Lançamento efetuado em decorrência de emissão de documento fiscal relativo a operação ou prestação também registradaem equipamento Emissorde Cupom Fiscal - ECF"),
        (5931, "Lançamento efetuado em decorrência da responsabilidade de retenção do imposto por substituição tributária, atribuída ao remetente ou alienante da mercadoria, pelo serviço de transporte realizado por transportador autônomo ou por transportador não inscrito na unidade da Federação onde iniciado o serviço"),
        (5932, "Prestação de serviço de transporte iniciada em unidade da Federação diversa daquela onde inscrito o prestador"),
        (5933, "Prestação de serviço tributado pelo ISSQN"),
        (5934, "Remessa simbólica de mercadoria depositada em armazém geral ou depósito fechado"),
        (5949, "Outra saída de mercadoria ou prestação de serviço não especificado"),
        (6000, "SAÍDAS OU PRESTAÇÕES DE SERVIÇOS PARA OUTROS ESTADOS"),
        (6100, "VENDAS DE PRODUÇÃO PRÓPRIA OU DE TERCEIROS"),
        (6101, "Venda de produção do estabelecimento"),
        (6102, "Venda de mercadoria adquirida ou recebida de terceiros"),
        (6103, "Venda de produção do estabelecimento, efetuada fora do estabelecimento"),
        (6104, "Venda de mercadoria adquirida ou recebida de terceiros, efetuada fora do estabelecimento"),
        (6105, "Venda de produção do estabelecimento que não deva por ele transitar"),
        (6106, "Venda de mercadoria adquirida ou recebida de terceiros, que não deva por ele transitar"),
        (6107, "Venda de produção do estabelecimento, destinada a não contribuinte"),
        (6108, "Venda de mercadoria adquirida ou recebida de terceiros, destinada a não contribuinte"),
        (6109, "Venda de produção do estabelecimento, destinada à Zona Franca de Manaus ou Áreas de Livre Comércio"),
        (6110, "Venda de mercadoria adquirida ou recebida de terceiros, destinada à Zona Franca de Manaus ou Áreas de Livre Comércio"),
        (6111, "Venda de produção do estabelecimento remetida anteriormente em consignação industrial"),
        (6112, "Venda de mercadoria adquirida ou recebida de Terceiros remetida anteriormente em consignação industrial"),
        (6113, "Venda de produção do estabelecimento remetida anteriormente em consignação mercantil"),
        (6114, "Venda de mercadoria adquirida ou recebida de terceiros remetida anteriormente em consignação mercantil"),
        (6115, "Venda de mercadoria adquirida ou recebida de terceiros, recebida anteriormente em consignação mercantil"),
        (6116, "Venda de produção do estabelecimento originada de encomenda para entrega futura"),
        (6117, "Venda de mercadoria adquirida ou recebida de terceiros, originada de encomenda para entrega futura"),
        (6118, "Venda de produção do estabelecimento entregue ao destinatário por conta e ordem do adquirente originário, em venda à ordem"),
        (6119, "Venda de mercadoria adquirida ou recebida de terceiros entregue ao destinatário por conta e ordem do adquirente originário, em venda à ordem"),
        (6120, "Venda de mercadoria adquirida ou recebida de terceiros entregue ao destinatário pelo vendedor remetente, em venda à ordem"),
        (6122, "Venda de produção do estabelecimento remetida para industrialização, por conta e ordem do adquirente, sem transitar pelo estabelecimento do adquirente"),
        (6123, "Venda de mercadoria adquirida ou recebida de terceiros remetida para industrialização, por conta e ordem do adquirente, sem transitar pelo estabelecimento do adquirente"),
        (6124, "Industrialização efetuada para outra empresa"),
        (6125, "Industrialização efetuada para outra empresa quando a mercadoria recebida para utilização no processo de industrialização não transitar pelo estabelecimento adquirente da mercadoria"),
        (6129, "Venda de insumo importado e de mercadoria industrializada sob o amparo do Regime Aduaneiro Especial de Entreposto Industrial sob Controle Informatizado do Sistema Público de Escrituração Digital (Recof-Sped)"),
        (6131, "Remessa de produção de estabelecimento, com previsão de posterior ajuste ou fixação de preço de ato cooperativo"),
        (6132, "Fixação de preço de produção do estabelecimento, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço ou fixação de preço de ato cooperativo"),
        (6150, "TRANSFERÊNCIAS DE PRODUÇÃO PRÓPRIA OU DE TERCEIROS"),
        (6151, "Transferência de produção do estabelecimento"),
        (6152, "Transferência de mercadoria adquirida ou recebida de terceiros"),
        (6153, "Transferência de energia elétrica"),
        (6155, "Transferência de produção do estabelecimento, que não deva por ele transitar"),
        (6156, "Transferência de mercadoria adquirida ou recebida de terceiros, que não deva por ele transitar"),
        (6159, "Fornecimento de produção do estabelecimento de ato cooperativo"),
        (6160, "Fornecimento de mercadoria adquirida ou recebida de terceiros de ato cooperativo"),
        (6200, "DEVOLUÇÕES DE COMPRAS PARA INDUSTRIALIZAÇÃO, PRODUÇÃO RURAL, COMERCIALIZAÇÃO OU ANULAÇÕES DE VALORES"),
        (6201, "Devolução de compra para industrialização ou produção rural"),
        (6202, "Devolução de compra para comercialização"),
        (6205, "Anulação de valor relativo a aquisição de serviço de comunicação"),
        (6206, "Anulação de valor relativo a aquisição de serviço de transporte"),
        (6207, "Anulação de valor relativo à compra de energia elétrica"),
        (6208, "Devolução de mercadoria recebida em transferência para industrialização ou produção rural"),
        (6209, "Devolução de mercadoria recebida em transferência para comercialização"),
        (6210, "Devolução de compra para utilização na prestação de serviço"),
        (6213, "Devolução de entrada de mercadoria com previsão de posterior ajuste ou fixação de preço, em ato cooperativo"),
        (6214, "Devolução de fixação de preço de produção do estabelecimento produtor, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço, de ato cooperativo, para comercialização"),
        (6215, "Devolução de fixação de preço de produção do estabelecimento produtor, inclusive quando remetidas anteriormente com previsão de posterior ajuste ou fixação de preço, de ato cooperativo para industrialização"),
        (6250, "VENDAS DE ENERGIA ELÉTRICA"),
        (6251, "Venda de energia elétrica para distribuição ou comercialização"),
        (6252, "Venda de energia elétrica para estabelecimento industrial"),
        (6253, "Venda de energia elétrica para estabelecimento comercial"),
        (6254, "Venda de energia elétrica para estabelecimento prestador de serviço de transporte"),
        (6255, "Venda de energia elétrica para estabelecimento prestador de serviço de comunicação"),
        (6256, "Venda de energia elétrica para estabelecimento de produtor rural"),
        (6257, "Venda de energia elétrica para consumo por demanda contratada"),
        (6258, "Venda de energia elétrica a não contribuinte"),
        (6300, "PRESTAÇÕES DE SERVIÇOS DE COMUNICAÇÃO"),
        (6301, "Prestação de serviço de comunicação para execução de serviço da mesma natureza"),
        (6302, "Prestação de serviço de comunicação a estabelecimento industrial"),
        (6303, "Prestação de serviço de comunicação a estabelecimento comercial"),
        (6304, "Prestação de serviço de comunicação a estabelecimento de prestador de serviço de transporte"),
        (6305, "Prestação de serviço de comunicação a estabelecimento de geradora ou de distribuidora de energia elétrica"),
        (6306, "Prestação de serviço de comunicação a estabelecimento de produtor rural"),
        (6307, "Prestação de serviço de comunicação a não contribuinte"),
        (6350, "PRESTAÇÕES DE SERVIÇOS DE TRANSPORTE"),
        (6351, "Prestação de serviço de transporte para execução de serviço da mesma natureza"),
        (6352, "Prestação de serviço de transporte a estabelecimento industrial"),
        (6353, "Prestação de serviço de transporte a estabelecimento comercial"),
        (6354, "Prestação de serviço de transporte a estabelecimento de prestador de serviço de comunicação"),
        (6355, "Prestação de serviço de transporte a estabelecimento de geradora ou de distribuidora de energia elétrica"),
        (6356, "Prestação de serviço de transporte a estabelecimento de produtor rural"),
        (6357, "Prestação de serviço de transporte a não contribuinte"),
        (6359, "Prestação de serviço de transporte a contribuinte ou a não contribuinte quando a mercadoria transportada está dispensada de emissão de nota fiscal"),
        (6360, "Prestação de serviço de transporte a contribuinte substituto em relação ao serviço de transporte"),
        (6400, "SAÍDAS DE MERCADORIAS SUJEITAS AO REGIME DE SUBSTITUIÇÃO TRIBUTÁRIA"),
        (6401, "Venda de produção do estabelecimento em operação com produto sujeito ao regime de substituição tributária, na condição de contribuinte substituto"),
        (6402, "Venda de produção do estabelecimento de produto sujeito ao regime de substituição tributária, em operação entre contribuintes substitutos do mesmo produto"),
        (6403, "Venda de mercadoria adquirida ou recebida de terceiros em operação com mercadoria sujeita ao regime de substituição tributária, na condição de contribuinte substituto"),
        (6404, "Venda de mercadoria sujeita ao regime de substituição tributária, cujo imposto já tenha sido retido anteriormente"),
        (6408, "Transferência de produção do estabelecimento em operação com produto sujeito ao regime de substituição tributária"),
        (6409, "Transferência de mercadoria adquirida ou recebida de terceiros em operação com mercadoria sujeita ao regime de substituição tributária"),
        (6410, "Devolução de compra para industrialização ou produção rural em operação com mercadoria sujeita ao regime de substituição tributária"),
        (6411, "Devolução de compra para comercialização em operação com mercadoria sujeita ao regime de substituição tributária"),
        (6412, "Devolução de bem do ativo imobilizado, em operação com mercadoria sujeita ao regime de substituição tributária"),
        (6413, "Devolução de mercadoria destinada ao uso ou consumo, em operação com mercadoria sujeita ao regime de substituição tributária"),
        (6414, "Remessa de produção do estabelecimento para venda fora do estabelecimento em operação com produto sujeito ao regime de substituição tributária"),
        (6415, "Remessa de mercadoria adquirida ou recebida de terceiros para venda fora do estabelecimento, em operação com mercadoria sujeita ao regime de substituição tributária"),
        (6500, "REMESSAS PARA FORMAÇÃO DE LOTE E COM FIM ESPECÍFICO DE EXPORTAÇÃO E EVENTUAIS DEVOLUÇÕES"),
        (6501, "Remessa de produção do estabelecimento, com fim específico de exportação"),
        (6502, "Remessa de mercadoria adquirida ou recebida de terceiros, com fim específico de exportação"),
        (6503, "Devolução de mercadoria recebida com fim específico de exportação"),
        (6504, "Remessa de mercadorias para formação de lote de exportação, de produtos industrializados ou produzidos pelo próprio estabelecimento"),
        (6505, "Remessa de mercadorias, adquiridas ou recebidas de terceiros, para formação de lote de exportação"),
        (6550, "OPERAÇÕES COM BENS DE ATIVO IMOBILIZADO E MATERIAIS PARA USO OU CONSUMO"),
        (6551, "Venda de bem do ativo imobilizado"),
        (6552, "Transferência de bem do ativo imobilizado"),
        (6553, "Devolução de compra de bem para o ativo imobilizado"),
        (6554, "Remessa de bem do ativo imobilizado para uso fora do estabelecimento"),
        (6555, "Devolução de bem do ativo imobilizado de terceiro, recebido para uso no estabelecimento"),
        (6556, "Devolução de compra de material de uso ou consumo"),
        (6557, "Transferência de material de uso ou consumo"),
        (6600, "CRÉDITOS E RESSARCIMENTOS DE ICMS"),
        (6603, "Ressarcimento de ICMS retido por substituição tributária"),
        (6650, "SAÍDAS DE COMBUSTÍVEIS, DERIVADOS OU NÃO DE PETRÓLEO E LUBRIFICANTES"),
        (6651, "Venda de combustível ou lubrificante de produção do estabelecimento destinado à industrialização subseqüente"),
        (6652, "Venda de combustível ou lubrificante de produção do estabelecimento destinado à comercialização"),
        (6653, "Venda de combustível ou lubrificante de produção do estabelecimento destinado a consumidor ou usuário final"),
        (6654, "Venda de combustível ou lubrificante adquirido ou recebido de terceiros destinado à industrialização subseqüente"),
        (6655, "Venda de combustível ou lubrificante adquirido ou recebido de terceiros destinado à comercialização"),
        (6656, "Venda de combustível ou lubrificante adquirido ou recebido de terceiros destinado a consumidor ou usuário final"),
        (6657, "Remessa de combustível ou lubrificante adquirido ou recebido de terceiros para venda fora do estabelecimento"),
        (6658, "Transferência de combustível ou lubrificante de produção do estabelecimento"),
        (6659, "Transferência de combustível ou lubrificante adquirido ou recebido de terceiro"),
        (6660, "Devolução de compra de combustível ou lubrificante adquirido para industrialização subseqüente"),
        (6661, "Devolução de compra de combustível ou lubrificante adquirido para comercialização"),
        (6662, "Devolução de compra de combustível ou lubrificante adquirido por consumidor ou usuário final"),
        (6663, "Remessa para armazenagem de combustível ou lubrificante"),
        (6664, "Retorno de combustível ou lubrificante recebido para armazenagem"),
        (6665, "Retorno simbólico de combustível ou lubrificante recebido para armazenagem"),
        (6666, "Remessa por conta e ordem de terceiros de combustível ou lubrificante recebido para armazenagem"),
        (6667, "Venda de combustível ou lubrificante a consumidor ou usuário final estabelecido em outra unidade da Federação diferente da que ocorrer o consumo"),
        (6900, "OUTRAS SAÍDAS DE MERCADORIAS OU PRESTAÇÕES DE SERVIÇOS"),
        (6901, "Remessa para industrialização por encomenda"),
        (6902, "Retorno de mercadoria utilizada na industrialização por encomenda"),
        (6903, "Retorno de mercadoria recebida para industrialização e não aplicada no referido processo"),
        (6904, "Remessa para venda fora do estabelecimento"),
        (6905, "Remessa para depósito fechado ou armazém geral"),
        (6906, "Retorno de mercadoria depositada em depósito fechado ou armazém geral"),
        (6907, "Retorno simbólico de mercadoria depositada em depósito fechado ou armazém geral"),
        (6908, "Remessa de bem por conta de contrato de comodato"),
        (6909, "Retorno de bem recebido por conta de contrato de comodato"),
        (6910, "Remessa em bonificação, doação ou brinde"),
        (6911, "Remessa de amostra grátis"),
        (6912, "Remessa de mercadoria ou bem para demonstração, mostruário ou treinamento"),
        (6913, "Retorno de mercadoria ou bem recebido para demonstração ou mostruário"),
        (6914, "Remessa de mercadoria ou bem para exposição ou feira"),
        (6915, "Remessa de mercadoria ou bem para conserto ou reparo"),
        (6916, "Retorno de mercadoria ou bem recebido para conserto ou reparo"),
        (6917, "Remessa de mercadoria em consignação mercantil ou industrial"),
        (6918, "Devolução de mercadoria recebida em consignação mercantil ou industrial"),
        (6919, "Devolução simbólica de mercadoria vendida ou utilizada em processo industrial, recebida anteriormente em consignação mercantil ou industrial"),
        (6920, "Remessa de vasilhame ou sacaria"),
        (6921, "Devolução de vasilhame ou sacaria"),
        (6922, "Lançamento efetuado a título de simples faturamento decorrente de venda para entrega futura"),
        (6923, "Remessa de mercadoria por conta e ordem de terceiros, em venda à ordem ou em operações com armazém geral ou depósito fechado"),
        (6924, "Remessa para industrialização por conta e ordem do adquirente da mercadoria, quando esta não transitar pelo estabelecimento do adquirente"),
        (6925, "Retorno de mercadoria recebida para industrialização por conta e ordem do adquirente da mercadoria, quando aquela não transitar pelo estabelecimento do adquirente"),
        (6929, "Lançamento efetuado em decorrência de emissão de documento fiscal relativo a operação ou prestação também registradaem equipamento Emissorde Cupom Fiscal - ECF"),
        (6931, "Lançamento efetuado em decorrência da responsabilidade de retenção do imposto por substituição tributária, atribuída ao remetente ou alienante da mercadoria, pelo serviço de transporte realizado por transportador autônomo ou por transportador não inscrito na unidade da Federação onde iniciado o serviço"),
        (6932, "Prestação de serviço de transporte iniciada em unidade da Federação diversa daquela onde inscrito o prestador"),
        (6933, "Prestação de serviço tributado pelo ISSQN"),
        (6934, "Remessa simbólica de mercadoria depositada em armazém geral ou depósito fechado"),
        (6949, "Outra saída de mercadoria ou prestação de serviço não especificado"),
        (7000, "SAÍDAS OU PRESTAÇÕES DE SERVIÇOS PARA O EXTERIOR"),
        (7100, "VENDAS DE PRODUÇÃO PRÓPRIA OU DE TERCEIROS"),
        (7101, "Venda de produção do estabelecimento"),
        (7102, "Venda de mercadoria adquirida ou recebida de terceiros"),
        (7105, "Venda de produção do estabelecimento, que não deva por ele transitar"),
        (7106, "Venda de mercadoria adquirida ou recebida de terceiros, que não deva por ele transitar"),
        (7127, "Venda de produção do estabelecimento sob o regime de 'drawback'"),
        (7129, "Venda de produção do estabelecimento ao mercado externo de mercadoria industrializada sob o amparo do Regime Aduaneiro Especial de Entreposto Industrial sob Controle Informatizado do Sistema Público de Escrituração Digital (Recof-Sped)"),
        (7200, "DEVOLUÇÕES DE COMPRAS PARA INDUSTRIALIZAÇÃO, PRODUÇÃO RURAL, COMERCIALIZAÇÃO OU ANULAÇÕES DE VALORES"),
        (7201, "Devolução de compra para industrialização ou produção rural"),
        (7202, "Devolução de compra para comercialização"),
        (7205, "Anulação de valor relativo à aquisição de serviço de comunicação"),
        (7206, "Anulação de valor relativo a aquisição de serviço de transporte"),
        (7207, "Anulação de valor relativo à compra de energia elétrica"),
        (7210, "Devolução de compra para utilização na prestação de serviço"),
        (7211, "Devolução de compras para industrialização sob o regime de drawback"),
        (7212, "Devolução de compras para industrialização sob o regime de Regime Aduaneiro Especial de Entreposto Industrial sob Controle Informatizado do Sistema Público de Escrituração Digital (Recof-Sped)"),
        (7250, "VENDAS DE ENERGIA ELÉTRICA"),
        (7251, "Venda de energia elétrica para o exterior"),
        (7300, "PRESTAÇÕES DE SERVIÇOS DE COMUNICAÇÃO"),
        (7301, "Prestação de serviço de comunicação para execução de serviço da mesma natureza"),
        (7350, "PRESTAÇÕES DE SERVIÇO DE TRANSPORTE"),
        (7358, "Prestação de serviço de transporte"),
        (7500, "EXPORTAÇÃO DE MERCADORIAS RECEBIDAS COM FIM ESPECÍFICO DE EXPORTAÇÃO"),
        (7501, "Exportação de mercadorias recebidas com fim específico de exportação"),
        (7504, "Exportação de mercadoria que foi objeto de formação de lote de exportação"),
        (7550, "OPERAÇÕES COM BENS DE ATIVO IMOBILIZADO E MATERIAIS PARA USO OU CONSUMO"),
        (7551, "Venda de bem do ativo imobilizado"),
        (7553, "Devolução de compra de bem para o ativo imobilizado"),
        (7556, "Devolução de compra de material de uso ou consumo"),
        (7650, "SAÍDAS DE COMBUSTÍVEIS, DERIVADOS OU NÃO DE PETRÓLEO E LUBRIFICANTES"),
        (7651, "Venda de combustível ou lubrificante de produção do estabelecimento"),
        (7654, "Venda de combustível ou lubrificante adquirido ou recebido de terceiros"),
        (7667, "Venda de combustível ou lubrificante a consumidor ou usuário final"),
        (7900, "OUTRAS SAÍDAS DE MERCADORIAS OU PRESTAÇÕES DE SERVIÇOS"),
        (7930, "Lançamento efetuado a título de devolução de bem cuja entrada tenha ocorrido sob amparo de regime especial aduaneiro de admissão temporária"),
        (7949, "Outra saída de mercadoria ou prestação de serviço não especificado"),
    ];

    let cfop: HashMap<Option<u16>, &'static str> = cfop_descricao
        .into_iter()
        .map(|(cfop, descricao)| (Some(cfop), descricao))
        .collect();

    cfop
});

pub fn obter_descricao_do_cfop(cfop: &Option<u16>) -> String {

    let cfop_descricao = match CFOP_DESCRICAO_RESUMIDA.get(cfop) {
        Some(&descricao) => format!("{:04} - {}", cfop.unwrap(), descricao),
        None => "".to_string(),
    };

    cfop_descricao
}

#[cfg(test)]
mod tests {
    // cargo test -- --help
    // cargo test -- --nocapture
    // cargo test -- --show-output
    use super::*;

    #[test]
    fn testar_month_to_str() {
    // cargo test -- --show-output testar_month_to_str
        let mes_deze = month_to_str(&Some(12));
        let mes_none = month_to_str(&Some(15));
        println!("mes Some(12): {}", mes_deze);
        println!("mes Some(15): {}", mes_none);
        assert_eq!(mes_deze, "dezembro");
        assert_eq!(mes_none, "");
    }
}
