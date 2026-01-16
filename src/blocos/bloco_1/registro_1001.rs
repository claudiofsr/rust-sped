use crate::{EFDError, EFDResult, SpedParser, StringParser, impl_reg_methods};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "1001";

#[derive(Debug, Clone)]
pub struct Registro1001 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_mov: Option<char>, // 2
}

impl_reg_methods!(Registro1001);

impl SpedParser for Registro1001 {
    type Output = Registro1001;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro 1001 possui 2 campos de dados + 2 delimitadores = 4.
        if len != 4 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 4,
                tamanho_encontrado: len,
            });
        }

        let ind_mov = fields.get(2).parse_opt();

        let reg = Registro1001 {
            nivel: 1,
            bloco: '1',
            registro: REGISTRO.into(),
            line_number,
            ind_mov,
        };

        Ok(reg)
    }
}
