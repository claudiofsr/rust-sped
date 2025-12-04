use crate::{EFDError, EFDResult, SpedParser, StringParser, impl_sped_record_trait};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "D010";

#[derive(Debug, Clone)]
pub struct RegistroD010 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: String,
    pub line_number: usize,
    pub cnpj: Option<Arc<str>>, // 2
}

impl_sped_record_trait!(RegistroD010);

impl SpedParser for RegistroD010 {
    type Output = RegistroD010;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

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

        let reg = RegistroD010 {
            nivel: 2,
            bloco: 'D',
            registro: REGISTRO.to_string(),
            line_number,
            cnpj,
        };

        Ok(reg)
    }
}
