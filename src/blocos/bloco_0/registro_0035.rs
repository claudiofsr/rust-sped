use crate::{EFDError, EFDResult, SpedParser, ToOptionalString, impl_sped_record_trait};
use std::path::Path;

const EXPECTED_FIELDS: usize = 6;

#[derive(Debug, Clone)]
pub struct Registro0035 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_scp: Option<String>,  // 2
    pub desc_scp: Option<String>, // 3
    pub inf_comp: Option<String>, // 4
}

impl_sped_record_trait!(Registro0035);

impl SpedParser for Registro0035 {
    type Output = Registro0035;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro 0035 possui 4 campos de dados + 2 delimitadores = 6.
        if len != EXPECTED_FIELDS {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: EXPECTED_FIELDS,
                tamanho_encontrado: len,
            });
        }

        let cod_scp = fields.get(2).to_optional_string();
        let desc_scp = fields.get(3).to_optional_string();
        let inf_comp = fields.get(4).to_optional_string();

        let reg = Registro0035 {
            nivel: 2,
            bloco: '0',
            registro,
            line_number,
            cod_scp,
            desc_scp,
            inf_comp,
        };

        Ok(reg)
    }
}
