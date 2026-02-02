use crate::{EFDError, EFDResult, ResultExt, SpedParser, ToOptionalInteger, impl_reg_methods};
use compact_str::CompactString;
use std::path::Path;

const REGISTRO: &str = "F990";

#[derive(Debug, Clone)]
pub struct RegistroF990 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub qtd_lin_f: Option<u64>, // 2
}

impl_reg_methods!(RegistroF990);

impl SpedParser for RegistroF990 {
    type Output = RegistroF990;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro F990 possui 2 campos de dados + 2 delimitadores = 4.
        if len != 4 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 4,
                tamanho_encontrado: len,
            })
            .loc();
        }

        // --- Closure auxiliar para campos u64 ---
        let get_integer = |idx: usize, field_name: &str| {
            fields
                .get(idx) // fields.get(idx) retorna Option<&&str>
                .to_optional_integer(file_path, line_number, field_name)
        };

        let qtd_lin_f = get_integer(2, "QTD_LIN_F")?; // O '?' propagará o erro se houver

        let reg = RegistroF990 {
            nivel: 1,
            bloco: 'F',
            registro: REGISTRO.into(),
            line_number,
            qtd_lin_f,
        };

        Ok(reg)
    }
}
