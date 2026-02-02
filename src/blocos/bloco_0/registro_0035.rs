use crate::{EFDError, EFDResult, ResultExt, SpedParser, StringParser, impl_reg_methods};
use std::{path::Path, sync::Arc};

const EXPECTED_FIELDS: usize = 6;
const REGISTRO: &str = "0035";

#[derive(Debug, Clone)]
pub struct Registro0035 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_scp: Option<Arc<str>>,  // 2
    pub desc_scp: Option<Arc<str>>, // 3
    pub inf_comp: Option<Arc<str>>, // 4
}

impl_reg_methods!(Registro0035);

impl SpedParser for Registro0035 {
    type Output = Registro0035;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro 0035 possui 4 campos de dados + 2 delimitadores = 6.
        if len != EXPECTED_FIELDS {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: EXPECTED_FIELDS,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let cod_scp = fields.get(2).to_arc();
        let desc_scp = fields.get(3).to_arc();
        let inf_comp = fields.get(4).to_arc();

        let reg = Registro0035 {
            nivel: 2,
            bloco: '0',
            registro: REGISTRO.into(),
            line_number,
            cod_scp,
            desc_scp,
            inf_comp,
        };

        Ok(reg)
    }
}
