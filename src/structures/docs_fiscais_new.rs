use chrono::NaiveDate;
use claudiofsr_lib::{
    CFOP_VENDA_DE_IMOBILIZADO, CODIGO_DA_NATUREZA_BC, CST_ALL, CST_CREDITO, CST_RECEITA_BRUTA,
    OUTRAS_RECEITAS_REGEX, StrExtension, match_cast,
};
use csv::StringRecord;
use rust_xlsxwriter::serialize_option_datetime_to_excel;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::serde_introspect;
use std::sync::Arc;
use struct_iterable::Iterable;

use crate::{
    DocsFiscais, FloatExt, IndicadorOrigem, InfoExtension, MesesDoAno, TipoDeCredito, TipoDeRateio,
    TipoDoItem, TipoOperacao, impl_from_struct, serialize_natureza,
};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone, Iterable)]
pub struct DocsFiscaisNew {
    #[serde(rename = "Linhas")]
    pub linhas: usize,

    #[serde(rename = "Arquivo da EFD Contribuições")]
    pub arquivo_efd: Arc<str>,

    #[serde(
        rename = "Nº da Linha da EFD",
        deserialize_with = "csv::invalid_option"
    )]
    pub num_linha_efd: Option<usize>,

    #[serde(rename = "CNPJ dos Estabelecimentos do Contribuinte")]
    pub estabelecimento_cnpj: Arc<str>,

    #[serde(rename = "Nome do Contribuinte")]
    pub estabelecimento_nome: Arc<str>,

    #[serde(
        rename = "Período de Apuração",
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
    pub registro: Arc<str>,

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
        serialize_with = "serialize_natureza"
    )]
    pub natureza_bc: Option<u16>,

    #[serde(rename = "CNPJ do Participante")]
    pub participante_cnpj: Arc<str>,

    #[serde(rename = "CPF do Participante")]
    pub participante_cpf: Arc<str>,

    #[serde(rename = "Nome do Participante")]
    pub participante_nome: Arc<str>,

    #[serde(
        rename = "Nº do Documento Fiscal",
        deserialize_with = "csv::invalid_option"
    )]
    pub num_doc: Option<usize>,

    #[serde(rename = "Chave do Documento")]
    pub chave_doc: Arc<str>,

    //#[serde(rename = "Verificação da Chave")]
    //pub verificacao_chave: Arc<str>,
    #[serde(rename = "Modelo do Documento Fiscal")]
    pub modelo_doc_fiscal: Arc<str>,

    #[serde(
        rename = "Nº do Item do Documento Fiscal",
        deserialize_with = "csv::invalid_option"
    )]
    pub num_item: Option<u16>,

    #[serde(rename = "Tipo do Item")]
    pub tipo_item: Option<TipoDoItem>,

    #[serde(rename = "Descrição do Item")]
    pub descr_item: Arc<str>,

    #[serde(rename = "Código NCM")]
    pub cod_ncm: Arc<str>,

    #[serde(rename = "Natureza da Operação/Prestação")]
    pub nat_operacao: Arc<str>,

    #[serde(rename = "Informação Complementar do Documento Fiscal")]
    pub complementar: Arc<str>,

    #[serde(rename = "Escrituração Contábil: Nome da Conta")]
    pub nome_da_conta: Arc<str>,

    #[serde(
        rename = "Data da Emissão do Documento Fiscal",
        serialize_with = "serialize_option_datetime_to_excel"
    )]
    pub data_emissao: Option<NaiveDate>,

    #[serde(
        rename = "Data da Entrada / Aquisição / Execução ou da Saída / Prestação / Conclusão",
        serialize_with = "serialize_option_datetime_to_excel"
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
impl InfoExtension for DocsFiscaisNew {}

trait PrintOptionFloat {
    fn to_some_string(&self) -> Option<String>;
}

impl PrintOptionFloat for Option<f64> {
    fn to_some_string(&self) -> Option<String> {
        self.map(|float_64| {
            if float_64.fract().eh_zero() {
                format!("{float_64:.1}")
            } else {
                float_64.to_string()
            }
        })
    }
}

impl DocsFiscaisNew {
    pub fn get_headers() -> StringRecord {
        let colunas_vec = serde_introspect::<DocsFiscaisNew>();
        StringRecord::from(colunas_vec)
    }

    pub fn get_values(&self) -> Vec<String> {
        self.iter()
            .map(|(_field, value)| {
                match_cast!( value {
                    val as Option<u16> => { val.as_ref().map(|s| s.to_string()) },
                    val as Option<u32> => { val.as_ref().map(|s| s.to_string()) },
                    val as Option<i32> => { val.as_ref().map(|s| s.to_string()) },
                    val as Option<usize> => { val.as_ref().map(|s| s.to_string()) },
                    val as Option<f64> => { val.to_some_string() },
                    val as Option<NaiveDate> => { val.as_ref().map(|s| s.to_string()) },
                    val as Option<IndicadorOrigem> => { val.as_ref().map(|&indicador| (indicador as u8).to_string()) },
                    val as Option<TipoOperacao> => { val.as_ref().map(|&tipo| (tipo as u8).to_string()) },
                    val as Option<TipoDeCredito> => { val.as_ref().map(|&tipo| (tipo as u8).to_string()) },
                    val as Option<TipoDoItem> => { val.as_ref().map(|&tipo| tipo.descricao_com_codigo()) },
                    val as Option<MesesDoAno> => { val.as_ref().map(|&mes| (mes as u8).to_string()) },
                    val as Option<TipoDeRateio> => { val.as_ref().map(|&tipo| (tipo as u8).to_string()) },
                    val as usize => { Some(val.to_string()) },
                    val as Arc<str> => { Some(val.to_string()) },
                    val as String => { Some(val.to_string()) },
                }).unwrap_or_default()
            })
            .collect()
    }

    pub fn get_cnpj_base(&self) -> String {
        if self.estabelecimento_cnpj.len() >= 10 {
            self.estabelecimento_cnpj[0..10].to_string()
        } else {
            self.estabelecimento_cnpj.to_string()
        }
    }

    pub fn cst_valido(&self) -> bool {
        self.cst
            .is_some_and(|value| CST_ALL.binary_search(&value).is_ok())
    }

    pub fn cst_de_credito(&self) -> bool {
        self.cst
            .is_some_and(|value| CST_CREDITO.binary_search(&value).is_ok())
    }

    pub fn cst_de_receita_bruta(&self) -> bool {
        self.cst
            .is_some_and(|value| CST_RECEITA_BRUTA.binary_search(&value).is_ok())
    }

    pub fn natureza_da_base_de_calculo(&self) -> bool {
        self.natureza_bc
            .is_some_and(|value| CODIGO_DA_NATUREZA_BC.binary_search(&value).is_ok())
    }

    pub fn aliquota_de_receita_financeira(&self) -> bool {
        self.aliq_pis == Some(0.65) && self.aliq_cofins == Some(4.00)
    }

    pub fn aliquota_de_receita_cumulativa(&self) -> bool {
        self.aliq_pis == Some(0.65) && self.aliq_cofins == Some(3.00)
    }

    pub fn cfop_de_venda_de_imobilizado(&self) -> bool {
        self.cfop
            .is_some_and(|value| CFOP_VENDA_DE_IMOBILIZADO.binary_search(&value).is_ok())
    }

    pub fn entrada_de_credito(&self) -> bool {
        self.cst_de_credito()
            && self.natureza_da_base_de_calculo()
            && self
                .tipo_de_operacao
                .is_some_and(|t| t == TipoOperacao::Entrada)
            && self.tipo_de_credito.is_some()
    }

    pub fn operacoes_de_entrada_ou_saida(&self) -> bool {
        self.tipo_de_operacao
            .is_some_and(|t| t.is_entrada_ou_saida())
    }

    pub fn descricao_de_outras_receitas(&self) -> bool {
        OUTRAS_RECEITAS_REGEX.is_match(&self.descr_item)
            || OUTRAS_RECEITAS_REGEX.is_match(&self.nome_da_conta)
            || OUTRAS_RECEITAS_REGEX.is_match(&self.complementar)
    }

    pub fn saida_de_receita_bruta(&self) -> bool {
        self.tipo_de_operacao
            .is_some_and(|t| t == TipoOperacao::Saida)
            && self.cst_de_receita_bruta()
            && !self.cfop_de_venda_de_imobilizado()
            && !self.aliquota_de_receita_financeira()
            && !self.descricao_de_outras_receitas()
    }

    /// Formatação dos campos.
    /// Como Arc<str> é imutável, criamos novas strings formatadas e reatribuímos.
    ///
    /// 44 digits: exemplo NFe: 01234567890123456789012345678901234567890123
    /// --> 01-2345-67.890.123/4567-89-01-234-567.890.123-456.789.012-3
    ///
    /// 14 digits: exemplo CNPJ: 01234567000890
    /// --> 01.234.567/0008-90
    ///
    /// 11 digits: exemplo CPF: 12345678901     
    /// --> 123.456.789-01
    ///
    ///  8 digits: exemplo NCM: 01234567        
    /// --> 0123.45.67
    pub fn format(&mut self) {
        // Helper para formatar e substituir apenas se necessário
        fn format_if_needed<F>(
            target: &mut Arc<str>,
            len_check: usize,
            predicate: fn(char) -> bool,
            formatter: F,
        ) where
            F: Fn(&str) -> String,
        {
            if target.len() == len_check && target.chars().all(predicate) {
                // formatter retorna String, Arc::from toma posse sem cópia extra se possível
                *target = Arc::from(formatter(target));
            }
        }

        format_if_needed(
            &mut self.estabelecimento_cnpj,
            14,
            char::is_alphanumeric,
            |s| s.format_cnpj(),
        );
        format_if_needed(
            &mut self.participante_cnpj,
            14,
            char::is_alphanumeric,
            |s| s.format_cnpj(),
        );
        format_if_needed(&mut self.participante_cpf, 11, char::is_alphanumeric, |s| {
            s.format_cpf()
        });
        format_if_needed(&mut self.cod_ncm, 8, char::is_alphanumeric, |s| {
            s.format_ncm()
        });

        let chave = &self.chave_doc;
        if chave.len() == 44 && chave.chars().all(char::is_numeric) {
            let formatted = format!(
                "{}-{}-{}.{}.{}/{}-{}-{}-{}-{}.{}.{}-{}.{}.{}-{}",
                &chave[0..2],
                &chave[2..6],
                &chave[6..8],
                &chave[8..11],
                &chave[11..14],
                &chave[14..18],
                &chave[18..20],
                &chave[20..22],
                &chave[22..25],
                &chave[25..28],
                &chave[28..31],
                &chave[31..34],
                &chave[34..37],
                &chave[37..40],
                &chave[40..43],
                &chave[43..]
            );
            self.chave_doc = Arc::from(formatted);
            //self.verificacao_chave = Arc::from("Válida");
        } else if !chave.is_empty() {
            //self.verificacao_chave = Arc::from("Inválida/Tamanho Incorreto");
        }
    }
}

impl_from_struct!(
    from DocsFiscaisNew to DocsFiscais,

    // Tipos compatíveis (numéricos iguais, datas, enums, usizes)
    simple: [
        linhas,
        num_linha_efd,
        periodo_de_apuracao,
        ano,
        trimestre,
        mes,
        tipo_de_operacao,
        indicador_de_origem,
        cod_credito,
        tipo_de_credito,
        tipo_item,
        cst,
        cfop,
        natureza_bc,
        num_doc,
        data_emissao,
        data_entrada,
        valor_item,
        valor_bc,
        aliq_pis,
        aliq_cofins,
        valor_pis,
        valor_cofins,
        valor_iss,
        valor_bc_icms,
        aliq_icms,
        valor_icms
    ],

    // Converte de Arc<str> para String automaticamente aqui
    strings: [
        arquivo_efd,
        estabelecimento_cnpj,
        estabelecimento_nome,
        registro,
        participante_cnpj,
        participante_cpf,
        participante_nome,
        chave_doc,
        modelo_doc_fiscal,
        descr_item,
        cod_ncm,
        nat_operacao,
        complementar,
        nome_da_conta
    ],

    // Resolve o erro do Option<u16> para Option<u32>
    custom: {
        num_item: |v: Option<u16>| v.map(u32::from),
        // Se houver outros campos numéricos que mudaram de tamanho dentro de Option, adicione aqui.
        // Exemplo hipotético:
        // num_doc: |v: Option<u32>| v.map(|x| x as usize),
    }
);

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//
//
// cargo test -- --help
// cargo test -- --nocapture
// cargo test -- --show-output

/// Run tests with:
/// cargo test -- --show-output docs_fiscais_new_tests
#[cfg(test)]
mod docs_fiscais_new_tests {
    use super::*;
    use claudiofsr_lib::print_split;

    #[test]
    fn cst_all_contains() {
        print_split(&CST_ALL.map(Some), 10);
        let samples = [Some(5), Some(99), None, Some(100), Some(0), Some(1)];
        let mut contains = Vec::new();
        for value in samples {
            if CST_ALL.map(Some).contains(&value) {
                contains.push(value);
            }
        }
        assert_eq!(contains, &[Some(5), Some(99), Some(1)]);
    }

    #[test]
    fn formatar_colunas() {
        let mut colunas = DocsFiscaisNew {
            ..Default::default()
        };
        colunas.estabelecimento_cnpj = Arc::from("01234567000890");
        colunas.participante_cnpj = Arc::from("12345678000912");
        colunas.participante_cpf = Arc::from("12345678901");
        colunas.cod_ncm = Arc::from("01234567");
        colunas.chave_doc = Arc::from("01234567890123456789012345678901234567890123");

        colunas.format();

        assert_eq!(colunas.estabelecimento_cnpj.as_ref(), "01.234.567/0008-90");
        assert_eq!(colunas.participante_cnpj.as_ref(), "12.345.678/0009-12");
        assert_eq!(colunas.participante_cpf.as_ref(), "123.456.789-01");
        assert_eq!(colunas.cod_ncm.as_ref(), "0123.45.67");
        assert_eq!(
            colunas.chave_doc.as_ref(),
            "01-2345-67.890.123/4567-89-01-234-567.890.123-456.789.012-3"
        );
        // assert_eq!(colunas.verificacao_chave.as_ref(), "Válida");
    }

    /// cargo test -- --show-output test_conversao_new_para_old
    #[test]
    fn test_conversao_new_para_old() {
        let doc_fiscais_new = DocsFiscaisNew {
            linhas: 10,
            estabelecimento_nome: "Empresa Teste".into(),
            ..Default::default()
        };
        println!("doc_fiscais_new: {doc_fiscais_new:?}\n");

        // Opção 1: Usando .into() (padrão do Rust)
        let doc_fiscais: DocsFiscais = doc_fiscais_new.into();

        println!("doc_fiscais: {doc_fiscais:?}\n");

        assert_eq!(doc_fiscais.linhas, 10);
        assert_eq!(doc_fiscais.estabelecimento_nome, "Empresa Teste");
    }
}
