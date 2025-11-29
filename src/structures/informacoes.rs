use crate::DocsFiscais;
use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct Informacoes {
    pub cnpj_base: u32,
    pub periodo_de_apuracao: NaiveDate,
    pub messages: String,
    pub all_docs: Vec<DocsFiscais>,
}
