use crate::{
    CodigoSituacaoTributaria, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal,
    ToNaiveDate, impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "1011";

#[derive(Debug, Clone)]
pub struct Registro1011 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub reg_ref: Option<CompactString>,               // 2
    pub chave_doc: Option<CompactString>,             // 3
    pub cod_part: Option<CompactString>,              // 4
    pub cod_item: Option<CompactString>,              // 5
    pub dt_oper: Option<NaiveDate>,                   // 6
    pub vl_oper: Option<Decimal>,                     // 7
    pub cst_pis: Option<CodigoSituacaoTributaria>,    // 8
    pub vl_bc_pis: Option<Decimal>,                   // 9
    pub aliq_pis: Option<Decimal>,                    // 10
    pub vl_pis: Option<Decimal>,                      // 11
    pub cst_cofins: Option<CodigoSituacaoTributaria>, // 12
    pub vl_bc_cofins: Option<Decimal>,                // 13
    pub aliq_cofins: Option<Decimal>,                 // 14
    pub vl_cofins: Option<Decimal>,                   // 15
    pub cst_pis_susp: Option<u16>,                    // 16
    pub vl_bc_pis_susp: Option<Decimal>,              // 17
    pub aliq_pis_susp: Option<Decimal>,               // 18
    pub vl_pis_susp: Option<Decimal>,                 // 19
    pub cst_cofins_susp: Option<u16>,                 // 20
    pub vl_bc_cofins_susp: Option<Decimal>,           // 21
    pub aliq_cofins_susp: Option<Decimal>,            // 22
    pub vl_cofins_susp: Option<Decimal>,              // 23
    pub cod_cta: Option<CompactString>,               // 24
    pub cod_ccus: Option<CompactString>,              // 25
    pub desc_doc_oper: Option<CompactString>,         // 26
}

impl_reg_methods!(Registro1011);

impl SpedParser for Registro1011 {
    type Output = Registro1011;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro 1011 possui 26 campos de dados + 2 delimitadores = 28.
        if len != 28 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 28,
                tamanho_encontrado: len,
            })
            .loc();
        }

        // Closures auxiliares
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

        let reg_ref = fields.get(2).to_compact_string();
        let chave_doc = fields.get(3).to_compact_string();
        let cod_part = fields.get(4).to_compact_string();
        let cod_item = fields.get(5).to_compact_string();
        let dt_oper = get_date(6, "DT_OPER")?;
        let vl_oper = get_decimal(7, "VL_OPER")?;
        let cst_pis = fields
            .get(8)
            .to_efd_field(file_path, line_number, "CST_PIS")?;
        let vl_bc_pis = get_decimal(9, "VL_BC_PIS")?;
        let aliq_pis = get_decimal(10, "ALIQ_PIS")?;
        let vl_pis = get_decimal(11, "VL_PIS")?;
        let cst_cofins = fields
            .get(12)
            .to_efd_field(file_path, line_number, "CST_COFINS")?;
        let vl_bc_cofins = get_decimal(13, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(14, "ALIQ_COFINS")?;
        let vl_cofins = get_decimal(15, "VL_COFINS")?;
        let cst_pis_susp = fields.get(16).parse_opt();
        let vl_bc_pis_susp = get_decimal(17, "VL_BC_PIS_SUSP")?;
        let aliq_pis_susp = get_decimal(18, "ALIQ_PIS_SUSP")?;
        let vl_pis_susp = get_decimal(19, "VL_PIS_SUSP")?;
        let cst_cofins_susp = fields.get(20).parse_opt();
        let vl_bc_cofins_susp = get_decimal(21, "VL_BC_COFINS_SUSP")?;
        let aliq_cofins_susp = get_decimal(22, "ALIQ_COFINS_SUSP")?;
        let vl_cofins_susp = get_decimal(23, "VL_COFINS_SUSP")?;
        let cod_cta = fields.get(24).to_compact_string();
        let cod_ccus = fields.get(25).to_compact_string();
        let desc_doc_oper = fields.get(26).to_compact_string();

        let reg = Registro1011 {
            nivel: 3,
            bloco: '1',
            registro: REGISTRO.into(),
            line_number,
            reg_ref,
            chave_doc,
            cod_part,
            cod_item,
            dt_oper,
            vl_oper,
            cst_pis,
            vl_bc_pis,
            aliq_pis,
            vl_pis,
            cst_cofins,
            vl_bc_cofins,
            aliq_cofins,
            vl_cofins,
            cst_pis_susp,
            vl_bc_pis_susp,
            aliq_pis_susp,
            vl_pis_susp,
            cst_cofins_susp,
            vl_bc_cofins_susp,
            aliq_cofins_susp,
            vl_cofins_susp,
            cod_cta,
            cod_ccus,
            desc_doc_oper,
        };

        Ok(reg)
    }
}
