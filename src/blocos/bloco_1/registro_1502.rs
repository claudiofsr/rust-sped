use crate::{EFDError, EFDResult, ResultExt, SpedParser, ToDecimal, impl_reg_methods};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "1502";

#[derive(Debug, Clone)]
pub struct Registro1502 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub vl_cred_cofins_trib_mi: Option<Decimal>, // 2
    pub vl_cred_cofins_nt_mi: Option<Decimal>,   // 3
    pub vl_cred_cofins_exp: Option<Decimal>,     // 4
}

impl_reg_methods!(Registro1502);

impl SpedParser for Registro1502 {
    type Output = Registro1502;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro 1502 possui 4 campos de dados + 2 delimitadores = 6.
        if len != 6 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 6,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let vl_cred_cofins_trib_mi = get_decimal(2, "VL_CRED_COFINS_TRIB_MI")?;
        let vl_cred_cofins_nt_mi = get_decimal(3, "VL_CRED_COFINS_NT_MI")?;
        let vl_cred_cofins_exp = get_decimal(4, "VL_CRED_COFINS_EXP")?;

        let reg = Registro1502 {
            nivel: 4,
            bloco: '1',
            registro: REGISTRO.into(),
            line_number,
            vl_cred_cofins_trib_mi,
            vl_cred_cofins_nt_mi,
            vl_cred_cofins_exp,
        };

        Ok(reg)
    }
}
