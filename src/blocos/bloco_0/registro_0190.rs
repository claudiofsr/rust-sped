use crate::{EFDError, EFDResult, SpedParser, ToOptionalString, impl_sped_record_trait};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Registro0190 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub unid: Option<String>,  // 2
    pub descr: Option<String>, // 3
}

impl_sped_record_trait!(Registro0190);

impl SpedParser for Registro0190 {
    type Output = Registro0190;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        if len != 5 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 5,
                tamanho_encontrado: len,
            });
        }

        let unid = fields.get(2).to_optional_string();
        let descr = fields.get(3).to_optional_string();

        let reg = Registro0190 {
            nivel: 3,
            bloco: '0',
            registro,
            line_number,
            unid,
            descr,
        };

        Ok(reg)
    }
}
