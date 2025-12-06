use crate::{DocsFiscais, DocsFiscaisNew, impl_from_struct};
use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct Informacoes {
    pub cnpj_base: u32,
    pub periodo_de_apuracao: NaiveDate,
    pub messages: String,
    pub all_docs: Vec<DocsFiscais>,
}

#[derive(Debug, Clone)]
pub struct InformacoesNew {
    pub cnpj_base: u32,
    pub periodo_de_apuracao: NaiveDate,
    pub messages: String,
    pub all_docs: Vec<DocsFiscaisNew>,
}

impl_from_struct!(
    from InformacoesNew to Informacoes,

    // Campos com tipos idênticos (copia direta)
    simple: [
        cnpj_base,
        periodo_de_apuracao,
        messages
    ],

    // Campos de texto que precisam de conversão Arc<str> -> String
    // (Deixei vazio pois 'messages' na sua struct já é String, mas se mudar para Arc, mova para cá)
    strings: [],

    // Lógica customizada para o vetor
    custom: {
        // Itera sobre o Vec<DocsFiscaisNew>, converte cada item para DocsFiscais e coleta num novo Vec
        all_docs: |docs: Vec<DocsFiscaisNew>| docs.into_iter().map(DocsFiscais::from).collect()
    }
);
