use crate::{EFDError, EFDResult, SpedParser, StringParser, impl_sped_record_trait};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "A110";

#[derive(Debug, Clone)]
pub struct RegistroA110 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_inf: Option<Arc<str>>,   // 2
    pub txt_compl: Option<Arc<str>>, // 3
}

impl_sped_record_trait!(RegistroA110);

impl SpedParser for RegistroA110 {
    type Output = RegistroA110;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro A110 possui 3 campos de dados + 2 delimitadores = 5.
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
        let txt_compl = fields.get(3).to_arc();

        let reg = RegistroA110 {
            nivel: 4,
            bloco: 'A',
            registro: REGISTRO.into(),
            line_number,
            cod_inf,
            txt_compl,
        };

        Ok(reg)
    }
}
