use crate::{EFDError, EFDResult, SpedParser, ToOptionalString, impl_sped_record_trait};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct RegistroC010 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cnpj: Option<String>,      // 2
    pub ind_escri: Option<String>, // 3
}

impl_sped_record_trait!(RegistroC010);

impl SpedParser for RegistroC010 {
    type Output = RegistroC010;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro C010 possui 3 campos de dados + 2 delimitadores = 5.
        if len != 5 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 5,
                tamanho_encontrado: len,
            });
        }

        let cnpj = fields.get(2).to_optional_string();
        let ind_escri = fields.get(3).to_optional_string();

        let reg = RegistroC010 {
            nivel: 2,
            bloco: 'C',
            registro,
            line_number,
            cnpj,
            ind_escri,
        };

        Ok(reg)
    }
}
