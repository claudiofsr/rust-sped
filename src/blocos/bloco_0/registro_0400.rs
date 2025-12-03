use crate::{EFDError, EFDResult, SpedParser, ToOptionalString, impl_sped_record_trait};
use std::path::Path;

const REGISTRO: &str = "0400";

#[derive(Debug, Clone)]
pub struct Registro0400 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_nat: Option<String>,   // 2
    pub descr_nat: Option<String>, // 3
}

impl_sped_record_trait!(Registro0400);

impl SpedParser for Registro0400 {
    type Output = Registro0400;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != 5 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 5,
                tamanho_encontrado: len,
            });
        }

        let cod_nat = fields.get(2).to_optional_string();
        let descr_nat = fields.get(3).to_optional_string();

        let reg = Registro0400 {
            nivel: 3,
            bloco: '0',
            registro: REGISTRO.to_string(),
            line_number,
            cod_nat,
            descr_nat,
        };

        Ok(reg)
    }
}
