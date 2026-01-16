use crate::{
    EFDError, EFDResult, SpedParser, StringParser, ToDecimal, ToNaiveDate, impl_reg_methods,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "C405";

#[derive(Debug, Clone)]
pub struct RegistroC405 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: Arc<str>,
    pub line_number: usize,
    pub dt_doc: Option<NaiveDate>,     // 2
    pub cro: Option<Arc<str>>,         // 3
    pub crz: Option<Arc<str>>,         // 4
    pub num_coo_fin: Option<Arc<str>>, // 5
    pub gt_fin: Option<Decimal>,       // 6 (Assumindo que GT_FIN é um valor decimal)
    pub vl_brt: Option<Decimal>,       // 7
}

impl_reg_methods!(RegistroC405);

impl SpedParser for RegistroC405 {
    type Output = RegistroC405;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != 9 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 9,
                tamanho_encontrado: len,
            });
        }

        let get_date = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let dt_doc = get_date(2, "DT_DOC")?;
        let cro = fields.get(3).to_arc();
        let crz = fields.get(4).to_arc();
        let num_coo_fin = fields.get(5).to_arc();
        let gt_fin = get_decimal(6, "GT_FIN")?;
        let vl_brt = get_decimal(7, "VL_BRT")?;

        let reg = RegistroC405 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.into(),
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
