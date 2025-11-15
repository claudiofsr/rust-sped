use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct RegistroC481 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: String,
    pub line_number: usize,
    pub cst_pis: Option<String>,         // 2
    pub vl_item: Option<Decimal>,        // 3
    pub vl_bc_pis: Option<Decimal>,      // 4
    pub aliq_pis: Option<Decimal>,       // 5
    pub quant_bc_pis: Option<String>,    // 6 (Pode ser Decimal dependendo do formato SPED)
    pub aliq_pis_quant: Option<Decimal>, // 7
    pub vl_pis: Option<Decimal>,         // 8
    pub cod_item: Option<String>,        // 9
    pub cod_cta: Option<String>,         // 10
}

impl_sped_record_trait!(RegistroC481);

impl SpedParser for RegistroC481 {
    type Output = RegistroC481;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        if len != 12 {
            return Err(EFDError::InvalidLength {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 12,
                tamanho_encontrado: len,
            });
        }

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let cst_pis = fields.get(2).to_optional_string();
        let vl_item = get_decimal_field(3, "VL_ITEM")?;
        let vl_bc_pis = get_decimal_field(4, "VL_BC_PIS")?;
        let aliq_pis = get_decimal_field(5, "ALIQ_PIS")?;
        let quant_bc_pis = fields.get(6).to_optional_string();
        let aliq_pis_quant = get_decimal_field(7, "ALIQ_PIS_QUANT")?;
        let vl_pis = get_decimal_field(8, "VL_PIS")?;
        let cod_item = fields.get(9).to_optional_string();
        let cod_cta = fields.get(10).to_optional_string();

        let reg = RegistroC481 {
            nivel: 5,
            bloco: 'C',
            registro,
            line_number,
            cst_pis,
            vl_item,
            vl_bc_pis,
            aliq_pis,
            quant_bc_pis,
            aliq_pis_quant,
            vl_pis,
            cod_item,
            cod_cta,
        };

        Ok(reg)
    }
}
