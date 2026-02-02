use crate::{EFDError, EFDResult, ResultExt, SpedParser, StringParser, impl_reg_methods};
use compact_str::CompactString;
use std::path::Path;

const REGISTRO: &str = "C110";

#[derive(Debug, Clone)]
pub struct RegistroC110 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: CompactString,
    pub line_number: usize,
    pub cod_inf: Option<CompactString>,   // 2
    pub txt_compl: Option<CompactString>, // 3
}

impl_reg_methods!(RegistroC110);

impl SpedParser for RegistroC110 {
    type Output = RegistroC110;

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

        let cod_inf = fields.get(2).to_compact_string();
        let txt_compl = fields.get(3).map(|&s| s.to_ascii_uppercase().into());

        let reg = RegistroC110 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cod_inf,
            txt_compl,
        };

        Ok(reg)
    }
}
