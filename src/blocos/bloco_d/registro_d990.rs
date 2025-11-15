use crate::{EFDError, EFDResult, SpedParser, ToOptionalInteger, impl_sped_record_trait};
use std::path::Path;

#[derive(Debug)]
pub struct RegistroD990 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub qtd_lin_d: Option<u64>, // 2 (Considerando como String, pode ser u64 se sempre numérico)
}

impl_sped_record_trait!(RegistroD990);

impl SpedParser for RegistroD990 {
    type Output = RegistroD990;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro D990 possui 2 campos de dados + 2 delimitadores = 4.
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

        let qtd_lin_d = get_integer_field(2, "QTD_LIN_D")?; // O '?' propagará o erro se houver        

        let reg = RegistroD990 {
            nivel: 1,
            bloco: 'D',
            registro,
            line_number,
            qtd_lin_d,
        };

        Ok(reg)
    }
}
