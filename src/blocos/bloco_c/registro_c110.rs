use crate::{EFDError, EFDResult, SpedParser, ToOptionalString, impl_sped_record_trait};
use std::path::Path;

#[derive(Debug)]
pub struct RegistroC110 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: String,
    pub line_number: usize,
    pub cod_inf: Option<String>,   // 2
    pub txt_compl: Option<String>, // 3
}

impl_sped_record_trait!(RegistroC110);

impl SpedParser for RegistroC110 {
    type Output = RegistroC110;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        if len != 5 {
            return Err(EFDError::InvalidLength {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 5,
                tamanho_encontrado: len,
            });
        }

        let cod_inf = fields.get(2).to_optional_string();
        let txt_compl = fields.get(3).to_optional_string();

        let reg = RegistroC110 {
            nivel: 4,
            bloco: 'C',
            registro,
            line_number,
            cod_inf,
            txt_compl,
        };

        Ok(reg)
    }
}
