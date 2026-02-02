use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C491";

#[derive(Debug, Clone)]
pub struct RegistroC491 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: CompactString,
    pub line_number: usize,
    pub cod_item: Option<CompactString>,     // 2
    pub cst_pis: Option<u16>,                // 3
    pub cfop: Option<u16>,                   // 4
    pub vl_item: Option<Decimal>,            // 5
    pub vl_bc_pis: Option<Decimal>,          // 6
    pub aliq_pis: Option<Decimal>,           // 7
    pub quant_bc_pis: Option<CompactString>, // 8 (Pode ser Decimal dependendo do formato SPED)
    pub aliq_pis_quant: Option<Decimal>,     // 9
    pub vl_pis: Option<Decimal>,             // 10
    pub cod_cta: Option<CompactString>,      // 11
}

impl_reg_methods!(RegistroC491);

impl SpedParser for RegistroC491 {
    type Output = RegistroC491;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

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

        let cod_item = fields.get(2).to_compact_string();
        let cst_pis = fields.get(3).parse_opt();
        let cfop = fields.get(4).parse_opt();
        let vl_item = get_decimal(5, "VL_ITEM")?;
        let vl_bc_pis = get_decimal(6, "VL_BC_PIS")?;
        let aliq_pis = get_decimal(7, "ALIQ_PIS")?;
        let quant_bc_pis = fields.get(8).to_compact_string();
        let aliq_pis_quant = get_decimal(9, "ALIQ_PIS_QUANT")?;
        let vl_pis = get_decimal(10, "VL_PIS")?;
        let cod_cta = fields.get(11).to_compact_string();

        let reg = RegistroC491 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cod_item,
            cst_pis,
            cfop,
            vl_item,
            vl_bc_pis,
            aliq_pis,
            quant_bc_pis,
            aliq_pis_quant,
            vl_pis,
            cod_cta,
        };

        Ok(reg)
    }
}
