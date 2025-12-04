use crate::{EFDError, EFDResult, SpedParser, StringParser, impl_sped_record_trait};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "I010";

#[derive(Debug, Clone)]
pub struct RegistroI010 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cnpj: Option<Arc<str>>,       // 2
    pub ind_ativ: Option<Arc<str>>,   // 3
    pub info_compl: Option<Arc<str>>, // 4
}

impl_sped_record_trait!(RegistroI010);

impl SpedParser for RegistroI010 {
    type Output = RegistroI010;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro I010 possui 4 campos de dados + 2 delimitadores = 6.
        if len != 6 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 6,
                tamanho_encontrado: len,
            });
        }

        let cnpj = fields.get(2).to_arc();
        let ind_ativ = fields.get(3).to_arc();
        let info_compl = fields.get(4).to_arc();

        let reg = RegistroI010 {
            nivel: 2,
            bloco: 'I',
            registro: REGISTRO.to_string(),
            line_number,
            cnpj,
            ind_ativ,
            info_compl,
        };

        Ok(reg)
    }
}
