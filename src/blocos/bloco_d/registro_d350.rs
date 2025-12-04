use crate::{
    EFDError, EFDResult, SpedParser, StringParser, ToDecimal, ToNaiveDate, impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "D350";

#[derive(Debug, Clone)]
pub struct RegistroD350 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_mod: Option<Arc<str>>,          // 2
    pub ecf_mod: Option<Arc<str>>,          // 3
    pub ecf_fab: Option<Arc<str>>,          // 4
    pub dt_doc: Option<NaiveDate>,          // 5
    pub cro: Option<Arc<str>>,              // 6
    pub crz: Option<Arc<str>>,              // 7
    pub num_coo_fin: Option<Arc<str>>,      // 8
    pub gt_fin: Option<Arc<str>>,           // 9
    pub vl_brt: Option<Decimal>,            // 10
    pub cst_pis: Option<u16>,               // 11
    pub vl_bc_pis: Option<Decimal>,         // 12
    pub aliq_pis: Option<Decimal>,          // 13
    pub quant_bc_pis: Option<Arc<str>>,     // 14
    pub aliq_pis_quant: Option<Decimal>,    // 15
    pub vl_pis: Option<Decimal>,            // 16
    pub cst_cofins: Option<u16>,            // 17
    pub vl_bc_cofins: Option<Decimal>,      // 18
    pub aliq_cofins: Option<Decimal>,       // 19
    pub quant_bc_cofins: Option<Arc<str>>,  // 20
    pub aliq_cofins_quant: Option<Decimal>, // 21
    pub vl_cofins: Option<Decimal>,         // 22
    pub cod_cta: Option<Arc<str>>,          // 23
}

impl_sped_record_trait!(RegistroD350);

impl SpedParser for RegistroD350 {
    type Output = RegistroD350;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro D350 possui 23 campos de dados + 2 delimitadores = 25.
        if len != 25 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 25,
                tamanho_encontrado: len,
            });
        }

        // Closure para campos de data (Option<NaiveDate>)
        let get_date_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cod_mod = fields.get(2).to_arc();
        let ecf_mod = fields.get(3).to_arc();
        let ecf_fab = fields.get(4).to_arc();
        let dt_doc = get_date_field(5, "DT_DOC")?;
        let cro = fields.get(6).to_arc();
        let crz = fields.get(7).to_arc();
        let num_coo_fin = fields.get(8).to_arc();
        let gt_fin = fields.get(9).to_arc();
        let vl_brt = get_decimal_field(10, "VL_BRT")?;
        let cst_pis = fields.get(11).parse_opt();
        let vl_bc_pis = get_decimal_field(12, "VL_BC_PIS")?;
        let aliq_pis = get_decimal_field(13, "ALIQ_PIS")?;
        let quant_bc_pis = fields.get(14).to_arc();
        let aliq_pis_quant = get_decimal_field(15, "ALIQ_PIS_QUANT")?;
        let vl_pis = get_decimal_field(16, "VL_PIS")?;
        let cst_cofins = fields.get(17).parse_opt();
        let vl_bc_cofins = get_decimal_field(18, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal_field(19, "ALIQ_COFINS")?;
        let quant_bc_cofins = fields.get(20).to_arc();
        let aliq_cofins_quant = get_decimal_field(21, "ALIQ_COFINS_QUANT")?;
        let vl_cofins = get_decimal_field(22, "VL_COFINS")?;
        let cod_cta = fields.get(23).to_arc();

        let reg = RegistroD350 {
            nivel: 3,
            bloco: 'D',
            registro: REGISTRO.to_string(),
            line_number,
            cod_mod,
            ecf_mod,
            ecf_fab,
            dt_doc,
            cro,
            crz,
            num_coo_fin,
            gt_fin,
            vl_brt,
            cst_pis,
            vl_bc_pis,
            aliq_pis,
            quant_bc_pis,
            aliq_pis_quant,
            vl_pis,
            cst_cofins,
            vl_bc_cofins,
            aliq_cofins,
            quant_bc_cofins,
            aliq_cofins_quant,
            vl_cofins,
            cod_cta,
        };

        Ok(reg)
    }
}
