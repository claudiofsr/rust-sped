use crate::{EFDError, EFDResult, SpedParser, ToOptionalString, impl_sped_record_trait};
use std::path::Path;

#[derive(Debug)]
pub struct Registro0450 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_inf: Option<String>, // 2
    pub txt: Option<String>,     // 3
}

impl_sped_record_trait!(Registro0450);

impl SpedParser for Registro0450 {
    type Output = Registro0450;

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

        let cod_inf = fields.get(2).to_optional_string();
        let txt = fields.get(3).to_optional_string();

        let reg = Registro0450 {
            nivel: 3,
            bloco: '0',
            registro,
            line_number,
            cod_inf,
            txt,
        };

        Ok(reg)
    }
}
