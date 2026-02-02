use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C385";

#[derive(Debug, Clone)]
pub struct RegistroC385 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cst_cofins: Option<u16>,                // 2
    pub cod_item: Option<CompactString>,        // 3
    pub vl_item: Option<Decimal>,               // 4
    pub vl_bc_cofins: Option<Decimal>,          // 5
    pub aliq_cofins: Option<Decimal>,           // 6
    pub quant_bc_cofins: Option<CompactString>, // 7
    pub aliq_cofins_quant: Option<Decimal>,     // 8
    pub vl_cofins: Option<Decimal>,             // 9
    pub cod_cta: Option<CompactString>,         // 10
}

impl_reg_methods!(RegistroC385);

impl SpedParser for RegistroC385 {
    type Output = RegistroC385;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C385 possui 9 campos de dados + 2 delimitadores = 11.
        if len != 12 {
            // 10 fields + REG + |
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
        let cod_item = fields.get(3).to_compact_string();
        let vl_item = get_decimal(4, "VL_ITEM")?;
        let vl_bc_cofins = get_decimal(5, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(6, "ALIQ_COFINS")?;
        let quant_bc_cofins = fields.get(7).to_compact_string();
        let aliq_cofins_quant = get_decimal(8, "ALIQ_COFINS_QUANT")?;
        let vl_cofins = get_decimal(9, "VL_COFINS")?;
        let cod_cta = fields.get(10).to_compact_string();

        let reg = RegistroC385 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cst_cofins,
            cod_item,
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
