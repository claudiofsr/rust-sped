use crate::{
    EFDError, EFDResult, SpedParser, ToDecimal, ToNaiveDate, ToOptionalString,
    impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct RegistroM300 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_cont: Option<String>,            // 2
    pub vl_cont_apur_difer: Option<Decimal>, // 3
    pub nat_cred_desc: Option<String>,       // 4
    pub vl_cred_desc_difer: Option<Decimal>, // 5
    pub vl_cont_difer_ant: Option<Decimal>,  // 6
    pub per_apur: Option<String>,            // 7
    pub dt_receb: Option<NaiveDate>,         // 8
}

impl_sped_record_trait!(RegistroM300);

impl SpedParser for RegistroM300 {
    type Output = RegistroM300;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro M300 possui 8 campos de dados + 2 delimitadores = 10.
        if len != 10 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 10,
                tamanho_encontrado: len,
            });
        }

        // Closure para campos de data (Option<NaiveDate>)
        let get_date_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path.to_path_buf(), line_number, field_name)
        };

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let cod_cont = fields.get(2).to_optional_string();
        let vl_cont_apur_difer = get_decimal_field(3, "VL_CONT_APUR_DIFER")?;
        let nat_cred_desc = fields.get(4).to_optional_string();
        let vl_cred_desc_difer = get_decimal_field(5, "VL_CRED_DESC_DIFER")?;
        let vl_cont_difer_ant = get_decimal_field(6, "VL_CONT_DIFER_ANT")?;
        let per_apur = fields.get(7).to_optional_string();
        let dt_receb = get_date_field(8, "DT_RECEB")?;

        let reg = RegistroM300 {
            nivel: 2,
            bloco: 'M',
            registro,
            line_number,
            cod_cont,
            vl_cont_apur_difer,
            nat_cred_desc,
            vl_cred_desc_difer,
            vl_cont_difer_ant,
            per_apur,
            dt_receb,
        };

        Ok(reg)
    }
}
