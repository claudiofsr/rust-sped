use crate::{
    CodigoSituacaoTributaria, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal,
    impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C481";

#[derive(Debug, Clone)]
pub struct RegistroC481 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: CompactString,
    pub line_number: usize,
    pub cst_pis: Option<CodigoSituacaoTributaria>, // 2
    pub vl_item: Option<Decimal>,                  // 3
    pub vl_bc_pis: Option<Decimal>,                // 4
    pub aliq_pis: Option<Decimal>,                 // 5
    pub quant_bc_pis: Option<CompactString>, // 6 (Pode ser Decimal dependendo do formato SPED)
    pub aliq_pis_quant: Option<Decimal>,     // 7
    pub vl_pis: Option<Decimal>,             // 8
    pub cod_item: Option<CompactString>,     // 9
    pub cod_cta: Option<CompactString>,      // 10
}

impl_reg_methods!(RegistroC481);

impl SpedParser for RegistroC481 {
    type Output = RegistroC481;

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

        let cst_pis = fields
            .get(2)
            .to_efd_field(file_path, line_number, "CST_PIS")?;
        let vl_item = get_decimal(3, "VL_ITEM")?;
        let vl_bc_pis = get_decimal(4, "VL_BC_PIS")?;
        let aliq_pis = get_decimal(5, "ALIQ_PIS")?;
        let quant_bc_pis = fields.get(6).to_compact_string();
        let aliq_pis_quant = get_decimal(7, "ALIQ_PIS_QUANT")?;
        let vl_pis = get_decimal(8, "VL_PIS")?;
        let cod_item = fields.get(9).to_compact_string();
        let cod_cta = fields.get(10).to_compact_string();

        let reg = RegistroC481 {
            nivel: 5,
            bloco: 'C',
            registro: REGISTRO.into(),
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
