use crate::{
    EFDError, EFDResult, SpedParser, ToNaiveDate, ToOptionalString, impl_sped_record_trait,
};
use chrono::NaiveDate;
use std::path::Path;

const REGISTRO: &str = "0500";

#[derive(Debug, Clone)]
pub struct Registro0500 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub dt_alt: Option<NaiveDate>,   // 2
    pub cod_nat_cc: Option<String>,  // 3
    pub ind_cta: Option<String>,     // 4
    pub nivel_conta: Option<String>, // 5 (renomeado para evitar conflito com 'nivel' do struct)
    pub cod_cta: Option<String>,     // 6
    pub nome_cta: Option<String>,    // 7
    pub cod_cta_ref: Option<String>, // 8
    pub cnpj_est: Option<String>,    // 9
}

impl_sped_record_trait!(Registro0500);

impl SpedParser for Registro0500 {
    type Output = Registro0500;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != 11 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 11,
                tamanho_encontrado: len,
            });
        }

        let get_date_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let dt_alt = get_date_field(2, "DT_ALT")?;
        let cod_nat_cc = fields.get(3).to_optional_string();
        let ind_cta = fields.get(4).to_optional_string();
        let nivel_conta = fields.get(5).to_optional_string();
        let cod_cta = fields.get(6).to_optional_string();
        let nome_cta = fields.get(7).to_optional_string();
        let cod_cta_ref = fields.get(8).to_optional_string();
        let cnpj_est = fields.get(9).to_optional_string();

        let reg = Registro0500 {
            nivel: 2,
            bloco: '0',
            registro: REGISTRO.to_string(),
            line_number,
            dt_alt,
            cod_nat_cc,
            ind_cta,
            nivel_conta,
            cod_cta,
            nome_cta,
            cod_cta_ref,
            cnpj_est,
        };

        Ok(reg)
    }
}
