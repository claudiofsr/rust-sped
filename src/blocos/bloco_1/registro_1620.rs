use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Registro1620 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub per_apu_cred: Option<String>, // 2
    pub orig_cred: Option<String>,    // 3
    pub cod_cred: Option<String>,     // 4
    pub vl_cred: Option<Decimal>,     // 5
}

impl_sped_record_trait!(Registro1620);

impl SpedParser for Registro1620 {
    type Output = Registro1620;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro 1620 possui 5 campos de dados + 2 delimitadores = 7.
        if len != 7 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 7,
                tamanho_encontrado: len,
            });
        }

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let per_apu_cred = fields.get(2).to_optional_string();
        let orig_cred = fields.get(3).to_optional_string();
        let cod_cred = fields.get(4).to_optional_string();
        let vl_cred = get_decimal_field(5, "VL_CRED")?;

        let reg = Registro1620 {
            nivel: 3,
            bloco: '1',
            registro,
            line_number,
            per_apu_cred,
            orig_cred,
            cod_cred,
            vl_cred,
        };

        Ok(reg)
    }
}
