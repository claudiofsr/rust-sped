use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "F150";

#[derive(Debug, Clone)]
pub struct RegistroF150 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub nat_bc_cred: Option<String>,     // 2
    pub vl_tot_est: Option<Decimal>,     // 3
    pub est_imp: Option<String>,         // 4
    pub vl_bc_est: Option<Decimal>,      // 5
    pub vl_bc_men_est: Option<Decimal>,  // 6
    pub cst_pis: Option<String>,         // 7
    pub aliq_pis: Option<Decimal>,       // 8
    pub vl_cred_pis: Option<Decimal>,    // 9
    pub cst_cofins: Option<String>,      // 10
    pub aliq_cofins: Option<Decimal>,    // 11
    pub vl_cred_cofins: Option<Decimal>, // 12
    pub desc_est: Option<String>,        // 13
    pub cod_cta: Option<String>,         // 14
}

impl_sped_record_trait!(RegistroF150);

impl SpedParser for RegistroF150 {
    type Output = RegistroF150;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro F150 possui 14 campos de dados + 2 delimitadores = 16.
        if len != 16 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 16,
                tamanho_encontrado: len,
            });
        }

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let nat_bc_cred = fields.get(2).to_optional_string();
        let vl_tot_est = get_decimal_field(3, "VL_TOT_EST")?;
        let est_imp = fields.get(4).to_optional_string();
        let vl_bc_est = get_decimal_field(5, "VL_BC_EST")?;
        let vl_bc_men_est = get_decimal_field(6, "VL_BC_MEN_EST")?;
        let cst_pis = fields.get(7).to_optional_string();
        let aliq_pis = get_decimal_field(8, "ALIQ_PIS")?;
        let vl_cred_pis = get_decimal_field(9, "VL_CRED_PIS")?;
        let cst_cofins = fields.get(10).to_optional_string();
        let aliq_cofins = get_decimal_field(11, "ALIQ_COFINS")?;
        let vl_cred_cofins = get_decimal_field(12, "VL_CRED_COFINS")?;
        let desc_est = fields.get(13).to_optional_string();
        let cod_cta = fields.get(14).to_optional_string();

        let reg = RegistroF150 {
            nivel: 3,
            bloco: 'F',
            registro: REGISTRO.to_string(),
            line_number,
            nat_bc_cred,
            vl_tot_est,
            est_imp,
            vl_bc_est,
            vl_bc_men_est,
            cst_pis,
            aliq_pis,
            vl_cred_pis,
            cst_cofins,
            aliq_cofins,
            vl_cred_cofins,
            desc_est,
            cod_cta,
        };

        Ok(reg)
    }
}
