use crate::{EFDError, EFDResult, SpedParser, ToOptionalString, impl_sped_record_trait};
use std::path::Path;

#[derive(Debug)]
pub struct Registro9900 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub reg_blc: Option<String>,     // 2
    pub qtd_reg_blc: Option<String>, // 3 (Assumindo que pode ser String se for complexo, ou u64)
}

impl_sped_record_trait!(Registro9900);

impl SpedParser for Registro9900 {
    type Output = Registro9900;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro 9900 possui 3 campos de dados + 2 delimitadores = 5.
        if len != 5 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 5,
                tamanho_encontrado: len,
            });
        }

        let reg_blc = fields.get(2).to_optional_string();
        let qtd_reg_blc = fields.get(3).to_optional_string(); // Pode ser convertido para u64 se necessário

        let reg = Registro9900 {
            nivel: 2,
            bloco: '9',
            registro,
            line_number,
            reg_blc,
            qtd_reg_blc,
        };

        Ok(reg)
    }
}
