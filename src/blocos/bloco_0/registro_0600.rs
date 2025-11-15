use crate::{
    EFDError, EFDResult, SpedParser, ToOptionalNaiveDate, ToOptionalString, impl_sped_record_trait,
};
use chrono::NaiveDate;
use std::path::Path;

#[derive(Debug)]
pub struct Registro0600 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub dt_alt: Option<NaiveDate>, // 2
    pub cod_ccus: Option<String>,  // 3
    pub ccus: Option<String>,      // 4
}

impl_sped_record_trait!(Registro0600);

impl SpedParser for Registro0600 {
    type Output = Registro0600;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        if len != 6 {
            return Err(EFDError::InvalidLength {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 6,
                tamanho_encontrado: len,
            });
        }

        let get_date_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path.to_path_buf(), line_number, field_name)
        };

        let dt_alt = get_date_field(2, "DT_ALT")?;
        let cod_ccus = fields.get(3).to_optional_string();
        let ccus = fields.get(4).to_optional_string();

        let reg = Registro0600 {
            nivel: 2,
            bloco: '0',
            registro,
            line_number,
            dt_alt,
            cod_ccus,
            ccus,
        };

        Ok(reg)
    }
}
