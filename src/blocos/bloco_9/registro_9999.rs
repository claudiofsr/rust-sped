use crate::{EFDError, EFDResult, SpedParser, ToOptionalInteger, impl_sped_record_trait};
use std::path::Path;

#[derive(Debug)]
pub struct Registro9999 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub qtd_lin: Option<u64>, // 2
}

impl_sped_record_trait!(Registro9999);

impl SpedParser for Registro9999 {
    type Output = Registro9999;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro 9999 possui 2 campos de dados + 2 delimitadores = 4.
        if len != 4 {
            return Err(EFDError::InvalidLength {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 4,
                tamanho_encontrado: len,
            });
        }

        // --- Closure auxiliar para campos u64 ---
        let get_integer_field = |idx: usize, field_name: &str| {
            fields
                .get(idx) // fields.get(idx) retorna Option<&&str>
                .to_optional_integer(file_path.to_path_buf(), line_number, field_name)
        };

        let qtd_lin = get_integer_field(2, "QTD_LIN")?; // O '?' propagará o erro se houver

        let reg = Registro9999 {
            nivel: 0,
            bloco: '9',
            registro,
            line_number,
            qtd_lin,
        };

        Ok(reg)
    }
}
