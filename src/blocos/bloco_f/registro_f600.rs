use crate::{
    EFDError, EFDResult, SpedParser, ToDecimal, ToNaiveDate, ToOptionalString,
    impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct RegistroF600 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_nat_ret: Option<String>,    // 2
    pub dt_ret: Option<NaiveDate>,      // 3
    pub vl_bc_ret: Option<Decimal>,     // 4
    pub vl_ret: Option<Decimal>,        // 5
    pub cod_rec: Option<String>,        // 6
    pub ind_nat_rec: Option<String>,    // 7
    pub cnpj: Option<String>,           // 8
    pub vl_ret_pis: Option<Decimal>,    // 9
    pub vl_ret_cofins: Option<Decimal>, // 10
    pub ind_dec: Option<String>,        // 11
}

impl_sped_record_trait!(RegistroF600);

impl SpedParser for RegistroF600 {
    type Output = RegistroF600;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro F600 possui 11 campos de dados + 2 delimitadores = 13.
        if len != 13 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 13,
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

        let ind_nat_ret = fields.get(2).to_optional_string();
        let dt_ret = get_date_field(3, "DT_RET")?;
        let vl_bc_ret = get_decimal_field(4, "VL_BC_RET")?;
        let vl_ret = get_decimal_field(5, "VL_RET")?;
        let cod_rec = fields.get(6).to_optional_string();
        let ind_nat_rec = fields.get(7).to_optional_string();
        let cnpj = fields.get(8).to_optional_string();
        let vl_ret_pis = get_decimal_field(9, "VL_RET_PIS")?;
        let vl_ret_cofins = get_decimal_field(10, "VL_RET_COFINS")?;
        let ind_dec = fields.get(11).to_optional_string();

        let reg = RegistroF600 {
            nivel: 3,
            bloco: 'F',
            registro,
            line_number,
            ind_nat_ret,
            dt_ret,
            vl_bc_ret,
            vl_ret,
            cod_rec,
            ind_nat_rec,
            cnpj,
            vl_ret_pis,
            vl_ret_cofins,
            ind_dec,
        };

        Ok(reg)
    }
}
