use crate::{
    EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalNaiveDate, ToOptionalString,
    impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct RegistroD200 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_mod: Option<String>,     // 2
    pub cod_sit: Option<String>,     // 3
    pub ser: Option<String>,         // 4
    pub sub: Option<String>,         // 5
    pub num_doc_ini: Option<String>, // 6
    pub num_doc_fin: Option<String>, // 7
    pub cfop: Option<String>,        // 8
    pub dt_ref: Option<NaiveDate>,   // 9
    pub vl_doc: Option<Decimal>,     // 10
    pub vl_desc: Option<Decimal>,    // 11
}

impl_sped_record_trait!(RegistroD200);

impl SpedParser for RegistroD200 {
    type Output = RegistroD200;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro D200 possui 11 campos de dados + 2 delimitadores = 13.
        if len != 13 {
            return Err(EFDError::InvalidLength {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 13,
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

        let cod_mod = fields.get(2).to_optional_string();
        let cod_sit = fields.get(3).to_optional_string();
        let ser = fields.get(4).to_optional_string();
        let sub = fields.get(5).to_optional_string();
        let num_doc_ini = fields.get(6).to_optional_string();
        let num_doc_fin = fields.get(7).to_optional_string();
        let cfop = fields.get(8).to_optional_string();
        let dt_ref = get_date_field(9, "DT_REF")?;
        let vl_doc = get_decimal_field(10, "VL_DOC")?;
        let vl_desc = get_decimal_field(11, "VL_DESC")?;

        let reg = RegistroD200 {
            nivel: 3,
            bloco: 'D',
            registro,
            line_number,
            cod_mod,
            cod_sit,
            ser,
            sub,
            num_doc_ini,
            num_doc_fin,
            cfop,
            dt_ref,
            vl_doc,
            vl_desc,
        };

        Ok(reg)
    }
}
