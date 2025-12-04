use crate::{
    EFDError, EFDResult, SpedParser, StringParser, ToDecimal, ToNaiveDate, ToOptionalString,
    impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "1101";

#[derive(Debug, Clone)]
pub struct Registro1101 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_part: Option<String>,      // 2
    pub cod_item: Option<String>,      // 3
    pub cod_mod: Option<String>,       // 4
    pub ser: Option<String>,           // 5
    pub sub_ser: Option<String>,       // 6
    pub num_doc: Option<String>,       // 7
    pub dt_oper: Option<NaiveDate>,    // 8
    pub chv_nfe: Option<String>,       // 9
    pub vl_oper: Option<Decimal>,      // 10
    pub cfop: Option<u16>,             // 11
    pub nat_bc_cred: Option<String>,   // 12
    pub ind_orig_cred: Option<String>, // 13
    pub cst_pis: Option<u16>,          // 14
    pub vl_bc_pis: Option<Decimal>,    // 15
    pub aliq_pis: Option<Decimal>,     // 16
    pub vl_pis: Option<Decimal>,       // 17
    pub cod_cta: Option<String>,       // 18
    pub cod_ccus: Option<String>,      // 19
    pub desc_compl: Option<String>,    // 20
    pub per_escrit: Option<String>,    // 21
    pub cnpj: Option<String>,          // 22
}

impl_sped_record_trait!(Registro1101);

impl SpedParser for Registro1101 {
    type Output = Registro1101;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro 1101 possui 22 campos de dados + 2 delimitadores = 24.
        if len != 24 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 24,
                tamanho_encontrado: len,
            });
        }

        // Closures auxiliares
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

        let cod_part = fields.get(2).to_optional_string();
        let cod_item = fields.get(3).to_optional_string();
        let cod_mod = fields.get(4).to_optional_string();
        let ser = fields.get(5).to_optional_string();
        let sub_ser = fields.get(6).to_optional_string();
        let num_doc = fields.get(7).to_optional_string();
        let dt_oper = get_date_field(8, "DT_OPER")?;
        let chv_nfe = fields.get(9).to_optional_string();
        let vl_oper = get_decimal_field(10, "VL_OPER")?;
        let cfop = fields.get(11).parse_opt();
        let nat_bc_cred = fields.get(12).to_optional_string();
        let ind_orig_cred = fields.get(13).to_optional_string();
        let cst_pis = fields.get(14).parse_opt();
        let vl_bc_pis = get_decimal_field(15, "VL_BC_PIS")?;
        let aliq_pis = get_decimal_field(16, "ALIQ_PIS")?;
        let vl_pis = get_decimal_field(17, "VL_PIS")?;
        let cod_cta = fields.get(18).to_optional_string();
        let cod_ccus = fields.get(19).to_optional_string();
        let desc_compl = fields.get(20).to_optional_string();
        let per_escrit = fields.get(21).to_optional_string();
        let cnpj = fields.get(22).to_optional_string();

        let reg = Registro1101 {
            nivel: 3,
            bloco: '1',
            registro: REGISTRO.to_string(),
            line_number,
            cod_part,
            cod_item,
            cod_mod,
            ser,
            sub_ser,
            num_doc,
            dt_oper,
            chv_nfe,
            vl_oper,
            cfop,
            nat_bc_cred,
            ind_orig_cred,
            cst_pis,
            vl_bc_pis,
            aliq_pis,
            vl_pis,
            cod_cta,
            cod_ccus,
            desc_compl,
            per_escrit,
            cnpj,
        };

        Ok(reg)
    }
}
