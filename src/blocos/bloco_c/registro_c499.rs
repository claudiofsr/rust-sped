use crate::{EFDError, EFDResult, SpedParser, ToOptionalString, impl_sped_record_trait};
use std::path::Path;

const REGISTRO: &str = "C499";

#[derive(Debug, Clone)]
pub struct RegistroC499 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: String,
    pub line_number: usize,
    pub num_proc: Option<String>, // 2
    pub ind_proc: Option<String>, // 3
}

impl_sped_record_trait!(RegistroC499);

impl SpedParser for RegistroC499 {
    type Output = RegistroC499;

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

        let num_proc = fields.get(2).to_optional_string();
        let ind_proc = fields.get(3).to_optional_string();

        let reg = RegistroC499 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.to_string(),
            line_number,
            num_proc,
            ind_proc,
        };

        Ok(reg)
    }
}
