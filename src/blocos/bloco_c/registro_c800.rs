use crate::{
    EFDError, EFDResult, SpedParser, ToDecimal, ToNaiveDate, ToOptionalString,
    impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C800";

#[derive(Debug, Clone)]
pub struct RegistroC800 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_mod: Option<String>,       // 2
    pub cod_sit: Option<String>,       // 3
    pub num_cfe: Option<String>,       // 4
    pub dt_doc: Option<NaiveDate>,     // 5
    pub vl_cfe: Option<Decimal>,       // 6
    pub vl_pis: Option<Decimal>,       // 7
    pub vl_cofins: Option<Decimal>,    // 8
    pub cnpj_cpf: Option<String>,      // 9
    pub nr_sat: Option<String>,        // 10
    pub chv_cfe: Option<String>,       // 11
    pub vl_desc: Option<Decimal>,      // 12
    pub vl_merc: Option<Decimal>,      // 13
    pub vl_out_da: Option<Decimal>,    // 14
    pub vl_icms: Option<Decimal>,      // 15
    pub vl_pis_st: Option<Decimal>,    // 16
    pub vl_cofins_st: Option<Decimal>, // 17
}

impl_sped_record_trait!(RegistroC800);

impl SpedParser for RegistroC800 {
    type Output = RegistroC800;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C800 possui 17 campos de dados + 2 delimitadores = 19.
        if len != 19 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 19,
                tamanho_encontrado: len,
            });
        }

        let get_date_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cod_mod = fields.get(2).to_optional_string();
        let cod_sit = fields.get(3).to_optional_string();
        let num_cfe = fields.get(4).to_optional_string();
        let dt_doc = get_date_field(5, "DT_DOC")?;
        let vl_cfe = get_decimal_field(6, "VL_CFE")?;
        let vl_pis = get_decimal_field(7, "VL_PIS")?;
        let vl_cofins = get_decimal_field(8, "VL_COFINS")?;
        let cnpj_cpf = fields.get(9).to_optional_string();
        let nr_sat = fields.get(10).to_optional_string();
        let chv_cfe = fields.get(11).to_optional_string();
        let vl_desc = get_decimal_field(12, "VL_DESC")?;
        let vl_merc = get_decimal_field(13, "VL_MERC")?;
        let vl_out_da = get_decimal_field(14, "VL_OUT_DA")?;
        let vl_icms = get_decimal_field(15, "VL_ICMS")?;
        let vl_pis_st = get_decimal_field(16, "VL_PIS_ST")?;
        let vl_cofins_st = get_decimal_field(17, "VL_COFINS_ST")?;

        let reg = RegistroC800 {
            nivel: 3,
            bloco: 'C',
            registro: REGISTRO.to_string(),
            line_number,
            cod_mod,
            cod_sit,
            num_cfe,
            dt_doc,
            vl_cfe,
            vl_pis,
            vl_cofins,
            cnpj_cpf,
            nr_sat,
            chv_cfe,
            vl_desc,
            vl_merc,
            vl_out_da,
            vl_icms,
            vl_pis_st,
            vl_cofins_st,
        };

        Ok(reg)
    }
}
