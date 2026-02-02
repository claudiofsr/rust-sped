use crate::{EFDError, EFDResult, ResultExt, SpedParser, StringParser, impl_reg_methods};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "0206";

#[derive(Debug, Clone)]
pub struct Registro0206 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_comb: Option<Arc<str>>, // 2
}

impl_reg_methods!(Registro0206);

impl SpedParser for Registro0206 {
    type Output = Registro0206;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != 4 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 4,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let cod_comb = fields.get(2).to_arc();

        let reg = Registro0206 {
            nivel: 4,
            bloco: '0',
            registro: REGISTRO.into(),
            line_number,
            cod_comb,
        };

        Ok(reg)
    }
}
