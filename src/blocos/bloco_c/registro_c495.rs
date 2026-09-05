use crate::{
    CodigoSituacaoTributaria, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal,
    impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C495";

#[derive(Debug, Clone)]
pub struct RegistroC495 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: CompactString,
    pub line_number: usize,
    pub cod_item: Option<CompactString>,              // 2
    pub cst_cofins: Option<CodigoSituacaoTributaria>, // 3
    pub cfop: Option<u16>,                            // 4
    pub vl_item: Option<Decimal>,                     // 5
    pub vl_bc_cofins: Option<Decimal>,                // 6
    pub aliq_cofins: Option<Decimal>,                 // 7
    pub quant_bc_cofins: Option<CompactString>, // 8 (Pode ser Decimal dependendo do formato SPED)
    pub aliq_cofins_quant: Option<Decimal>,     // 9
    pub vl_cofins: Option<Decimal>,             // 10
    pub cod_cta: Option<CompactString>,         // 11
}

impl_reg_methods!(RegistroC495);

impl SpedParser for RegistroC495 {
    type Output = RegistroC495;

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
        let cst_cofins = fields
            .get(3)
            .to_efd_field(file_path, line_number, "CST_COFINS")?;
        let cfop = fields.get(4).parse_opt();
        let vl_item = get_decimal(5, "VL_ITEM")?;
        let vl_bc_cofins = get_decimal(6, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(7, "ALIQ_COFINS")?;
        let quant_bc_cofins = fields.get(8).to_compact_string();
        let aliq_cofins_quant = get_decimal(9, "ALIQ_COFINS_QUANT")?;
        let vl_cofins = get_decimal(10, "VL_COFINS")?;
        let cod_cta = fields.get(11).to_compact_string();

        let reg = RegistroC495 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.into(),
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
