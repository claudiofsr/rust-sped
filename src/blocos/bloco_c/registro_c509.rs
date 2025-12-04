use crate::{EFDError, EFDResult, SpedParser, StringParser, impl_sped_record_trait};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "C509";

#[derive(Debug, Clone)]
pub struct RegistroC509 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub num_proc: Option<Arc<str>>, // 2
    pub ind_proc: Option<Arc<str>>, // 3
}

impl_sped_record_trait!(RegistroC509);

impl SpedParser for RegistroC509 {
    type Output = RegistroC509;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C509 possui 3 campos de dados + 2 delimitadores = 5.
        if len != 5 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 5,
                tamanho_encontrado: len,
            });
        }

        let num_proc = fields.get(2).to_arc();
        let ind_proc = fields.get(3).to_arc();

        let reg = RegistroC509 {
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
