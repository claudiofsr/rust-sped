use crate::{EFDError, EFDResult, ResultExt, SpedParser, StringParser, impl_reg_methods};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "0400";

#[derive(Debug, Clone)]
pub struct Registro0400 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_nat: Option<Arc<str>>,   // 2
    pub descr_nat: Option<Arc<str>>, // 3
}

impl_reg_methods!(Registro0400);

impl SpedParser for Registro0400 {
    type Output = Registro0400;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != 5 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 5,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let cod_nat = fields.get(2).to_arc();
        let descr_nat = fields.get(3).to_arc();

        let reg = Registro0400 {
            nivel: 3,
            bloco: '0',
            registro: REGISTRO.into(),
            line_number,
            cod_nat,
            descr_nat,
        };

        Ok(reg)
    }
}
