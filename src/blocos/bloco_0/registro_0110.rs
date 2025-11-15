use crate::{EFDError, EFDResult, SpedParser, ToOptionalString, impl_sped_record_trait};
use std::path::Path;

#[derive(Debug)]
pub struct Registro0110 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_inc_trib: Option<String>,  // 2
    pub ind_apro_cred: Option<String>, // 3
    pub cod_tipo_cont: Option<String>, // 4
    pub ind_reg_cum: Option<String>,   // 5
}

impl_sped_record_trait!(Registro0110);

impl SpedParser for Registro0110 {
    type Output = Registro0110;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        if len != 7 {
            return Err(EFDError::InvalidLength {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 7,
                tamanho_encontrado: len,
            });
        }

        let cod_inc_trib = fields.get(2).to_optional_string();
        let ind_apro_cred = fields.get(3).to_optional_string();
        let cod_tipo_cont = fields.get(4).to_optional_string();
        let ind_reg_cum = fields.get(5).to_optional_string();

        let reg = Registro0110 {
            nivel: 2,
            bloco: '0',
            registro,
            line_number,
            cod_inc_trib,
            ind_apro_cred,
            cod_tipo_cont,
            ind_reg_cum,
        };

        Ok(reg)
    }
}
