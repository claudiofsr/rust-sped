use crate::{EFDError, EFDResult, SpedParser, StringParser, impl_sped_record_trait};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "A010";

#[derive(Debug, Clone)]
pub struct RegistroA010 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cnpj: Option<Arc<str>>, // 2
}

impl_sped_record_trait!(RegistroA010);

impl SpedParser for RegistroA010 {
    type Output = RegistroA010;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro A010 possui 2 campos de dados + 2 delimitadores = 4.
        if len != 4 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 4,
                tamanho_encontrado: len,
            });
        }

        let cnpj = fields.get(2).to_arc();

        let reg = RegistroA010 {
            nivel: 2,
            bloco: 'A',
            registro: REGISTRO.to_string(),
            line_number,
            cnpj,
        };

        Ok(reg)
    }
}
