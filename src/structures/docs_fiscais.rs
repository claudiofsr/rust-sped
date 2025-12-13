use chrono::NaiveDate;
use claudiofsr_lib::{
    CFOP_VENDA_DE_IMOBILIZADO, CODIGO_DA_NATUREZA_BC, CST_ALL, CST_CREDITO, CST_RECEITA_BRUTA,
    OUTRAS_RECEITAS_REGEX, StrExtension, match_cast,
};
use csv::StringRecord;
use rust_xlsxwriter::serialize_option_datetime_to_excel;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::serde_introspect;
use struct_iterable::Iterable;

use crate::{
    IndicadorOrigem, InfoExtension, MesesDoAno, TipoDeCredito, TipoDeRateio, TipoDoItem,
    TipoOperacao, serialize_natureza,
};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone, Iterable)]
pub struct DocsFiscais {
    #[serde(rename = "Linhas")]
    pub linhas: usize,

    #[serde(rename = "Arquivo da EFD Contribuições")] // não pode conter vírgulas
    pub arquivo_efd: String,

    #[serde(
        rename = "Nº da Linha da EFD",
        deserialize_with = "csv::invalid_option"
    )]
    pub num_linha_efd: Option<usize>,

    #[serde(rename = "CNPJ dos Estabelecimentos do Contribuinte")]
    pub estabelecimento_cnpj: String,

    #[serde(rename = "Nome do Contribuinte")]
    pub estabelecimento_nome: String,

    #[serde(
        rename = "Período de Apuração",
        //deserialize_with = "csv::invalid_option",
        serialize_with = "serialize_option_datetime_to_excel"
    )]
    pub periodo_de_apuracao: Option<NaiveDate>,

    #[serde(
        rename = "Ano do Período de Apuração",
        deserialize_with = "csv::invalid_option"
    )]
    pub ano: Option<i32>,

    #[serde(
        rename = "Trimestre do Período de Apuração",
        deserialize_with = "csv::invalid_option"
    )]
    pub trimestre: Option<u32>,

    #[serde(rename = "Mês do Período de Apuração")]
    pub mes: Option<MesesDoAno>,

    #[serde(rename = "Tipo de Operação")]
    pub tipo_de_operacao: Option<TipoOperacao>,

    #[serde(rename = "Indicador de Origem")]
    pub indicador_de_origem: Option<IndicadorOrigem>,

    #[serde(
        rename = "Código do Tipo de Crédito",
        deserialize_with = "csv::invalid_option"
    )]
    pub cod_credito: Option<u16>,

    #[serde(rename = "Tipo de Crédito")]
    pub tipo_de_credito: Option<TipoDeCredito>,

    #[serde(rename = "Registro")]
    pub registro: String,

    #[serde(
        rename = "Código de Situação Tributária (CST)",
        deserialize_with = "csv::invalid_option"
    )]
    pub cst: Option<u16>,

    #[serde(
        rename = "Código Fiscal de Operações e Prestações (CFOP)",
        deserialize_with = "csv::invalid_option"
    )]
    pub cfop: Option<u16>,

    #[serde(
        rename = "Natureza da Base de Cálculo dos Créditos",
        //deserialize_with = "csv::invalid_option",
        serialize_with = "serialize_natureza",
    )]
    pub natureza_bc: Option<u16>,

    #[serde(rename = "CNPJ do Participante")]
    pub participante_cnpj: String,

    #[serde(rename = "CPF do Participante")]
    pub participante_cpf: String,

    #[serde(rename = "Nome do Participante")]
    pub participante_nome: String,

    #[serde(
        rename = "Nº do Documento Fiscal",
        deserialize_with = "csv::invalid_option"
    )]
    pub num_doc: Option<usize>,

    #[serde(rename = "Chave do Documento")]
    pub chave_doc: String,

    #[serde(rename = "Modelo do Documento Fiscal")]
    pub modelo_doc_fiscal: String,

    #[serde(
        rename = "Nº do Item do Documento Fiscal",
        deserialize_with = "csv::invalid_option"
    )]
    pub num_item: Option<u32>,

    #[serde(rename = "Tipo do Item")]
    pub tipo_item: Option<TipoDoItem>,

    #[serde(rename = "Descrição do Item")]
    pub descr_item: String,

    #[serde(rename = "Código NCM")]
    pub cod_ncm: String,

    #[serde(rename = "Natureza da Operação/Prestação")]
    pub nat_operacao: String,

    #[serde(rename = "Informação Complementar do Documento Fiscal")]
    pub complementar: String,

    #[serde(rename = "Escrituração Contábil: Nome da Conta")]
    pub nome_da_conta: String,

    #[serde(
        rename = "Data da Emissão do Documento Fiscal",
        //deserialize_with = "csv::invalid_option",
        serialize_with = "serialize_option_datetime_to_excel",
    )]
    pub data_emissao: Option<NaiveDate>,

    #[serde(
        rename = "Data da Entrada / Aquisição / Execução ou da Saída / Prestação / Conclusão",
        //deserialize_with = "csv::invalid_option",
        serialize_with = "serialize_option_datetime_to_excel",
    )]
    pub data_entrada: Option<NaiveDate>,

    #[serde(
        rename = "Valor Total do Item",
        deserialize_with = "csv::invalid_option"
    )]
    pub valor_item: Option<f64>,

    #[serde(
        rename = "Valor da Base de Cálculo das Contribuições",
        deserialize_with = "csv::invalid_option"
    )]
    pub valor_bc: Option<f64>,
    #[serde(
        rename = "Alíquota de PIS/PASEP (em percentual)",
        deserialize_with = "csv::invalid_option"
    )]
    pub aliq_pis: Option<f64>,

    #[serde(
        rename = "Alíquota de COFINS (em percentual)",
        deserialize_with = "csv::invalid_option"
    )]
    pub aliq_cofins: Option<f64>,

    #[serde(
        rename = "Valor de PIS/PASEP",
        deserialize_with = "csv::invalid_option"
    )]
    pub valor_pis: Option<f64>,

    #[serde(rename = "Valor de COFINS", deserialize_with = "csv::invalid_option")]
    pub valor_cofins: Option<f64>,

    #[serde(rename = "Valor de ISS", deserialize_with = "csv::invalid_option")]
    pub valor_iss: Option<f64>,

    #[serde(
        rename = "Valor da Base de Cálculo de ICMS",
        deserialize_with = "csv::invalid_option"
    )]
    pub valor_bc_icms: Option<f64>,

    #[serde(
        rename = "Alíquota de ICMS (em percentual)",
        deserialize_with = "csv::invalid_option"
    )]
    pub aliq_icms: Option<f64>,

    #[serde(rename = "Valor de ICMS", deserialize_with = "csv::invalid_option")]
    pub valor_icms: Option<f64>,
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl InfoExtension for DocsFiscais {}

trait PrintOptionFloat {
    fn to_some_string(&self) -> Option<String>;
}

impl PrintOptionFloat for Option<f64> {
    fn to_some_string(&self) -> Option<String> {
        self.map(|float_64| {
            if float_64 == float_64.trunc() {
                format!("{float_64:.1}")
            } else {
                float_64.to_string()
            }
        })
    }
}

impl DocsFiscais {
    // Used by fn write_csv()
    pub fn get_headers() -> StringRecord {
        // use serde_aux::prelude::serde_introspect;
        let colunas_vec = serde_introspect::<DocsFiscais>();
        StringRecord::from(colunas_vec)
    }

    pub fn get_values(&self) -> Vec<String> {
        /*
        vec![
            self.linhas.to_string(),
            self.arquivo_efd.clone(),
            self.num_linha_efd.to_string(),
            self.estabelecimento_cnpj.clone(),
            self.estabelecimento_nome.clone(),
            self.periodo_de_apuracao.to_string(),
            self.ano.to_string(),
            self.trimestre.to_string(),
            self.mes.to_string(),
            self.tipo_de_operacao.to_string(),
            self.indicador_de_origem.to_string(),
            self.cod_credito.to_string(),
            self.tipo_de_credito.to_string(),
            self.registro.clone(),
            self.cst.to_string(),
            self.cfop.to_string(),
            self.natureza_bc.to_string(),
            self.participante_cnpj.clone(),
            self.participante_cpf.clone(),
            self.participante_nome.clone(),
            self.num_doc.to_string(),
            self.chave_doc.clone(),
            self.modelo_doc_fiscal.clone(),
            self.num_item.to_string(),
            self.tipo_item.clone(),
            self.descr_item.clone(),
            self.cod_ncm.clone(),
            self.nat_operacao.clone(),
            self.complementar.clone(),
            self.nome_da_conta.clone(),
            self.data_emissao.to_string(),
            self.data_entrada.to_string(),
            self.valor_item.to_string(),
            self.valor_bc.to_string(),
            self.aliq_pis.to_string(),
            self.aliq_cofins.to_string(),
            self.valor_pis.to_string(),
            self.valor_cofins.to_string(),
            self.valor_iss.to_string(),
            self.valor_bc_icms.to_string(),
            self.aliq_icms.to_string(),
            self.valor_icms.to_string(),
        ]
        */

        self.iter()
            .map(|(_field, value)| {
                let opt_string: Option<String> = match_cast!( value {
                    val as Option<u16> => {
                        val.as_ref().map(|s| s.to_string())
                    },
                    val as Option<u32> => {
                        val.as_ref().map(|s| s.to_string())
                    },
                    val as Option<i32> => {
                        val.as_ref().map(|s| s.to_string())
                    },
                    val as Option<u16> => {
                        val.as_ref().map(|s| s.to_string())
                    },
                    val as Option<usize> => {
                        val.as_ref().map(|s| s.to_string())
                    },
                    val as Option<f64> => {
                        val.to_some_string()
                    },
                    val as Option<NaiveDate> => {
                        val.as_ref().map(|s| s.to_string())
                    },
                    val as Option<IndicadorOrigem> => {
                        // IndicadorOrigem::MercadoInterno as u8, o resultado será 0.
                        // IndicadorOrigem::Importacao     as u8, o resultado será 1.
                        val.as_ref().map(|&indicador| (indicador as u8).to_string())
                    },
                    val as Option<TipoOperacao> => {
                        val.as_ref().map(|&tipo| (tipo as u8).to_string())
                    },
                    val as Option<TipoDeCredito> => {
                        val.as_ref().map(|&tipo| (tipo as u8).to_string())
                    },
                    val as Option<TipoDoItem> => {
                        val.as_ref().map(|&tipo| tipo.descricao_com_codigo())
                    },
                    val as Option<MesesDoAno> => {
                        val.as_ref().map(|&mes| (mes as u8).to_string())
                    },
                    val as Option<TipoDeRateio> => {
                        val.as_ref().map(|&tipo| (tipo as u8).to_string())
                    },
                    val as usize => {
                        Some(val.to_string())
                    },
                    val as String => {
                        Some(val.to_string())
                    },
                });

                match opt_string {
                    Some(string) => string,
                    None => "".to_string(),
                }
            })
            .collect()
    }

    /// Get CNPJ Base
    pub fn get_cnpj_base(&self) -> String {
        self.estabelecimento_cnpj[0..10].to_string()
    }

    #[allow(dead_code)]
    pub fn get_number_of_fields() -> usize {
        // use serde_aux::prelude::serde_introspect;
        let colunas_vec = serde_introspect::<DocsFiscais>();
        let number_of_fields = colunas_vec.len();
        println!("number_of_fields: {number_of_fields}");
        number_of_fields
    }

    /// CST Válido
    ///
    /// Valores entre 1 e 99 (inclusive).
    pub fn cst_valido(&self) -> bool {
        self.cst
            .is_some_and(|value| CST_ALL.binary_search(&value).is_ok())
    }

    /// CST de crédito das Contribuições
    ///
    /// Valores entre 50 a 56 ou 60 a 66
    pub fn cst_de_credito(&self) -> bool {
        //(self.cst >= Some(50) && self.cst <= Some(56)) ||
        //(self.cst >= Some(60) && self.cst <= Some(66))
        self.cst
            .is_some_and(|value| CST_CREDITO.binary_search(&value).is_ok())
    }

    /// CST de Receita Bruta
    ///
    /// Valores entre 1 a 9
    pub fn cst_de_receita_bruta(&self) -> bool {
        //self.cst >= Some(1) && self.cst <= Some(9)
        self.cst
            .is_some_and(|value| CST_RECEITA_BRUTA.binary_search(&value).is_ok())
    }

    /// Natureza da Base de Cálculo
    ///
    /// Valores entre 1 a 18
    pub fn natureza_da_base_de_calculo(&self) -> bool {
        //self.natureza_bc >= Some(1) && self.natureza_bc <= Some(18)
        self.natureza_bc
            .is_some_and(|value| CODIGO_DA_NATUREZA_BC.binary_search(&value).is_ok())
    }

    /// Alíquota de Receita Financeira
    pub fn aliquota_de_receita_financeira(&self) -> bool {
        self.aliq_pis == Some(0.65) && self.aliq_cofins == Some(4.00)
    }

    /// Alíquota de Receita Cumulativa
    pub fn aliquota_de_receita_cumulativa(&self) -> bool {
        self.aliq_pis == Some(0.65) && self.aliq_cofins == Some(3.00)
    }

    /// CFOPs de venda de ativo imobilizado: 5551, 6551 e 7551.
    /// 1. CFOP 5551: Venda de ativo imobilizado para o estado;
    /// 2. CFOP 6551: Venda de ativo imobilizado para outros estados;
    /// 3. CFOP 7551: Venda de ativo imobilizado para o exterior.
    pub fn cfop_de_venda_de_imobilizado(&self) -> bool {
        //self.cfop == Some(5551) ||
        //self.cfop == Some(6551) ||
        //self.cfop == Some(7551)
        // matches!(self.cfop, Some(5551|6551|7551))
        self.cfop
            .is_some_and(|value| CFOP_VENDA_DE_IMOBILIZADO.binary_search(&value).is_ok())
    }

    /// Insumos com direito ao desconto de crédito das Contribuições
    pub fn entrada_de_credito(&self) -> bool {
        self.cst_de_credito()
            && self.natureza_da_base_de_calculo()
            && self
                .tipo_de_operacao
                .is_some_and(|t| t == TipoOperacao::Entrada)
            && self.tipo_de_credito.is_some()
    }

    /// Operações de Entrada ou de Saída
    ///
    /// 1: Entrada,
    ///
    /// 2: Saída
    pub fn operacoes_de_entrada_ou_saida(&self) -> bool {
        self.tipo_de_operacao
            .is_some_and(|t| t.is_entrada_ou_saida())
    }

    /// Receitas Não Operacionais (outras receitas):
    /// são aquelas decorrentes de transações não incluídas nas atividades
    /// principais ou acessórias que constituam objeto da empresa.
    ///
    /// Receita Bruta:
    /// receitas das atividades principais ou acessórias oriundas
    /// da venda de produtos e da prestação de serviços.
    ///
    /// Esta é uma lista com possíveis Receitas Não Operacionais
    /// a depender das atividades que constituam objeto da empresa.
    pub fn descricao_de_outras_receitas(&self) -> bool {
        OUTRAS_RECEITAS_REGEX.is_match(&self.descr_item)
            || OUTRAS_RECEITAS_REGEX.is_match(&self.nome_da_conta)
            || OUTRAS_RECEITAS_REGEX.is_match(&self.complementar)
    }

    /**
    A Receita Bruta compreende:

    I - o produto da venda de bens nas operações de conta própria;

    II - o preço da prestação de serviços em geral;

    III - o resultado auferido nas operações de conta alheia; e

    IV - as receitas da atividade ou objeto principal da pessoa jurídica não compreendidas nos incisos I a III.

    Legislação:

    art. 12 do Decreto-Lei nº 1.598/1977 (legislação do Imposto sobre a Renda (IR))

    art. 3º da Lei nº 9.715/1998

    Em geral, não integram o cálculo da Receita Bruta:

    Venda de Ativo Imobilizado (CFOP 5551, 6551 e 7551),

    Receitas Financeiras,

    Receitas de Variação Cambial,

    Descontos Financeiros,

    Juros sobre Recebimento ou Juros sobre Capital Próprio,

    Hedge,

    Receitas de Aluguéis de bens móveis e imóveis,

    entre outras.
    */
    pub fn saida_de_receita_bruta(&self) -> bool {
        self.tipo_de_operacao
            .is_some_and(|t| t == TipoOperacao::Saida)
            && self.cst_de_receita_bruta()
            && !self.cfop_de_venda_de_imobilizado()
            && !self.aliquota_de_receita_financeira()
            && !self.descricao_de_outras_receitas()
    }

    pub fn format(&mut self) {
        // 44 digits: exemplo NFe: 01234567890123456789012345678901234567890123 --> 01-2345-67.890.123/4567-89-01-234-567.890.123-456.789.012-3
        // 14 digits: exemplo CNPJ: 01234567000890 --> 01.234.567/0008-90
        // 11 digits: exemplo CPF: 12345678901     --> 123.456.789-01
        //  8 digits: exemplo NCM: 01234567        --> 0123.45.67

        let fields: [(&str, &mut String); 5] = [
            ("cnpj", &mut self.estabelecimento_cnpj),
            ("cnpj", &mut self.participante_cnpj),
            ("cpf", &mut self.participante_cpf),
            ("ncm", &mut self.cod_ncm),
            ("chave", &mut self.chave_doc),
        ];

        // https://stackoverflow.com/questions/71275260/how-to-iterate-over-an-array-with-rayon
        // for_each take closure whose return type is unit ()
        fields
            .into_iter()
            //.into_par_iter() // rayon: parallel iterator
            .filter(|(_t, field)| field.is_ascii_alphanumeric())
            .for_each(|(t, field)| match (t, field.chars().count()) {
                ("cnpj", 14) => {
                    *field = field.format_cnpj();
                }
                ("cpf", 11) => {
                    *field = field.format_cpf();
                }
                ("ncm", 8) => {
                    *field = field.format_ncm();
                }
                ("chave", 44) => {
                    *field = [
                        &field[0..2],
                        "-",
                        &field[2..6],
                        "-",
                        &field[6..8],
                        ".",
                        &field[8..11],
                        ".",
                        &field[11..14],
                        "/",
                        &field[14..18],
                        "-",
                        &field[18..20],
                        "-",
                        &field[20..22],
                        "-",
                        &field[22..25],
                        "-",
                        &field[25..28],
                        ".",
                        &field[28..31],
                        ".",
                        &field[31..34],
                        "-",
                        &field[34..37],
                        ".",
                        &field[37..40],
                        ".",
                        &field[40..43],
                        "-",
                        &field[43..],
                    ]
                    .concat();
                }
                _ => (),
            });
    }
}

#[allow(dead_code)]
mod option_date {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer, de::Error};

    const FORMAT: &str = "%-d/%-m/%Y %H:%M:%S";

    pub fn serialize<S>(date: &Option<NaiveDate>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(ref d) = *date {
            return s.serialize_str(&d.format("%d/%m/%Y").to_string());
        }
        s.serialize_none()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        if let Some(s) = s {
            return Ok(Some(NaiveDate::parse_from_str(&s, FORMAT).map_err({
                //eprintln!("Option<NaiveDate> Error: {s:?}");
                Error::custom
            })?));
        }

        Ok(None)
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
/// cargo test -- --show-output docs_fiscais
#[cfg(test)]
mod docs_fiscais_tests {
    use super::*;
    use claudiofsr_lib::print_split;

    // cargo test -- --help
    // cargo test -- --nocapture
    // cargo test -- --show-output

    #[test]
    fn cst_all_contains() {
        // cargo test -- --show-output cst_all_contains

        print_split(&CST_ALL.map(Some), 10);

        let samples = [Some(5), Some(99), None, Some(100), Some(0), Some(1)];
        let mut contains = Vec::new();

        for value in samples {
            if CST_ALL.map(Some).contains(&value) {
                contains.push(value);
            }
        }

        println!();
        println!("samples: {samples:?}");
        println!("contains: {contains:?}");
        assert_eq!(contains, &[Some(5), Some(99), Some(1)]);
    }

    #[test]
    fn show_headers() {
        // cargo test -- --show-output show_headers
        DocsFiscais::get_number_of_fields();
        let headers = DocsFiscais::get_headers();
        println!("Colunas headers: {headers:#?}");
        assert_eq!(&headers[0], "Linhas");
        assert_eq!(&headers[1], "Arquivo da EFD Contribuições");
    }

    #[test]
    fn formatar_colunas() {
        // cargo test -- --show-output formatar_colunas
        let mut colunas = DocsFiscais {
            ..Default::default()
        };
        colunas.estabelecimento_cnpj = "01234567000890".to_string();
        colunas.participante_cnpj = "12345678000912".to_string();
        colunas.participante_cpf = "12345678901".to_string();
        colunas.cod_ncm = "01234567".to_string();
        colunas.chave_doc = "01234567890123456789012345678901234567890123".to_string();
        println!("Colunas: {colunas:#?}");

        colunas.format();
        println!("Colunas: {colunas:#?}");
        assert_eq!(colunas.estabelecimento_cnpj, "01.234.567/0008-90");
        assert_eq!(colunas.participante_cnpj, "12.345.678/0009-12");
        assert_eq!(colunas.participante_cpf, "123.456.789-01");
        assert_eq!(colunas.cod_ncm, "0123.45.67");
        assert_eq!(
            colunas.chave_doc,
            "01-2345-67.890.123/4567-89-01-234-567.890.123-456.789.012-3"
        );
    }

    #[test]
    fn classificar_receitas() {
        // cargo test -- --show-output classificar_receitas
        let mut colunas = DocsFiscais {
            ..Default::default()
        };
        colunas.descr_item = "Descrição do Item".to_string();
        colunas.nome_da_conta = "Nome da Conta".to_string();
        colunas.complementar = "Informação Complementar".to_string();
        println!("Colunas: {colunas:#?}");

        let mut linhas: Vec<DocsFiscais> = Vec::new();
        let mut receitas: Vec<bool> = Vec::from([colunas.descricao_de_outras_receitas()]);

        linhas.push({
            let mut colunas = colunas.clone();
            colunas.descr_item = "123 VARIAÇÃO Cambial ABC".to_string();
            colunas
        });

        linhas.push({
            let mut colunas = colunas.clone();
            colunas.nome_da_conta = "XYZ Juros SOBRE Capital PROPRIO WWW".to_string();
            colunas
        });

        linhas.push({
            let mut colunas = colunas;
            colunas.complementar = "AAA Receitas FINANCEIRAS BBB".to_string();
            colunas
        });

        for (index, linha) in linhas.iter().enumerate() {
            println!("linha {index}: {linha:?}\n");
            receitas.push(linha.descricao_de_outras_receitas())
        }

        println!("receitas: {receitas:?}\n");

        assert_eq!(receitas, [false, true, true, true]);
    }
}

/*
Introduzir as colunas:
    'Verificação da Chave'
    'Natureza do Frete Contratado'
*/
