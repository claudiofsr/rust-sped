use crate::{EFDError, EFDResult, SpedParser, ToDecimal, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct Registro1502 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub vl_cred_cofins_trib_mi: Option<Decimal>, // 2
    pub vl_cred_cofins_nt_mi: Option<Decimal>,   // 3
    pub vl_cred_cofins_exp: Option<Decimal>,     // 4
}

impl_sped_record_trait!(Registro1502);

impl SpedParser for Registro1502 {
    type Output = Registro1502;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro 1502 possui 4 campos de dados + 2 delimitadores = 6.
        if len != 6 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 6,
                tamanho_encontrado: len,
            });
        }

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let vl_cred_cofins_trib_mi = get_decimal_field(2, "VL_CRED_COFINS_TRIB_MI")?;
        let vl_cred_cofins_nt_mi = get_decimal_field(3, "VL_CRED_COFINS_NT_MI")?;
        let vl_cred_cofins_exp = get_decimal_field(4, "VL_CRED_COFINS_EXP")?;

        let reg = Registro1502 {
            nivel: 4,
            bloco: '1',
            registro,
            line_number,
            vl_cred_cofins_trib_mi,
            vl_cred_cofins_nt_mi,
            vl_cred_cofins_exp,
        };

        Ok(reg)
    }
}
