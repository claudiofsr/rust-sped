use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct RegistroC396 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_item: Option<String>,      // 2
    pub vl_item: Option<Decimal>,      // 3
    pub vl_desc: Option<Decimal>,      // 4
    pub nat_bc_cred: Option<String>,   // 5
    pub cst_pis: Option<String>,       // 6
    pub vl_bc_pis: Option<Decimal>,    // 7
    pub aliq_pis: Option<Decimal>,     // 8
    pub vl_pis: Option<Decimal>,       // 9
    pub cst_cofins: Option<String>,    // 10
    pub vl_bc_cofins: Option<Decimal>, // 11
    pub aliq_cofins: Option<Decimal>,  // 12
    pub vl_cofins: Option<Decimal>,    // 13
    pub cod_cta: Option<String>,       // 14
}

impl_sped_record_trait!(RegistroC396);

impl SpedParser for RegistroC396 {
    type Output = RegistroC396;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro C396 possui 14 campos de dados + 2 delimitadores = 16.
        if len != 16 {
            return Err(EFDError::InvalidLength {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 16,
                tamanho_encontrado: len,
            });
        }

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let cod_item = fields.get(2).to_optional_string();
        let vl_item = get_decimal_field(3, "VL_ITEM")?;
        let vl_desc = get_decimal_field(4, "VL_DESC")?;
        let nat_bc_cred = fields.get(5).to_optional_string();
        let cst_pis = fields.get(6).to_optional_string();
        let vl_bc_pis = get_decimal_field(7, "VL_BC_PIS")?;
        let aliq_pis = get_decimal_field(8, "ALIQ_PIS")?;
        let vl_pis = get_decimal_field(9, "VL_PIS")?;
        let cst_cofins = fields.get(10).to_optional_string();
        let vl_bc_cofins = get_decimal_field(11, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal_field(12, "ALIQ_COFINS")?;
        let vl_cofins = get_decimal_field(13, "VL_COFINS")?;
        let cod_cta = fields.get(14).to_optional_string();

        let reg = RegistroC396 {
            nivel: 4,
            bloco: 'C',
            registro,
            line_number,
            cod_item,
            vl_item,
            vl_desc,
            nat_bc_cred,
            cst_pis,
            vl_bc_pis,
            aliq_pis,
            vl_pis,
            cst_cofins,
            vl_bc_cofins,
            aliq_cofins,
            vl_cofins,
            cod_cta,
        };

        Ok(reg)
    }
}
