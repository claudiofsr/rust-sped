use crate::{EFDError, EFDResult, SpedParser, StringParser, impl_reg_methods};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "C110";

#[derive(Debug, Clone)]
pub struct RegistroC110 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: Arc<str>,
    pub line_number: usize,
    pub cod_inf: Option<Arc<str>>,   // 2
    pub txt_compl: Option<Arc<str>>, // 3
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
            });
        }

        let cod_inf = fields.get(2).to_arc();
        let txt_compl = fields.get(3).to_upper_arc();

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
