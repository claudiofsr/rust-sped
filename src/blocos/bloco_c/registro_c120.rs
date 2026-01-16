// registro_c120.rs
use crate::{EFDError, EFDResult, SpedParser, StringParser, ToDecimal, impl_reg_methods};
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "C120";

#[derive(Debug, Clone)]
pub struct RegistroC120 {
    pub nivel: u16,
    pub bloco: char,
    pub registro: Arc<str>,
    pub line_number: usize,
    pub cod_doc_imp: Option<Arc<str>>,  // 2
    pub num_doc_imp: Option<Arc<str>>,  // 3
    pub vl_pis_imp: Option<Decimal>,    // 4
    pub vl_cofins_imp: Option<Decimal>, // 5
    pub num_acdraw: Option<Arc<str>>,   // 6
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
            });
        }

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cod_doc_imp = fields.get(2).to_arc();
        let num_doc_imp = fields.get(3).to_arc();
        let vl_pis_imp = get_decimal(4, "VL_PIS_IMP")?;
        let vl_cofins_imp = get_decimal(5, "VL_COFINS_IMP")?;
        let num_acdraw = fields.get(6).to_arc();

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
