use crate::{
    EFDError, EFDResult, SpedParser, ToDecimal, ToNaiveDate, ToOptionalString,
    impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C500";

#[derive(Debug, Clone)]
pub struct RegistroC500 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: String,
    pub line_number: usize,
    pub cod_part: Option<String>,   // 2
    pub cod_mod: Option<String>,    // 3
    pub cod_sit: Option<String>,    // 4
    pub ser: Option<String>,        // 5
    pub sub: Option<String>,        // 6
    pub num_doc: Option<String>,    // 7
    pub dt_doc: Option<NaiveDate>,  // 8
    pub dt_ent: Option<NaiveDate>,  // 9
    pub vl_doc: Option<Decimal>,    // 10
    pub vl_icms: Option<Decimal>,   // 11
    pub cod_inf: Option<String>,    // 12
    pub vl_pis: Option<Decimal>,    // 13
    pub vl_cofins: Option<Decimal>, // 14
    pub chv_doce: Option<String>,   // 15
}

// O campo 15 pode não existir.
// Campo 15 (CHV_DOCe) - Preenchimento: Informar a chave do documento eletrônico.
// A partir de 01/01/2020, o campo é obrigatório quando COD_MOD for igual a “66” ou “55”.

impl_sped_record_trait!(RegistroC500);

impl SpedParser for RegistroC500 {
    type Output = RegistroC500;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if !(len == 16 || len == 17) {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 17,
                tamanho_encontrado: len,
            });
        }

        let get_date_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cod_part = fields.get(2).to_optional_string();
        let cod_mod = fields.get(3).to_optional_string();
        let cod_sit = fields.get(4).to_optional_string();
        let ser = fields.get(5).to_optional_string();
        let sub = fields.get(6).to_optional_string();
        let num_doc = fields.get(7).to_optional_string();
        let dt_doc = get_date_field(8, "DT_DOC")?;
        let dt_ent = get_date_field(9, "DT_ENT")?;
        let vl_doc = get_decimal_field(10, "VL_DOC")?;
        let vl_icms = get_decimal_field(11, "VL_ICMS")?;
        let cod_inf = fields.get(12).to_optional_string();
        let vl_pis = get_decimal_field(13, "VL_PIS")?;
        let vl_cofins = get_decimal_field(14, "VL_COFINS")?;
        let chv_doce = fields.get(15).to_optional_string();

        let reg = RegistroC500 {
            nivel: 3,
            bloco: 'C',
            registro: REGISTRO.to_string(),
            line_number,
            cod_part,
            cod_mod,
            cod_sit,
            ser,
            sub,
            num_doc,
            dt_doc,
            dt_ent,
            vl_doc,
            vl_icms,
            cod_inf,
            vl_pis,
            vl_cofins,
            chv_doce,
        };

        Ok(reg)
    }
}
