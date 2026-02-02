use crate::{EFDError, EFDResult, ResultExt, SpedParser, StringParser, impl_reg_methods};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "0450";

/// Registro 0450: Tabela de Informação Complementar do Documento Fiscal
#[derive(Debug, Clone)]
pub struct Registro0450 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_inf: Option<Arc<str>>, // 2
    pub txt: Option<Arc<str>>,     // 3
}

impl_reg_methods!(Registro0450);

impl SpedParser for Registro0450 {
    type Output = Registro0450;

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

        let cod_inf = fields.get(2).to_arc();
        let txt = fields.get(3).to_upper_arc();

        let reg = Registro0450 {
            nivel: 3,
            bloco: '0',
            registro: REGISTRO.into(),
            line_number,
            cod_inf,
            txt,
        };

        Ok(reg)
    }
}
