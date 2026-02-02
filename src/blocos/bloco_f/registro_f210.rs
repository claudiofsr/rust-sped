use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "F210";

#[derive(Debug, Clone)]
pub struct RegistroF210 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub vl_cus_orc: Option<Decimal>,          // 2
    pub vl_exc: Option<Decimal>,              // 3
    pub vl_cus_orc_aju: Option<Decimal>,      // 4
    pub vl_bc_cred: Option<Decimal>,          // 5
    pub cst_pis: Option<u16>,                 // 6
    pub aliq_pis: Option<Decimal>,            // 7
    pub vl_cred_pis_util: Option<Decimal>,    // 8
    pub cst_cofins: Option<u16>,              // 9
    pub aliq_cofins: Option<Decimal>,         // 10
    pub vl_cred_cofins_util: Option<Decimal>, // 11
}

impl_reg_methods!(RegistroF210);

impl SpedParser for RegistroF210 {
    type Output = RegistroF210;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro F210 possui 11 campos de dados + 2 delimitadores = 13.
        if len != 13 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 13,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let vl_cus_orc = get_decimal(2, "VL_CUS_ORC")?;
        let vl_exc = get_decimal(3, "VL_EXC")?;
        let vl_cus_orc_aju = get_decimal(4, "VL_CUS_ORC_AJU")?;
        let vl_bc_cred = get_decimal(5, "VL_BC_CRED")?;
        let cst_pis = fields.get(6).parse_opt();
        let aliq_pis = get_decimal(7, "ALIQ_PIS")?;
        let vl_cred_pis_util = get_decimal(8, "VL_CRED_PIS_UTIL")?;
        let cst_cofins = fields.get(9).parse_opt();
        let aliq_cofins = get_decimal(10, "ALIQ_COFINS")?;
        let vl_cred_cofins_util = get_decimal(11, "VL_CRED_COFINS_UTIL")?;

        let reg = RegistroF210 {
            nivel: 4,
            bloco: 'F',
            registro: REGISTRO.into(),
            line_number,
            vl_cus_orc,
            vl_exc,
            vl_cus_orc_aju,
            vl_bc_cred,
            cst_pis,
            aliq_pis,
            vl_cred_pis_util,
            cst_cofins,
            aliq_cofins,
            vl_cred_cofins_util,
        };

        Ok(reg)
    }
}
