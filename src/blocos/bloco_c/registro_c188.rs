use crate::{EFDError, EFDResult, SpedParser, ToOptionalString, impl_sped_record_trait};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct RegistroC188 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub num_proc: Option<String>, // 2
    pub ind_proc: Option<String>, // 3
}

impl_sped_record_trait!(RegistroC188);

impl SpedParser for RegistroC188 {
    type Output = RegistroC188;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro C188 possui 2 campos de dados + 2 delimitadores = 4.
        if len != 4 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 4,
                tamanho_encontrado: len,
            });
        }

        let num_proc = fields.get(2).to_optional_string();
        let ind_proc = fields.get(3).to_optional_string();

        let reg = RegistroC188 {
            nivel: 4,
            bloco: 'C',
            registro,
            line_number,
            num_proc,
            ind_proc,
        };

        Ok(reg)
    }
}
