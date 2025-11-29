use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
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
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cst_cofins: Option<String>,    // 2
    pub vl_item: Option<Decimal>,      // 3
    pub vl_bc_cofins: Option<Decimal>, // 4
    pub aliq_cofins: Option<Decimal>,  // 5
    pub vl_cofins: Option<Decimal>,    // 6
    pub cod_cta: Option<String>,       // 7
}

impl_sped_record_trait!(RegistroC605);

impl SpedParser for RegistroC605 {
    type Output = RegistroC605;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C605 possui 7 campos de dados + 2 delimitadores = 9.
        if len != 9 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 9,
                tamanho_encontrado: len,
            });
        }

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cst_cofins = fields.get(2).to_optional_string();
        let vl_item = get_decimal_field(3, "VL_ITEM")?;
        let vl_bc_cofins = get_decimal_field(4, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal_field(5, "ALIQ_COFINS")?;
        let vl_cofins = get_decimal_field(6, "VL_COFINS")?;
        let cod_cta = fields.get(7).to_optional_string();

        let reg = RegistroC605 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.to_string(),
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
