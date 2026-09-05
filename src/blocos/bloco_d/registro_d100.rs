use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, ToNaiveDate,
    impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "D100";

#[derive(Debug, Clone)]
pub struct RegistroD100 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: CompactString,
    pub line_number: usize,
    pub ind_oper: Option<CompactString>,    // 2
    pub ind_emit: Option<CompactString>,    // 3
    pub cod_part: Option<CompactString>,    // 4
    pub cod_mod: Option<CompactString>,     // 5
    pub cod_sit: Option<CompactString>,     // 6
    pub ser: Option<CompactString>,         // 7
    pub sub: Option<CompactString>,         // 8
    pub num_doc: Option<usize>,             // 9
    pub chv_cte: Option<CompactString>,     // 10
    pub dt_doc: Option<NaiveDate>,          // 11
    pub dt_a_p: Option<NaiveDate>,          // 12
    pub tp_cte: Option<CompactString>,      // 13
    pub chv_cte_ref: Option<CompactString>, // 14
    pub vl_doc: Option<Decimal>,            // 15
    pub vl_desc: Option<Decimal>,           // 16
    pub ind_frt: Option<CompactString>,     // 17
    pub vl_serv: Option<Decimal>,           // 18
    pub vl_bc_icms: Option<Decimal>,        // 19
    pub vl_icms: Option<Decimal>,           // 20
    pub vl_nt: Option<Decimal>,             // 21
    pub cod_inf: Option<CompactString>,     // 22
    pub cod_cta: Option<CompactString>,     // 23
}

impl_reg_methods!(RegistroD100);

impl SpedParser for RegistroD100 {
    type Output = RegistroD100;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != 25 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 25,
                tamanho_encontrado: len,
            })
            .loc();
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

        let ind_oper = fields.get(2).to_compact_string();
        let ind_emit = fields.get(3).to_compact_string();
        let cod_part = fields.get(4).to_compact_string();
        let cod_mod = fields.get(5).to_compact_string();
        let cod_sit = fields.get(6).to_compact_string();
        let ser = fields.get(7).to_compact_string();
        let sub = fields.get(8).to_compact_string();
        let num_doc = fields.get(9).parse_opt();
        let chv_cte = fields.get(10).to_compact_string();
        let dt_doc = get_date(11, "DT_DOC")?;
        let dt_a_p = get_date(12, "DT_A_P")?;
        let tp_cte = fields.get(13).to_compact_string();
        let chv_cte_ref = fields.get(14).to_compact_string();
        let vl_doc = get_decimal(15, "VL_DOC")?;
        let vl_desc = get_decimal(16, "VL_DESC")?;
        let ind_frt = fields.get(17).to_compact_string();
        let vl_serv = get_decimal(18, "VL_SERV")?;
        let vl_bc_icms = get_decimal(19, "VL_BC_ICMS")?;
        let vl_icms = get_decimal(20, "VL_ICMS")?;
        let vl_nt = get_decimal(21, "VL_NT")?;
        let cod_inf = fields.get(22).to_compact_string();
        let cod_cta = fields.get(23).to_compact_string();

        let reg = RegistroD100 {
            nivel: 3,
            bloco: 'D',
            registro: REGISTRO.into(),
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
