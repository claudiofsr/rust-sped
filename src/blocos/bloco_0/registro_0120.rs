use crate::{EFDError, EFDResult, ResultExt, SpedParser, StringParser, impl_reg_methods};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "0120";

#[derive(Debug, Clone)]
pub struct Registro0120 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub mes_refer: Option<Arc<str>>, // 2
    pub inf_comp: Option<Arc<str>>,  // 3
}

impl_reg_methods!(Registro0120);

impl SpedParser for Registro0120 {
    type Output = Registro0120;

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

        let mes_refer = fields.get(2).to_arc();
        let inf_comp = fields.get(3).to_arc();

        let reg = Registro0120 {
            nivel: 2,
            bloco: '0',
            registro: REGISTRO.into(),
            line_number,
            mes_refer,
            inf_comp,
        };

        Ok(reg)
    }
}
