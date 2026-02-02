use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C181";

#[derive(Debug, Clone)]
pub struct RegistroC181 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cst_pis: Option<u16>,                // 2
    pub cfop: Option<u16>,                   // 3
    pub vl_item: Option<Decimal>,            // 4
    pub vl_desc: Option<Decimal>,            // 5
    pub vl_bc_pis: Option<Decimal>,          // 6
    pub aliq_pis: Option<Decimal>,           // 7
    pub quant_bc_pis: Option<CompactString>, // 8
    pub aliq_pis_quant: Option<Decimal>,     // 9
    pub vl_pis: Option<Decimal>,             // 10
    pub cod_cta: Option<CompactString>,      // 11
}

impl_reg_methods!(RegistroC181);

impl SpedParser for RegistroC181 {
    type Output = RegistroC181;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C181 possui 11 campos de dados + 2 delimitadores = 13.
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

        let cst_pis = fields.get(2).parse_opt();
        let cfop = fields.get(3).parse_opt();
        let vl_item = get_decimal(4, "VL_ITEM")?;
        let vl_desc = get_decimal(5, "VL_DESC")?;
        let vl_bc_pis = get_decimal(6, "VL_BC_PIS")?;
        let aliq_pis = get_decimal(7, "ALIQ_PIS")?;
        let quant_bc_pis = fields.get(8).to_compact_string();
        let aliq_pis_quant = get_decimal(9, "ALIQ_PIS_QUANT")?;
        let vl_pis = get_decimal(10, "VL_PIS")?;
        let cod_cta = fields.get(11).to_compact_string();

        let reg = RegistroC181 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cst_pis,
            cfop,
            vl_item,
            vl_desc,
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
