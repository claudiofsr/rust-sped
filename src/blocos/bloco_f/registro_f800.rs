use crate::{
    EFDError, EFDResult, SpedParser, ToDecimal, ToNaiveDate, ToOptionalString,
    impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct RegistroF800 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_nat_even: Option<String>,    // 2
    pub dt_even: Option<NaiveDate>,      // 3
    pub cnpj_suced: Option<String>,      // 4
    pub pa_cont_cred: Option<String>,    // 5
    pub cod_cred: Option<String>,        // 6
    pub vl_cred_pis: Option<Decimal>,    // 7
    pub vl_cred_cofins: Option<Decimal>, // 8
    pub per_cred_cis: Option<String>,    // 9
}

impl_sped_record_trait!(RegistroF800);

impl SpedParser for RegistroF800 {
    type Output = RegistroF800;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro F800 possui 9 campos de dados + 2 delimitadores = 11.
        if len != 11 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 11,
                tamanho_encontrado: len,
            });
        }

        let get_date_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path.to_path_buf(), line_number, field_name)
        };

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let ind_nat_even = fields.get(2).to_optional_string();
        let dt_even = get_date_field(3, "DT_EVEN")?;
        let cnpj_suced = fields.get(4).to_optional_string();
        let pa_cont_cred = fields.get(5).to_optional_string();
        let cod_cred = fields.get(6).to_optional_string();
        let vl_cred_pis = get_decimal_field(7, "VL_CRED_PIS")?;
        let vl_cred_cofins = get_decimal_field(8, "VL_CRED_COFINS")?;
        let per_cred_cis = fields.get(9).to_optional_string();

        let reg = RegistroF800 {
            nivel: 3,
            bloco: 'F',
            registro,
            line_number,
            ind_nat_even,
            dt_even,
            cnpj_suced,
            pa_cont_cred,
            cod_cred,
            vl_cred_pis,
            vl_cred_cofins,
            per_cred_cis,
        };

        Ok(reg)
    }
}
