use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C605";

#[derive(Debug, Clone)]
pub struct RegistroC605 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cst_cofins: Option<u16>,        // 2
    pub vl_item: Option<Decimal>,       // 3
    pub vl_bc_cofins: Option<Decimal>,  // 4
    pub aliq_cofins: Option<Decimal>,   // 5
    pub vl_cofins: Option<Decimal>,     // 6
    pub cod_cta: Option<CompactString>, // 7
}

impl_reg_methods!(RegistroC605);

impl SpedParser for RegistroC605 {
    type Output = RegistroC605;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C605 possui 7 campos de dados + 2 delimitadores = 9.
        if len != 9 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 9,
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
        let vl_cofins = get_decimal(6, "VL_COFINS")?;
        let cod_cta = fields.get(7).to_compact_string();

        let reg = RegistroC605 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cst_cofins,
            vl_item,
            vl_bc_cofins,
            aliq_cofins,
            vl_cofins,
            cod_cta,
        };

        Ok(reg)
    }
}
