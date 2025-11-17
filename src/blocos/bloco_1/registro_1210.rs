use crate::{
    EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalNaiveDate, ToOptionalString,
    impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct Registro1210 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cnpj: Option<String>,       // 2
    pub cst_pis: Option<String>,    // 3
    pub cod_part: Option<String>,   // 4
    pub dt_oper: Option<NaiveDate>, // 5
    pub vl_oper: Option<Decimal>,   // 6
    pub vl_bc_pis: Option<Decimal>, // 7
    pub aliq_pis: Option<Decimal>,  // 8
    pub vl_pis: Option<Decimal>,    // 9
    pub cod_cta: Option<String>,    // 10
    pub desc_compl: Option<String>, // 11
}

impl_sped_record_trait!(Registro1210);

impl SpedParser for Registro1210 {
    type Output = Registro1210;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro 1210 possui 11 campos de dados + 2 delimitadores = 13.
        if len != 13 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 13,
                tamanho_encontrado: len,
            });
        }

        // Closures auxiliares
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

        let cnpj = fields.get(2).to_optional_string();
        let cst_pis = fields.get(3).to_optional_string();
        let cod_part = fields.get(4).to_optional_string();
        let dt_oper = get_date_field(5, "DT_OPER")?;
        let vl_oper = get_decimal_field(6, "VL_OPER")?;
        let vl_bc_pis = get_decimal_field(7, "VL_BC_PIS")?;
        let aliq_pis = get_decimal_field(8, "ALIQ_PIS")?;
        let vl_pis = get_decimal_field(9, "VL_PIS")?;
        let cod_cta = fields.get(10).to_optional_string();
        let desc_compl = fields.get(11).to_optional_string();

        let reg = Registro1210 {
            nivel: 3,
            bloco: '1',
            registro,
            line_number,
            cnpj,
            cst_pis,
            cod_part,
            dt_oper,
            vl_oper,
            vl_bc_pis,
            aliq_pis,
            vl_pis,
            cod_cta,
            desc_compl,
        };

        Ok(reg)
    }
}
