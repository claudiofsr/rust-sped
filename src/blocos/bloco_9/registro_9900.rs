use crate::{EFDError, EFDResult, SpedParser, StringParser, impl_reg_methods};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "9900";

#[derive(Debug, Clone)]
pub struct Registro9900 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub reg_blc: Option<Arc<str>>,     // 2
    pub qtd_reg_blc: Option<Arc<str>>, // 3 (Assumindo que pode ser String se for complexo, ou u64)
}

impl_reg_methods!(Registro9900);

impl SpedParser for Registro9900 {
    type Output = Registro9900;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro 9900 possui 3 campos de dados + 2 delimitadores = 5.
        if len != 5 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 5,
                tamanho_encontrado: len,
            });
        }

        let reg_blc = fields.get(2).to_arc();
        let qtd_reg_blc = fields.get(3).to_arc(); // Pode ser convertido para u64 se necessário

        let reg = Registro9900 {
            nivel: 2,
            bloco: '9',
            registro: REGISTRO.into(),
            line_number,
            reg_blc,
            qtd_reg_blc,
        };

        Ok(reg)
    }
}
