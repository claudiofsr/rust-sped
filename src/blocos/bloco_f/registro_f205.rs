use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "F205";

#[derive(Debug, Clone)]
pub struct RegistroF205 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub vl_cus_inc_acum_ant: Option<Decimal>,     // 2
    pub vl_cus_inc_per_esc: Option<Decimal>,      // 3
    pub vl_cus_inc_acum: Option<Decimal>,         // 4
    pub vl_exc_bc_cus_inc_acum: Option<Decimal>,  // 5
    pub vl_bc_cus_inc: Option<Decimal>,           // 6
    pub cst_pis: Option<u16>,                     // 7
    pub aliq_pis: Option<Decimal>,                // 8
    pub vl_cred_pis_acum: Option<Decimal>,        // 9
    pub vl_cred_pis_desc_ant: Option<Decimal>,    // 10
    pub vl_cred_pis_desc: Option<Decimal>,        // 11
    pub vl_cred_pis_desc_fut: Option<Decimal>,    // 12
    pub cst_cofins: Option<u16>,                  // 13
    pub aliq_cofins: Option<Decimal>,             // 14
    pub vl_cred_cofins_acum: Option<Decimal>,     // 15
    pub vl_cred_cofins_desc_ant: Option<Decimal>, // 16
    pub vl_cred_cofins_desc: Option<Decimal>,     // 17
    pub vl_cred_cofins_desc_fut: Option<Decimal>, // 18
}

impl_reg_methods!(RegistroF205);

impl SpedParser for RegistroF205 {
    type Output = RegistroF205;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro F205 possui 18 campos de dados + 2 delimitadores = 20.
        if len != 20 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 20,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let vl_cus_inc_acum_ant = get_decimal(2, "VL_CUS_INC_ACUM_ANT")?;
        let vl_cus_inc_per_esc = get_decimal(3, "VL_CUS_INC_PER_ESC")?;
        let vl_cus_inc_acum = get_decimal(4, "VL_CUS_INC_ACUM")?;
        let vl_exc_bc_cus_inc_acum = get_decimal(5, "VL_EXC_BC_CUS_INC_ACUM")?;
        let vl_bc_cus_inc = get_decimal(6, "VL_BC_CUS_INC")?;
        let cst_pis = fields.get(7).parse_opt();
        let aliq_pis = get_decimal(8, "ALIQ_PIS")?;
        let vl_cred_pis_acum = get_decimal(9, "VL_CRED_PIS_ACUM")?;
        let vl_cred_pis_desc_ant = get_decimal(10, "VL_CRED_PIS_DESC_ANT")?;
        let vl_cred_pis_desc = get_decimal(11, "VL_CRED_PIS_DESC")?;
        let vl_cred_pis_desc_fut = get_decimal(12, "VL_CRED_PIS_DESC_FUT")?;
        let cst_cofins = fields.get(13).parse_opt();
        let aliq_cofins = get_decimal(14, "ALIQ_COFINS")?;
        let vl_cred_cofins_acum = get_decimal(15, "VL_CRED_COFINS_ACUM")?;
        let vl_cred_cofins_desc_ant = get_decimal(16, "VL_CRED_COFINS_DESC_ANT")?;
        let vl_cred_cofins_desc = get_decimal(17, "VL_CRED_COFINS_DESC")?;
        let vl_cred_cofins_desc_fut = get_decimal(18, "VL_CRED_COFINS_DESC_FUT")?;

        let reg = RegistroF205 {
            nivel: 4,
            bloco: 'F',
            registro: REGISTRO.into(),
            line_number,
            vl_cus_inc_acum_ant,
            vl_cus_inc_per_esc,
            vl_cus_inc_acum,
            vl_exc_bc_cus_inc_acum,
            vl_bc_cus_inc,
            cst_pis,
            aliq_pis,
            vl_cred_pis_acum,
            vl_cred_pis_desc_ant,
            vl_cred_pis_desc,
            vl_cred_pis_desc_fut,
            cst_cofins,
            aliq_cofins,
            vl_cred_cofins_acum,
            vl_cred_cofins_desc_ant,
            vl_cred_cofins_desc,
            vl_cred_cofins_desc_fut,
        };

        Ok(reg)
    }
}
