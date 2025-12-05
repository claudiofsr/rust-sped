use crate::{EFDError, EFDResult, SpedParser, StringParser, impl_sped_record_trait};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "I001";

#[derive(Debug, Clone)]
pub struct RegistroI001 {
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

impl_sped_record_trait!(RegistroI001);

impl SpedParser for RegistroI001 {
    type Output = RegistroI001;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro I001 possui 2 campos de dados + 2 delimitadores = 4.
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

        let reg = RegistroI001 {
            nivel: 1,
            bloco: 'I',
            registro: REGISTRO.into(),
            line_number,
            ind_mov,
        };

        Ok(reg)
    }
}
