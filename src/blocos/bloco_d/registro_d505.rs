use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct RegistroD505 {
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
    pub nat_bc_cred: Option<String>,   // 4
    pub vl_bc_cofins: Option<Decimal>, // 5
    pub aliq_cofins: Option<Decimal>,  // 6
    pub vl_cofins: Option<Decimal>,    // 7
    pub cod_cta: Option<String>,       // 8
}

impl_sped_record_trait!(RegistroD505);

impl SpedParser for RegistroD505 {
    type Output = RegistroD505;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro D505 possui 8 campos de dados + 2 delimitadores = 10.
        if len != 10 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 10,
                tamanho_encontrado: len,
            });
        }

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let cst_cofins = fields.get(2).to_optional_string();
        let vl_item = get_decimal_field(3, "VL_ITEM")?;
        let nat_bc_cred = fields.get(4).to_optional_string();
        let vl_bc_cofins = get_decimal_field(5, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal_field(6, "ALIQ_COFINS")?;
        let vl_cofins = get_decimal_field(7, "VL_COFINS")?;
        let cod_cta = fields.get(8).to_optional_string();

        let reg = RegistroD505 {
            nivel: 4,
            bloco: 'D',
            registro,
            line_number,
            cst_cofins,
            vl_item,
            nat_bc_cred,
            vl_bc_cofins,
            aliq_cofins,
            vl_cofins,
            cod_cta,
        };

        Ok(reg)
    }
}
