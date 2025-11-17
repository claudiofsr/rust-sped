use crate::{
    EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalNaiveDate, ToOptionalString,
    impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct Registro1800 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub inc_imob: Option<String>,       // 2
    pub rec_receb_ret: Option<Decimal>, // 3
    pub rec_fin_ret: Option<Decimal>,   // 4
    pub bc_ret: Option<String>,         // 5
    pub aliq_ret: Option<Decimal>,      // 6
    pub vl_rec_uni: Option<Decimal>,    // 7
    pub dt_rec_uni: Option<NaiveDate>,  // 8
    pub cod_rec: Option<String>,        // 9
}

impl_sped_record_trait!(Registro1800);

impl SpedParser for Registro1800 {
    type Output = Registro1800;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro 1800 possui 9 campos de dados + 2 delimitadores = 11.
        if len != 11 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 11,
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

        let inc_imob = fields.get(2).to_optional_string();
        let rec_receb_ret = get_decimal_field(3, "REC_RECEB_RET")?;
        let rec_fin_ret = get_decimal_field(4, "REC_FIN_RET")?;
        let bc_ret = fields.get(5).to_optional_string();
        let aliq_ret = get_decimal_field(6, "ALIQ_RET")?;
        let vl_rec_uni = get_decimal_field(7, "VL_REC_UNI")?;
        let dt_rec_uni = get_date_field(8, "DT_REC_UNI")?;
        let cod_rec = fields.get(9).to_optional_string();

        let reg = Registro1800 {
            nivel: 2,
            bloco: '1',
            registro,
            line_number,
            inc_imob,
            rec_receb_ret,
            rec_fin_ret,
            bc_ret,
            aliq_ret,
            vl_rec_uni,
            dt_rec_uni,
            cod_rec,
        };

        Ok(reg)
    }
}
