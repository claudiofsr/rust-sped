use crate::{EFDError, EFDResult, SpedParser, StringParser, impl_sped_record_trait};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "0190";

#[derive(Debug, Clone)]
pub struct Registro0190 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub unid: Option<Arc<str>>,  // 2
    pub descr: Option<Arc<str>>, // 3
}

impl_sped_record_trait!(Registro0190);

impl SpedParser for Registro0190 {
    type Output = Registro0190;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != 5 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 5,
                tamanho_encontrado: len,
            });
        }

        let unid = fields.get(2).to_arc();
        let descr = fields.get(3).to_arc();

        let reg = Registro0190 {
            nivel: 3,
            bloco: '0',
            registro: REGISTRO.to_string(),
            line_number,
            unid,
            descr,
        };

        Ok(reg)
    }
}
