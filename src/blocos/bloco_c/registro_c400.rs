use crate::{EFDError, EFDResult, SpedParser, StringParser, impl_sped_record_trait};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "C400";

#[derive(Debug, Clone)]
pub struct RegistroC400 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_mod: Option<Arc<str>>, // 2
    pub ecf_mod: Option<Arc<str>>, // 3
    pub ecf_fab: Option<Arc<str>>, // 4
    pub ecf_cx: Option<Arc<str>>,  // 5
}

impl_sped_record_trait!(RegistroC400);

impl SpedParser for RegistroC400 {
    type Output = RegistroC400;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C400 possui 5 campos de dados + 2 delimitadores = 7.
        if len != 7 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 7,
                tamanho_encontrado: len,
            });
        }

        let cod_mod = fields.get(2).to_arc();
        let ecf_mod = fields.get(3).to_arc();
        let ecf_fab = fields.get(4).to_arc();
        let ecf_cx = fields.get(5).to_arc();

        let reg = RegistroC400 {
            nivel: 3,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cod_mod,
            ecf_mod,
            ecf_fab,
            ecf_cx,
        };

        Ok(reg)
    }
}
