use crate::{EFDError, EFDResult, SpedParser, StringParser, impl_sped_record_trait};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "C010";

#[derive(Debug, Clone)]
pub struct RegistroC010 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cnpj: Option<Arc<str>>,      // 2
    pub ind_escri: Option<Arc<str>>, // 3
}

impl_sped_record_trait!(RegistroC010);

impl SpedParser for RegistroC010 {
    type Output = RegistroC010;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C010 possui 3 campos de dados + 2 delimitadores = 5.
        if len != 5 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 5,
                tamanho_encontrado: len,
            });
        }

        let cnpj = fields.get(2).to_arc();
        let ind_escri = fields.get(3).to_arc();

        let reg = RegistroC010 {
            nivel: 2,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cnpj,
            ind_escri,
        };

        Ok(reg)
    }
}
