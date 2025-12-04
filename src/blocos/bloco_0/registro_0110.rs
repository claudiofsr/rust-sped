use crate::{EFDError, EFDResult, SpedParser, StringParser, impl_sped_record_trait};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "0110";

#[derive(Debug, Clone)]
pub struct Registro0110 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_inc_trib: Option<Arc<str>>,  // 2
    pub ind_apro_cred: Option<Arc<str>>, // 3
    pub cod_tipo_cont: Option<Arc<str>>, // 4
    pub ind_reg_cum: Option<Arc<str>>,   // 5
}

impl_sped_record_trait!(Registro0110);

impl SpedParser for Registro0110 {
    type Output = Registro0110;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != 7 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 7,
                tamanho_encontrado: len,
            });
        }

        let cod_inc_trib = fields.get(2).to_arc();
        let ind_apro_cred = fields.get(3).to_arc();
        let cod_tipo_cont = fields.get(4).to_arc();
        let ind_reg_cum = fields.get(5).to_arc();

        let reg = Registro0110 {
            nivel: 2,
            bloco: '0',
            registro: REGISTRO.to_string(),
            line_number,
            cod_inc_trib,
            ind_apro_cred,
            cod_tipo_cont,
            ind_reg_cum,
        };

        Ok(reg)
    }
}
