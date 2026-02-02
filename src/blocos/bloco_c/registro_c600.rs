use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, ToNaiveDate,
    impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C600";

#[derive(Debug, Clone)]
pub struct RegistroC600 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_mod: Option<CompactString>,  // 2
    pub cod_mun: Option<CompactString>,  // 3
    pub ser: Option<CompactString>,      // 4
    pub sub: Option<CompactString>,      // 5
    pub cod_cons: Option<CompactString>, // 6
    pub qtd_cons: Option<CompactString>, // 7 (Assumindo que QTD pode ser string se for só int)
    pub qtd_canc: Option<CompactString>, // 8 (Assumindo que QTD pode ser string se for só int)
    pub dt_doc: Option<NaiveDate>,       // 9
    pub vl_doc: Option<Decimal>,         // 10
    pub vl_desc: Option<Decimal>,        // 11
    pub cons: Option<CompactString>,     // 12
    pub vl_forn: Option<Decimal>,        // 13
    pub vl_serv_nt: Option<Decimal>,     // 14
    pub vl_terc: Option<Decimal>,        // 15
    pub vl_da: Option<Decimal>,          // 16
    pub vl_bc_icms: Option<Decimal>,     // 17
    pub vl_icms: Option<Decimal>,        // 18
    pub vl_bc_icms_st: Option<Decimal>,  // 19
    pub vl_icms_st: Option<Decimal>,     // 20
    pub vl_pis: Option<Decimal>,         // 21
    pub vl_cofins: Option<Decimal>,      // 22
}

impl_reg_methods!(RegistroC600);

impl SpedParser for RegistroC600 {
    type Output = RegistroC600;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C600 possui 22 campos de dados + 2 delimitadores = 24.
        if len != 24 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 24,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_date = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cod_mod = fields.get(2).to_compact_string();
        let cod_mun = fields.get(3).to_compact_string();
        let ser = fields.get(4).to_compact_string();
        let sub = fields.get(5).to_compact_string();
        let cod_cons = fields.get(6).to_compact_string();
        let qtd_cons = fields.get(7).to_compact_string(); // Pode ser String se for apenas inteiro
        let qtd_canc = fields.get(8).to_compact_string(); // Pode ser String se for apenas inteiro
        let dt_doc = get_date(9, "DT_DOC")?;
        let vl_doc = get_decimal(10, "VL_DOC")?;
        let vl_desc = get_decimal(11, "VL_DESC")?;
        let cons = fields.get(12).to_compact_string();
        let vl_forn = get_decimal(13, "VL_FORN")?;
        let vl_serv_nt = get_decimal(14, "VL_SERV_NT")?;
        let vl_terc = get_decimal(15, "VL_TERC")?;
        let vl_da = get_decimal(16, "VL_DA")?;
        let vl_bc_icms = get_decimal(17, "VL_BC_ICMS")?;
        let vl_icms = get_decimal(18, "VL_ICMS")?;
        let vl_bc_icms_st = get_decimal(19, "VL_BC_ICMS_ST")?;
        let vl_icms_st = get_decimal(20, "VL_ICMS_ST")?;
        let vl_pis = get_decimal(21, "VL_PIS")?;
        let vl_cofins = get_decimal(22, "VL_COFINS")?;

        let reg = RegistroC600 {
            nivel: 3,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cod_mod,
            cod_mun,
            ser,
            sub,
            cod_cons,
            qtd_cons,
            qtd_canc,
            dt_doc,
            vl_doc,
            vl_desc,
            cons,
            vl_forn,
            vl_serv_nt,
            vl_terc,
            vl_da,
            vl_bc_icms,
            vl_icms,
            vl_bc_icms_st,
            vl_icms_st,
            vl_pis,
            vl_cofins,
        };

        Ok(reg)
    }
}
