use crate::{
    EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalNaiveDate, ToOptionalString,
    impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct Registro1050 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub dt_ref: Option<NaiveDate>,    // 2
    pub ind_aj_bc: Option<String>,    // 3
    pub cnpj: Option<String>,         // 4
    pub vl_aj_tot: Option<Decimal>,   // 5
    pub vl_aj_cst01: Option<Decimal>, // 6
    pub vl_aj_cst02: Option<Decimal>, // 7
    pub vl_aj_cst03: Option<Decimal>, // 8
    pub vl_aj_cst04: Option<Decimal>, // 9
    pub vl_aj_cst05: Option<Decimal>, // 10
    pub vl_aj_cst06: Option<Decimal>, // 11
    pub vl_aj_cst07: Option<Decimal>, // 12
    pub vl_aj_cst08: Option<Decimal>, // 13
    pub vl_aj_cst09: Option<Decimal>, // 14
    pub vl_aj_cst49: Option<Decimal>, // 15
    pub vl_aj_cst99: Option<Decimal>, // 16
    pub ind_aprop: Option<String>,    // 17
    pub num_rec: Option<String>,      // 18
    pub info_compl: Option<String>,   // 19
}

impl_sped_record_trait!(Registro1050);

impl SpedParser for Registro1050 {
    type Output = Registro1050;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro 1050 possui 19 campos de dados + 2 delimitadores = 21.
        if len != 21 {
            return Err(EFDError::InvalidLength {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 21,
                tamanho_encontrado: len,
            });
        }

        // Closures auxiliares
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

        let dt_ref = get_date_field(2, "DT_REF")?;
        let ind_aj_bc = fields.get(3).to_optional_string();
        let cnpj = fields.get(4).to_optional_string();
        let vl_aj_tot = get_decimal_field(5, "VL_AJ_TOT")?;
        let vl_aj_cst01 = get_decimal_field(6, "VL_AJ_CST01")?;
        let vl_aj_cst02 = get_decimal_field(7, "VL_AJ_CST02")?;
        let vl_aj_cst03 = get_decimal_field(8, "VL_AJ_CST03")?;
        let vl_aj_cst04 = get_decimal_field(9, "VL_AJ_CST04")?;
        let vl_aj_cst05 = get_decimal_field(10, "VL_AJ_CST05")?;
        let vl_aj_cst06 = get_decimal_field(11, "VL_AJ_CST06")?;
        let vl_aj_cst07 = get_decimal_field(12, "VL_AJ_CST07")?;
        let vl_aj_cst08 = get_decimal_field(13, "VL_AJ_CST08")?;
        let vl_aj_cst09 = get_decimal_field(14, "VL_AJ_CST09")?;
        let vl_aj_cst49 = get_decimal_field(15, "VL_AJ_CST49")?;
        let vl_aj_cst99 = get_decimal_field(16, "VL_AJ_CST99")?;
        let ind_aprop = fields.get(17).to_optional_string();
        let num_rec = fields.get(18).to_optional_string();
        let info_compl = fields.get(19).to_optional_string();

        let reg = Registro1050 {
            nivel: 2,
            bloco: '1',
            registro,
            line_number,
            dt_ref,
            ind_aj_bc,
            cnpj,
            vl_aj_tot,
            vl_aj_cst01,
            vl_aj_cst02,
            vl_aj_cst03,
            vl_aj_cst04,
            vl_aj_cst05,
            vl_aj_cst06,
            vl_aj_cst07,
            vl_aj_cst08,
            vl_aj_cst09,
            vl_aj_cst49,
            vl_aj_cst99,
            ind_aprop,
            num_rec,
            info_compl,
        };

        Ok(reg)
    }
}
