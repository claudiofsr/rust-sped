use crate::{EFDError, EFDResult, SpedParser, ToOptionalString, impl_sped_record_trait};
use std::path::Path;

#[derive(Debug)]
pub struct Registro0001 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_mov: Option<String>, // 2
}

impl_sped_record_trait!(Registro0001);

impl SpedParser for Registro0001 {
    type Output = Registro0001;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        if len != 4 {
            return Err(EFDError::InvalidLength {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(), // Aqui precisa do clone porque `registro` será usado depois.
                tamanho_esperado: 4,
                tamanho_encontrado: len,
            });
        }

        let ind_mov = fields.get(2).to_optional_string();

        Ok(Registro0001 {
            nivel: 1,
            bloco: '0',
            line_number,
            registro,
            ind_mov,
        })
    }
}
