use crate::{
    EFDError, EFDResult, SpedParser, StringParser, ToDecimal, ToNaiveDate, impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "D500";

#[derive(Debug, Clone)]
pub struct RegistroD500 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_oper: Option<Arc<str>>,  // 2
    pub ind_emit: Option<Arc<str>>,  // 3
    pub cod_part: Option<Arc<str>>,  // 4
    pub cod_mod: Option<Arc<str>>,   // 5
    pub cod_sit: Option<Arc<str>>,   // 6
    pub ser: Option<Arc<str>>,       // 7
    pub sub: Option<Arc<str>>,       // 8
    pub num_doc: Option<Arc<str>>,   // 9
    pub dt_doc: Option<NaiveDate>,   // 10
    pub dt_a_p: Option<NaiveDate>,   // 11
    pub vl_doc: Option<Decimal>,     // 12
    pub vl_desc: Option<Decimal>,    // 13
    pub vl_serv: Option<Decimal>,    // 14
    pub vl_serv_nt: Option<Decimal>, // 15
    pub vl_terc: Option<Decimal>,    // 16
    pub vl_da: Option<Decimal>,      // 17
    pub vl_bc_icms: Option<Decimal>, // 18
    pub vl_icms: Option<Decimal>,    // 19
    pub cod_inf: Option<Arc<str>>,   // 20
    pub vl_pis: Option<Decimal>,     // 21
    pub vl_cofins: Option<Decimal>,  // 22
}

impl_sped_record_trait!(RegistroD500);

impl SpedParser for RegistroD500 {
    type Output = RegistroD500;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro D500 possui 22 campos de dados + 2 delimitadores = 24.
        if len != 24 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 24,
                tamanho_encontrado: len,
            });
        }

        // Closure para campos de data (Option<NaiveDate>)
        let get_date = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let ind_oper = fields.get(2).to_arc();
        let ind_emit = fields.get(3).to_arc();
        let cod_part = fields.get(4).to_arc();
        let cod_mod = fields.get(5).to_arc();
        let cod_sit = fields.get(6).to_arc();
        let ser = fields.get(7).to_arc();
        let sub = fields.get(8).to_arc();
        let num_doc = fields.get(9).to_arc();
        let dt_doc = get_date(10, "DT_DOC")?;
        let dt_a_p = get_date(11, "DT_A_P")?;
        let vl_doc = get_decimal(12, "VL_DOC")?;
        let vl_desc = get_decimal(13, "VL_DESC")?;
        let vl_serv = get_decimal(14, "VL_SERV")?;
        let vl_serv_nt = get_decimal(15, "VL_SERV_NT")?;
        let vl_terc = get_decimal(16, "VL_TERC")?;
        let vl_da = get_decimal(17, "VL_DA")?;
        let vl_bc_icms = get_decimal(18, "VL_BC_ICMS")?;
        let vl_icms = get_decimal(19, "VL_ICMS")?;
        let cod_inf = fields.get(20).to_arc();
        let vl_pis = get_decimal(21, "VL_PIS")?;
        let vl_cofins = get_decimal(22, "VL_COFINS")?;

        let reg = RegistroD500 {
            nivel: 3,
            bloco: 'D',
            registro: REGISTRO.into(),
            line_number,
            ind_oper,
            ind_emit,
            cod_part,
            cod_mod,
            cod_sit,
            ser,
            sub,
            num_doc,
            dt_doc,
            dt_a_p,
            vl_doc,
            vl_desc,
            vl_serv,
            vl_serv_nt,
            vl_terc,
            vl_da,
            vl_bc_icms,
            vl_icms,
            cod_inf,
            vl_pis,
            vl_cofins,
        };

        Ok(reg)
    }
}
