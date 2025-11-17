use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct RegistroC495 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: String,
    pub line_number: usize,
    pub cod_item: Option<String>,           // 2
    pub cst_cofins: Option<String>,         // 3
    pub cfop: Option<String>,               // 4
    pub vl_item: Option<Decimal>,           // 5
    pub vl_bc_cofins: Option<Decimal>,      // 6
    pub aliq_cofins: Option<Decimal>,       // 7
    pub quant_bc_cofins: Option<String>,    // 8 (Pode ser Decimal dependendo do formato SPED)
    pub aliq_cofins_quant: Option<Decimal>, // 9
    pub vl_cofins: Option<Decimal>,         // 10
    pub cod_cta: Option<String>,            // 11
}

impl_sped_record_trait!(RegistroC495);

impl SpedParser for RegistroC495 {
    type Output = RegistroC495;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        if len != 13 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 13,
                tamanho_encontrado: len,
            });
        }

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let cod_item = fields.get(2).to_optional_string();
        let cst_cofins = fields.get(3).to_optional_string();
        let cfop = fields.get(4).to_optional_string();
        let vl_item = get_decimal_field(5, "VL_ITEM")?;
        let vl_bc_cofins = get_decimal_field(6, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal_field(7, "ALIQ_COFINS")?;
        let quant_bc_cofins = fields.get(8).to_optional_string();
        let aliq_cofins_quant = get_decimal_field(9, "ALIQ_COFINS_QUANT")?;
        let vl_cofins = get_decimal_field(10, "VL_COFINS")?;
        let cod_cta = fields.get(11).to_optional_string();

        let reg = RegistroC495 {
            nivel: 4,
            bloco: 'C',
            registro,
            line_number,
            cod_item,
            cst_cofins,
            cfop,
            vl_item,
            vl_bc_cofins,
            aliq_cofins,
            quant_bc_cofins,
            aliq_cofins_quant,
            vl_cofins,
            cod_cta,
        };

        Ok(reg)
    }
}
