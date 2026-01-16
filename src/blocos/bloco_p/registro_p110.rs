use crate::{EFDError, EFDResult, SpedParser, StringParser, impl_reg_methods};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "P110";

#[derive(Debug, Clone)]
pub struct RegistroP110 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub num_campo: Option<Arc<str>>, // 2
    pub cod_det: Option<Arc<str>>,   // 3
    pub det_valor: Option<Arc<str>>, // 4
    pub inf_compl: Option<Arc<str>>, // 5
}

impl_reg_methods!(RegistroP110);

impl SpedParser for RegistroP110 {
    type Output = RegistroP110;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro P110 possui 5 campos de dados + 2 delimitadores = 7.
        if len != 7 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 7,
                tamanho_encontrado: len,
            });
        }

        let num_campo = fields.get(2).to_arc();
        let cod_det = fields.get(3).to_arc();
        let det_valor = fields.get(4).to_arc();
        let inf_compl = fields.get(5).to_arc();

        let reg = RegistroP110 {
            nivel: 4,
            bloco: 'P',
            registro: REGISTRO.into(),
            line_number,
            num_campo,
            cod_det,
            det_valor,
            inf_compl,
        };

        Ok(reg)
    }
}
