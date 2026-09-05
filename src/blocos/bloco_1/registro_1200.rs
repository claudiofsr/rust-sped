use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, ToNaiveDate,
    impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "1200";

#[derive(Debug, Clone)]
pub struct Registro1200 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub per_apur_ant: Option<CompactString>, // 2
    pub nat_cont_rec: Option<CompactString>, // 3
    pub vl_cont_apur: Option<Decimal>,       // 4
    pub vl_cred_pis_desc: Option<Decimal>,   // 5
    pub vl_cont_dev: Option<Decimal>,        // 6
    pub vl_out_ded: Option<Decimal>,         // 7
    pub vl_cont_ext: Option<Decimal>,        // 8
    pub vl_mul: Option<Decimal>,             // 9
    pub vl_jur: Option<Decimal>,             // 10
    pub dt_recol: Option<NaiveDate>,         // 11
}

impl_reg_methods!(Registro1200);

impl SpedParser for Registro1200 {
    type Output = Registro1200;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro 1200 possui 11 campos de dados + 2 delimitadores = 13.
        if len != 13 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 13,
                tamanho_encontrado: len,
            })
            .loc();
        }

        // Closures auxiliares
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

        let per_apur_ant = fields.get(2).to_compact_string();
        let nat_cont_rec = fields.get(3).to_compact_string();
        let vl_cont_apur = get_decimal(4, "VL_CONT_APUR")?;
        let vl_cred_pis_desc = get_decimal(5, "VL_CRED_PIS_DESC")?;
        let vl_cont_dev = get_decimal(6, "VL_CONT_DEV")?;
        let vl_out_ded = get_decimal(7, "VL_OUT_DED")?;
        let vl_cont_ext = get_decimal(8, "VL_CONT_EXT")?;
        let vl_mul = get_decimal(9, "VL_MUL")?;
        let vl_jur = get_decimal(10, "VL_JUR")?;
        let dt_recol = get_date(11, "DT_RECOL")?;

        let reg = Registro1200 {
            nivel: 2,
            bloco: '1',
            registro: REGISTRO.into(),
            line_number,
            per_apur_ant,
            nat_cont_rec,
            vl_cont_apur,
            vl_cred_pis_desc,
            vl_cont_dev,
            vl_out_ded,
            vl_cont_ext,
            vl_mul,
            vl_jur,
            dt_recol,
        };

        Ok(reg)
    }
}
