use crate::{
    EFDError, EFDResult, SpedParser, StringParser, ToDecimal, ToNaiveDate, impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "1800";

#[derive(Debug, Clone)]
pub struct Registro1800 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub inc_imob: Option<Arc<str>>,     // 2
    pub rec_receb_ret: Option<Decimal>, // 3
    pub rec_fin_ret: Option<Decimal>,   // 4
    pub bc_ret: Option<Arc<str>>,       // 5
    pub aliq_ret: Option<Decimal>,      // 6
    pub vl_rec_uni: Option<Decimal>,    // 7
    pub dt_rec_uni: Option<NaiveDate>,  // 8
    pub cod_rec: Option<Arc<str>>,      // 9
}

impl_sped_record_trait!(Registro1800);

impl SpedParser for Registro1800 {
    type Output = Registro1800;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro 1800 possui 9 campos de dados + 2 delimitadores = 11.
        if len != 11 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 11,
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

        let inc_imob = fields.get(2).to_arc();
        let rec_receb_ret = get_decimal(3, "REC_RECEB_RET")?;
        let rec_fin_ret = get_decimal(4, "REC_FIN_RET")?;
        let bc_ret = fields.get(5).to_arc();
        let aliq_ret = get_decimal(6, "ALIQ_RET")?;
        let vl_rec_uni = get_decimal(7, "VL_REC_UNI")?;
        let dt_rec_uni = get_date(8, "DT_REC_UNI")?;
        let cod_rec = fields.get(9).to_arc();

        let reg = Registro1800 {
            nivel: 2,
            bloco: '1',
            registro: REGISTRO.to_string(),
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
