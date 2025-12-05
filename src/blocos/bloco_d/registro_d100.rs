use crate::{
    EFDError, EFDResult, SpedParser, StringParser, ToDecimal, ToNaiveDate, impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "D100";

#[derive(Debug, Clone)]
pub struct RegistroD100 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: String,
    pub line_number: usize,
    pub ind_oper: Option<Arc<str>>,    // 2
    pub ind_emit: Option<Arc<str>>,    // 3
    pub cod_part: Option<Arc<str>>,    // 4
    pub cod_mod: Option<Arc<str>>,     // 5
    pub cod_sit: Option<Arc<str>>,     // 6
    pub ser: Option<Arc<str>>,         // 7
    pub sub: Option<Arc<str>>,         // 8
    pub num_doc: Option<Arc<str>>,     // 9
    pub chv_cte: Option<Arc<str>>,     // 10
    pub dt_doc: Option<NaiveDate>,     // 11
    pub dt_a_p: Option<NaiveDate>,     // 12
    pub tp_cte: Option<Arc<str>>,      // 13
    pub chv_cte_ref: Option<Arc<str>>, // 14
    pub vl_doc: Option<Decimal>,       // 15
    pub vl_desc: Option<Decimal>,      // 16
    pub ind_frt: Option<Arc<str>>,     // 17
    pub vl_serv: Option<Decimal>,      // 18
    pub vl_bc_icms: Option<Decimal>,   // 19
    pub vl_icms: Option<Decimal>,      // 20
    pub vl_nt: Option<Decimal>,        // 21
    pub cod_inf: Option<Arc<str>>,     // 22
    pub cod_cta: Option<Arc<str>>,     // 23
}

impl_sped_record_trait!(RegistroD100);

impl SpedParser for RegistroD100 {
    type Output = RegistroD100;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != 25 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 25,
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

        let ind_oper = fields.get(2).to_arc();
        let ind_emit = fields.get(3).to_arc();
        let cod_part = fields.get(4).to_arc();
        let cod_mod = fields.get(5).to_arc();
        let cod_sit = fields.get(6).to_arc();
        let ser = fields.get(7).to_arc();
        let sub = fields.get(8).to_arc();
        let num_doc = fields.get(9).to_arc();
        let chv_cte = fields.get(10).to_arc();
        let dt_doc = get_date(11, "DT_DOC")?;
        let dt_a_p = get_date(12, "DT_A_P")?;
        let tp_cte = fields.get(13).to_arc();
        let chv_cte_ref = fields.get(14).to_arc();
        let vl_doc = get_decimal(15, "VL_DOC")?;
        let vl_desc = get_decimal(16, "VL_DESC")?;
        let ind_frt = fields.get(17).to_arc();
        let vl_serv = get_decimal(18, "VL_SERV")?;
        let vl_bc_icms = get_decimal(19, "VL_BC_ICMS")?;
        let vl_icms = get_decimal(20, "VL_ICMS")?;
        let vl_nt = get_decimal(21, "VL_NT")?;
        let cod_inf = fields.get(22).to_arc();
        let cod_cta = fields.get(23).to_arc();

        let reg = RegistroD100 {
            nivel: 3,
            bloco: 'D',
            registro: REGISTRO.to_string(),
            line_number,
            ind_oper,
            ind_emit,
            cod_part,
            cod_mod,
            cod_sit,
            ser,
            sub,
            num_doc,
            chv_cte,
            dt_doc,
            dt_a_p,
            tp_cte,
            chv_cte_ref,
            vl_doc,
            vl_desc,
            ind_frt,
            vl_serv,
            vl_bc_icms,
            vl_icms,
            vl_nt,
            cod_inf,
            cod_cta,
        };

        Ok(reg)
    }
}
