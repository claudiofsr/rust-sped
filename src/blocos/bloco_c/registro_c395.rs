use crate::{
    EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalNaiveDate, ToOptionalString,
    impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct RegistroC395 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_mod: Option<String>,   // 2
    pub cod_part: Option<String>,  // 3
    pub ser: Option<String>,       // 4
    pub sub_ser: Option<String>,   // 5
    pub num_doc: Option<String>,   // 6
    pub dt_doc: Option<NaiveDate>, // 7
    pub vl_doc: Option<Decimal>,   // 8
}

impl_sped_record_trait!(RegistroC395);

impl SpedParser for RegistroC395 {
    type Output = RegistroC395;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro C395 possui 8 campos de dados + 2 delimitadores = 10.
        if len != 10 {
            return Err(EFDError::InvalidLength {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 10,
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

        let cod_mod = fields.get(2).to_optional_string();
        let cod_part = fields.get(3).to_optional_string();
        let ser = fields.get(4).to_optional_string();
        let sub_ser = fields.get(5).to_optional_string();
        let num_doc = fields.get(6).to_optional_string();
        let dt_doc = get_date_field(7, "DT_DOC")?;
        let vl_doc = get_decimal_field(8, "VL_DOC")?;

        let reg = RegistroC395 {
            nivel: 3,
            bloco: 'C',
            registro,
            line_number,
            cod_mod,
            cod_part,
            ser,
            sub_ser,
            num_doc,
            dt_doc,
            vl_doc,
        };

        Ok(reg)
    }
}
