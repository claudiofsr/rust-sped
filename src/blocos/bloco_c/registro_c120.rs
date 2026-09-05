// registro_c120.rs
use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C120";

#[derive(Debug, Clone)]
pub struct RegistroC120 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: CompactString,
    pub line_number: usize,
    pub cod_doc_imp: Option<CompactString>, // 2
    pub num_doc_imp: Option<CompactString>, // 3
    pub vl_pis_imp: Option<Decimal>,        // 4
    pub vl_cofins_imp: Option<Decimal>,     // 5
    pub num_acdraw: Option<CompactString>,  // 6
}

impl_reg_methods!(RegistroC120);

impl SpedParser for RegistroC120 {
    type Output = RegistroC120;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != 8 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 8,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cod_doc_imp = fields.get(2).to_compact_string();
        let num_doc_imp = fields.get(3).to_compact_string();
        let vl_pis_imp = get_decimal(4, "VL_PIS_IMP")?;
        let vl_cofins_imp = get_decimal(5, "VL_COFINS_IMP")?;
        let num_acdraw = fields.get(6).to_compact_string();

        let reg = RegistroC120 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cod_doc_imp,
            num_doc_imp,
            vl_pis_imp,
            vl_cofins_imp,
            num_acdraw,
        };

        Ok(reg)
    }
}
