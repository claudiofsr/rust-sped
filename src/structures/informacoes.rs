use crate::{DocsFiscais, DocsFiscaisNew};
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
