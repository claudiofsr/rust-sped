use crate::{
    EFDError, EFDResult, SpedParser, ToDecimal, ToNaiveDate, ToOptionalString,
    impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct RegistroM220 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_aj: Option<String>,    // 2
    pub vl_aj: Option<Decimal>,    // 3
    pub cod_aj: Option<String>,    // 4
    pub num_doc: Option<String>,   // 5
    pub descr_aj: Option<String>,  // 6
    pub dt_ref: Option<NaiveDate>, // 7
}

impl_sped_record_trait!(RegistroM220);

impl SpedParser for RegistroM220 {
    type Output = RegistroM220;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro M220 possui 7 campos de dados + 2 delimitadores = 9.
        if len != 9 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 9,
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

        let ind_aj = fields.get(2).to_optional_string();
        let vl_aj = get_decimal_field(3, "VL_AJ")?;
        let cod_aj = fields.get(4).to_optional_string();
        let num_doc = fields.get(5).to_optional_string();
        let descr_aj = fields.get(6).to_optional_string();
        let dt_ref = get_date_field(7, "DT_REF")?;

        let reg = RegistroM220 {
            nivel: 4,
            bloco: 'M',
            registro,
            line_number,
            ind_aj,
            vl_aj,
            cod_aj,
            num_doc,
            descr_aj,
            dt_ref,
        };

        Ok(reg)
    }
}
