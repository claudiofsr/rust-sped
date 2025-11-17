use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct RegistroD101 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_nat_frt: Option<String>, // 2
    pub vl_item: Option<Decimal>,    // 3
    pub cst_pis: Option<String>,     // 4
    pub nat_bc_cred: Option<String>, // 5
    pub vl_bc_pis: Option<Decimal>,  // 6
    pub aliq_pis: Option<Decimal>,   // 7
    pub vl_pis: Option<Decimal>,     // 8
    pub cod_cta: Option<String>,     // 9
}

impl_sped_record_trait!(RegistroD101);

impl SpedParser for RegistroD101 {
    type Output = RegistroD101;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro D101 possui 9 campos de dados + 2 delimitadores = 11.
        if len != 11 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 11,
                tamanho_encontrado: len,
            });
        }

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let ind_nat_frt = fields.get(2).to_optional_string();
        let vl_item = get_decimal_field(3, "VL_ITEM")?;
        let cst_pis = fields.get(4).to_optional_string();
        let nat_bc_cred = fields.get(5).to_optional_string();
        let vl_bc_pis = get_decimal_field(6, "VL_BC_PIS")?;
        let aliq_pis = get_decimal_field(7, "ALIQ_PIS")?;
        let vl_pis = get_decimal_field(8, "VL_PIS")?;
        let cod_cta = fields.get(9).to_optional_string();

        let reg = RegistroD101 {
            nivel: 4,
            bloco: 'D',
            registro,
            line_number,
            ind_nat_frt,
            vl_item,
            cst_pis,
            nat_bc_cred,
            vl_bc_pis,
            aliq_pis,
            vl_pis,
            cod_cta,
        };

        Ok(reg)
    }
}
