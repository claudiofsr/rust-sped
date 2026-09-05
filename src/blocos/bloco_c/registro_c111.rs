use crate::{EFDError, EFDResult, ResultExt, SpedParser, StringParser, impl_reg_methods};
use compact_str::CompactString;
use std::path::Path;

const REGISTRO: &str = "C111";

#[derive(Debug, Clone)]
pub struct RegistroC111 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: CompactString,
    pub line_number: usize,
    pub num_proc: Option<CompactString>, // 2
    pub ind_proc: Option<CompactString>, // 3
}

impl_reg_methods!(RegistroC111);

impl SpedParser for RegistroC111 {
    type Output = RegistroC111;

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

        let num_proc = fields.get(2).to_compact_string();
        let ind_proc = fields.get(3).to_compact_string();

        let reg = RegistroC111 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            num_proc,
            ind_proc,
        };

        Ok(reg)
    }
}
