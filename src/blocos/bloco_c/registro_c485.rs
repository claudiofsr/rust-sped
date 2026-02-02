use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C485";

#[derive(Debug, Clone)]
pub struct RegistroC485 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: CompactString,
    pub line_number: usize,
    pub cst_cofins: Option<u16>,                // 2
    pub vl_item: Option<Decimal>,               // 3
    pub vl_bc_cofins: Option<Decimal>,          // 4
    pub aliq_cofins: Option<Decimal>,           // 5
    pub quant_bc_cofins: Option<CompactString>, // 6 (Pode ser Decimal dependendo do formato SPED)
    pub aliq_cofins_quant: Option<Decimal>,     // 7
    pub vl_cofins: Option<Decimal>,             // 8
    pub cod_item: Option<CompactString>,        // 9
    pub cod_cta: Option<CompactString>,         // 10
}

impl_reg_methods!(RegistroC485);

impl SpedParser for RegistroC485 {
    type Output = RegistroC485;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != 12 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 12,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cst_cofins = fields.get(2).parse_opt();
        let vl_item = get_decimal(3, "VL_ITEM")?;
        let vl_bc_cofins = get_decimal(4, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(5, "ALIQ_COFINS")?;
        let quant_bc_cofins = fields.get(6).to_compact_string();
        let aliq_cofins_quant = get_decimal(7, "ALIQ_COFINS_QUANT")?;
        let vl_cofins = get_decimal(8, "VL_COFINS")?;
        let cod_item = fields.get(9).to_compact_string();
        let cod_cta = fields.get(10).to_compact_string();

        let reg = RegistroC485 {
            nivel: 5,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cst_cofins,
            vl_item,
            vl_bc_cofins,
            aliq_cofins,
            quant_bc_cofins,
            aliq_cofins_quant,
            vl_cofins,
            cod_item,
            cod_cta,
        };

        Ok(reg)
    }
}
