use crate::{
    EFDError, EFDResult, SpedParser, ToNaiveDate, ToOptionalString, impl_sped_record_trait,
};
use chrono::NaiveDate;
use std::path::Path;

const REGISTRO: &str = "C490";

#[derive(Debug, Clone)]
pub struct RegistroC490 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: String,
    pub line_number: usize,
    pub dt_doc_ini: Option<NaiveDate>, // 2
    pub dt_doc_fin: Option<NaiveDate>, // 3
    pub cod_mod: Option<String>,       // 4
}

impl_sped_record_trait!(RegistroC490);

impl SpedParser for RegistroC490 {
    type Output = RegistroC490;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != 6 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 6,
                tamanho_encontrado: len,
            });
        }

        let get_date_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let dt_doc_ini = get_date_field(2, "DT_DOC_INI")?;
        let dt_doc_fin = get_date_field(3, "DT_DOC_FIN")?;
        let cod_mod = fields.get(4).to_optional_string();

        let reg = RegistroC490 {
            nivel: 3,
            bloco: 'C',
            registro: REGISTRO.to_string(),
            line_number,
            dt_doc_ini,
            dt_doc_fin,
            cod_mod,
        };

        Ok(reg)
    }
}
