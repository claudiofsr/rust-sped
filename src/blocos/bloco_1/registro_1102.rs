use crate::{EFDError, EFDResult, SpedParser, ToDecimal, impl_reg_methods};
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "1102";

#[derive(Debug, Clone)]
pub struct Registro1102 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub vl_cred_pis_trib_mi: Option<Decimal>, // 2
    pub vl_cred_pis_nt_mi: Option<Decimal>,   // 3
    pub vl_cred_pis_exp: Option<Decimal>,     // 4
}

impl_reg_methods!(Registro1102);

impl SpedParser for Registro1102 {
    type Output = Registro1102;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro 1102 possui 3 campos de dados + 2 delimitadores = 5.
        if len != 5 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 5,
                tamanho_encontrado: len,
            });
        }

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let vl_cred_pis_trib_mi = get_decimal(2, "VL_CRED_PIS_TRIB_MI")?;
        let vl_cred_pis_nt_mi = get_decimal(3, "VL_CRED_PIS_NT_MI")?;
        let vl_cred_pis_exp = get_decimal(4, "VL_CRED_PIS_EXP")?;

        let reg = Registro1102 {
            nivel: 4,
            bloco: '1',
            registro: REGISTRO.into(),
            line_number,
            vl_cred_pis_trib_mi,
            vl_cred_pis_nt_mi,
            vl_cred_pis_exp,
        };

        Ok(reg)
    }
}
