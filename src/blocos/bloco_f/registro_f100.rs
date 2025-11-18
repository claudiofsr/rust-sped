use crate::{
    EFDError, EFDResult, SpedParser, ToDecimal, ToNaiveDate, ToOptionalString,
    impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct RegistroF100 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_oper: Option<String>,      // 2
    pub cod_part: Option<String>,      // 3
    pub cod_item: Option<String>,      // 4
    pub dt_oper: Option<NaiveDate>,    // 5
    pub vl_oper: Option<Decimal>,      // 6
    pub cst_pis: Option<String>,       // 7
    pub vl_bc_pis: Option<Decimal>,    // 8
    pub aliq_pis: Option<Decimal>,     // 9
    pub vl_pis: Option<Decimal>,       // 10
    pub cst_cofins: Option<String>,    // 11
    pub vl_bc_cofins: Option<Decimal>, // 12
    pub aliq_cofins: Option<Decimal>,  // 13
    pub vl_cofins: Option<Decimal>,    // 14
    pub nat_bc_cred: Option<String>,   // 15
    pub ind_orig_cred: Option<String>, // 16
    pub cod_cta: Option<String>,       // 17
    pub cod_ccus: Option<String>,      // 18
    pub desc_doc_oper: Option<String>, // 19
}

impl_sped_record_trait!(RegistroF100);

impl SpedParser for RegistroF100 {
    type Output = RegistroF100;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro F100 possui 19 campos de dados + 2 delimitadores = 21.
        if len != 21 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 21,
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

        let ind_oper = fields.get(2).to_optional_string();
        let cod_part = fields.get(3).to_optional_string();
        let cod_item = fields.get(4).to_optional_string();
        let dt_oper = get_date_field(5, "DT_OPER")?;
        let vl_oper = get_decimal_field(6, "VL_OPER")?;
        let cst_pis = fields.get(7).to_optional_string();
        let vl_bc_pis = get_decimal_field(8, "VL_BC_PIS")?;
        let aliq_pis = get_decimal_field(9, "ALIQ_PIS")?;
        let vl_pis = get_decimal_field(10, "VL_PIS")?;
        let cst_cofins = fields.get(11).to_optional_string();
        let vl_bc_cofins = get_decimal_field(12, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal_field(13, "ALIQ_COFINS")?;
        let vl_cofins = get_decimal_field(14, "VL_COFINS")?;
        let nat_bc_cred = fields.get(15).to_optional_string();
        let ind_orig_cred = fields.get(16).to_optional_string();
        let cod_cta = fields.get(17).to_optional_string();
        let cod_ccus = fields.get(18).to_optional_string();
        let desc_doc_oper = fields.get(19).to_optional_string();

        let reg = RegistroF100 {
            nivel: 3,
            bloco: 'F',
            registro,
            line_number,
            ind_oper,
            cod_part,
            cod_item,
            dt_oper,
            vl_oper,
            cst_pis,
            vl_bc_pis,
            aliq_pis,
            vl_pis,
            cst_cofins,
            vl_bc_cofins,
            aliq_cofins,
            vl_cofins,
            nat_bc_cred,
            ind_orig_cred,
            cod_cta,
            cod_ccus,
            desc_doc_oper,
        };

        Ok(reg)
    }
}
