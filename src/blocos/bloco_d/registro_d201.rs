use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct RegistroD201 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cst_pis: Option<String>,    // 2
    pub vl_item: Option<Decimal>,   // 3
    pub vl_bc_pis: Option<Decimal>, // 4
    pub aliq_pis: Option<Decimal>,  // 5
    pub vl_pis: Option<Decimal>,    // 6
    pub cod_cta: Option<String>,    // 7
}

impl_sped_record_trait!(RegistroD201);

impl SpedParser for RegistroD201 {
    type Output = RegistroD201;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro D201 possui 7 campos de dados + 2 delimitadores = 9.
        if len != 9 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 9,
                tamanho_encontrado: len,
            });
        }

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let cst_pis = fields.get(2).to_optional_string();
        let vl_item = get_decimal_field(3, "VL_ITEM")?;
        let vl_bc_pis = get_decimal_field(4, "VL_BC_PIS")?;
        let aliq_pis = get_decimal_field(5, "ALIQ_PIS")?;
        let vl_pis = get_decimal_field(6, "VL_PIS")?;
        let cod_cta = fields.get(7).to_optional_string();

        let reg = RegistroD201 {
            nivel: 4,
            bloco: 'D',
            registro,
            line_number,
            cst_pis,
            vl_item,
            vl_bc_pis,
            aliq_pis,
            vl_pis,
            cod_cta,
        };

        Ok(reg)
    }
}
