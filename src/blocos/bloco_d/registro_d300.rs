use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, ToNaiveDate,
    impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "D300";

#[derive(Debug, Clone)]
pub struct RegistroD300 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_mod: Option<CompactString>,     // 2
    pub ser: Option<CompactString>,         // 3
    pub sub: Option<CompactString>,         // 4
    pub num_doc_ini: Option<CompactString>, // 5
    pub num_doc_fin: Option<CompactString>, // 6
    pub cfop: Option<u16>,                  // 7
    pub dt_ref: Option<NaiveDate>,          // 8
    pub vl_doc: Option<Decimal>,            // 9
    pub vl_desc: Option<Decimal>,           // 10
    pub cst_pis: Option<u16>,               // 11
    pub vl_bc_pis: Option<Decimal>,         // 12
    pub aliq_pis: Option<Decimal>,          // 13
    pub vl_pis: Option<Decimal>,            // 14
    pub cst_cofins: Option<u16>,            // 15
    pub vl_bc_cofins: Option<Decimal>,      // 16
    pub aliq_cofins: Option<Decimal>,       // 17
    pub vl_cofins: Option<Decimal>,         // 18
    pub cod_cta: Option<CompactString>,     // 19
}

impl_reg_methods!(RegistroD300);

impl SpedParser for RegistroD300 {
    type Output = RegistroD300;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro D300 possui 19 campos de dados + 2 delimitadores = 21.
        if len != 21 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 21,
                tamanho_encontrado: len,
            })
            .loc();
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

        let cod_mod = fields.get(2).to_compact_string();
        let ser = fields.get(3).to_compact_string();
        let sub = fields.get(4).to_compact_string();
        let num_doc_ini = fields.get(5).to_compact_string();
        let num_doc_fin = fields.get(6).to_compact_string();
        let cfop = fields.get(7).parse_opt();
        let dt_ref = get_date(8, "DT_REF")?;
        let vl_doc = get_decimal(9, "VL_DOC")?;
        let vl_desc = get_decimal(10, "VL_DESC")?;
        let cst_pis = fields.get(11).parse_opt();
        let vl_bc_pis = get_decimal(12, "VL_BC_PIS")?;
        let aliq_pis = get_decimal(13, "ALIQ_PIS")?;
        let vl_pis = get_decimal(14, "VL_PIS")?;
        let cst_cofins = fields.get(15).parse_opt();
        let vl_bc_cofins = get_decimal(16, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(17, "ALIQ_COFINS")?;
        let vl_cofins = get_decimal(18, "VL_COFINS")?;
        let cod_cta = fields.get(19).to_compact_string();

        let reg = RegistroD300 {
            nivel: 3,
            bloco: 'D',
            registro: REGISTRO.into(),
            line_number,
            cod_mod,
            ser,
            sub,
            num_doc_ini,
            num_doc_fin,
            cfop,
            dt_ref,
            vl_doc,
            vl_desc,
            cst_pis,
            vl_bc_pis,
            aliq_pis,
            vl_pis,
            cst_cofins,
            vl_bc_cofins,
            aliq_cofins,
            vl_cofins,
            cod_cta,
        };

        Ok(reg)
    }
}
