use crate::{EFDError, EFDResult, SpedParser, ToOptionalString, impl_sped_record_trait};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Registro0120 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub mes_refer: Option<String>, // 2
    pub inf_comp: Option<String>,  // 3
}

impl_sped_record_trait!(Registro0120);

impl SpedParser for Registro0120 {
    type Output = Registro0120;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        if len != 5 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 5,
                tamanho_encontrado: len,
            });
        }

        let mes_refer = fields.get(2).to_optional_string();
        let inf_comp = fields.get(3).to_optional_string();

        let reg = Registro0120 {
            nivel: 2,
            bloco: '0',
            registro,
            line_number,
            mes_refer,
            inf_comp,
        };

        Ok(reg)
    }
}
