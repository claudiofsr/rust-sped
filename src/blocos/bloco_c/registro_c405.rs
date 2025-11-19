use crate::{
    EFDError, EFDResult, SpedParser, ToDecimal, ToNaiveDate, ToOptionalString,
    impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct RegistroC405 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: String,
    pub line_number: usize,
    pub dt_doc: Option<NaiveDate>,   // 2
    pub cro: Option<String>,         // 3
    pub crz: Option<String>,         // 4
    pub num_coo_fin: Option<String>, // 5
    pub gt_fin: Option<Decimal>,     // 6 (Assumindo que GT_FIN é um valor decimal)
    pub vl_brt: Option<Decimal>,     // 7
}

impl_sped_record_trait!(RegistroC405);

impl SpedParser for RegistroC405 {
    type Output = RegistroC405;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        if len != 9 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 9,
                tamanho_encontrado: len,
            });
        }

        let get_date_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path.to_path_buf(), line_number, field_name)
        };

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let dt_doc = get_date_field(2, "DT_DOC")?;
        let cro = fields.get(3).to_optional_string();
        let crz = fields.get(4).to_optional_string();
        let num_coo_fin = fields.get(5).to_optional_string();
        let gt_fin = get_decimal_field(6, "GT_FIN")?;
        let vl_brt = get_decimal_field(7, "VL_BRT")?;

        let reg = RegistroC405 {
            nivel: 4,
            bloco: 'C',
            registro,
            line_number,
            dt_doc,
            cro,
            crz,
            num_coo_fin,
            gt_fin,
            vl_brt,
        };

        Ok(reg)
    }
}
