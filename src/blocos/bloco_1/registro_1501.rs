use crate::{
    EFDError, EFDResult, SpedParser, StringParser, ToDecimal, ToNaiveDate, impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "1501";

#[derive(Debug, Clone)]
pub struct Registro1501 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_part: Option<Arc<str>>,      // 2
    pub cod_item: Option<Arc<str>>,      // 3
    pub cod_mod: Option<Arc<str>>,       // 4
    pub ser: Option<Arc<str>>,           // 5
    pub sub_ser: Option<Arc<str>>,       // 6
    pub num_doc: Option<Arc<str>>,       // 7
    pub dt_oper: Option<NaiveDate>,      // 8
    pub chv_nfe: Option<Arc<str>>,       // 9
    pub vl_oper: Option<Decimal>,        // 10
    pub cfop: Option<u16>,               // 11
    pub nat_bc_cred: Option<Arc<str>>,   // 12
    pub ind_orig_cred: Option<Arc<str>>, // 13
    pub cst_cofins: Option<u16>,         // 14
    pub vl_bc_cofins: Option<Decimal>,   // 15
    pub aliq_cofins: Option<Decimal>,    // 16
    pub vl_cofins: Option<Decimal>,      // 17
    pub cod_cta: Option<Arc<str>>,       // 18
    pub cod_ccus: Option<Arc<str>>,      // 19
    pub desc_compl: Option<Arc<str>>,    // 20
    pub per_escrit: Option<Arc<str>>,    // 21
    pub cnpj: Option<Arc<str>>,          // 22
}

impl_sped_record_trait!(Registro1501);

impl SpedParser for Registro1501 {
    type Output = Registro1501;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro 1501 possui 22 campos de dados + 2 delimitadores = 24.
        if len != 24 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 24,
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

        let cod_part = fields.get(2).to_arc();
        let cod_item = fields.get(3).to_arc();
        let cod_mod = fields.get(4).to_arc();
        let ser = fields.get(5).to_arc();
        let sub_ser = fields.get(6).to_arc();
        let num_doc = fields.get(7).to_arc();
        let dt_oper = get_date_field(8, "DT_OPER")?;
        let chv_nfe = fields.get(9).to_arc();
        let vl_oper = get_decimal_field(10, "VL_OPER")?;
        let cfop = fields.get(11).parse_opt();
        let nat_bc_cred = fields.get(12).to_arc();
        let ind_orig_cred = fields.get(13).to_arc();
        let cst_cofins = fields.get(14).parse_opt();
        let vl_bc_cofins = get_decimal_field(15, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal_field(16, "ALIQ_COFINS")?;
        let vl_cofins = get_decimal_field(17, "VL_COFINS")?;
        let cod_cta = fields.get(18).to_arc();
        let cod_ccus = fields.get(19).to_arc();
        let desc_compl = fields.get(20).to_arc();
        let per_escrit = fields.get(21).to_arc();
        let cnpj = fields.get(22).to_arc();

        let reg = Registro1501 {
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
            cst_cofins,
            vl_bc_cofins,
            aliq_cofins,
            vl_cofins,
            cod_cta,
            cod_ccus,
            desc_compl,
            per_escrit,
            cnpj,
        };

        Ok(reg)
    }
}
