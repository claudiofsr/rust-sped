use crate::{
    EFDError, EFDResult, SpedParser, ToOptionalNaiveDate, ToOptionalString, impl_sped_record_trait,
};
use chrono::NaiveDate;
use std::path::Path;

#[derive(Debug)]
pub struct Registro0205 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub descr_ant_item: Option<String>, // 2
    pub dt_ini: Option<NaiveDate>,      // 3
    pub dt_fim: Option<NaiveDate>,      // 4
    pub cod_ant_item: Option<String>,   // 5
}

impl_sped_record_trait!(Registro0205);

impl SpedParser for Registro0205 {
    type Output = Registro0205;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        if len != 7 {
            return Err(EFDError::InvalidLength {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 7,
                tamanho_encontrado: len,
            });
        }

        let get_date_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path.to_path_buf(), line_number, field_name)
        };

        let descr_ant_item = fields.get(2).to_optional_string();
        let dt_ini = get_date_field(3, "DT_INI")?;
        let dt_fim = get_date_field(4, "DT_FIM")?;
        let cod_ant_item = fields.get(5).to_optional_string();

        let reg = Registro0205 {
            nivel: 4,
            bloco: '0',
            registro,
            line_number,
            descr_ant_item,
            dt_ini,
            dt_fim,
            cod_ant_item,
        };

        Ok(reg)
    }
}
