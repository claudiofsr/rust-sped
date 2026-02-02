use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, ToNaiveDate,
    impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "1501";

#[derive(Debug, Clone)]
pub struct Registro1501 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_part: Option<CompactString>,      // 2
    pub cod_item: Option<CompactString>,      // 3
    pub cod_mod: Option<CompactString>,       // 4
    pub ser: Option<CompactString>,           // 5
    pub sub_ser: Option<CompactString>,       // 6
    pub num_doc: Option<usize>,               // 7
    pub dt_oper: Option<NaiveDate>,           // 8
    pub chv_nfe: Option<CompactString>,       // 9
    pub vl_oper: Option<Decimal>,             // 10
    pub cfop: Option<u16>,                    // 11
    pub nat_bc_cred: Option<u16>,             // 12
    pub ind_orig_cred: Option<CompactString>, // 13
    pub cst_cofins: Option<u16>,              // 14
    pub vl_bc_cofins: Option<Decimal>,        // 15
    pub aliq_cofins: Option<Decimal>,         // 16
    pub vl_cofins: Option<Decimal>,           // 17
    pub cod_cta: Option<CompactString>,       // 18
    pub cod_ccus: Option<CompactString>,      // 19
    pub desc_compl: Option<CompactString>,    // 20
    pub per_escrit: Option<CompactString>,    // 21
    pub cnpj: Option<CompactString>,          // 22
}

impl_reg_methods!(Registro1501);

impl SpedParser for Registro1501 {
    type Output = Registro1501;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro 1501 possui 22 campos de dados + 2 delimitadores = 24.
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

        let cod_part = fields.get(2).to_compact_string();
        let cod_item = fields.get(3).to_compact_string();
        let cod_mod = fields.get(4).to_compact_string();
        let ser = fields.get(5).to_compact_string();
        let sub_ser = fields.get(6).to_compact_string();
        let num_doc = fields.get(7).parse_opt();
        let dt_oper = get_date(8, "DT_OPER")?;
        let chv_nfe = fields.get(9).to_compact_string();
        let vl_oper = get_decimal(10, "VL_OPER")?;
        let cfop = fields.get(11).parse_opt();
        let nat_bc_cred = fields.get(12).parse_opt();
        let ind_orig_cred = fields.get(13).to_compact_string();
        let cst_cofins = fields.get(14).parse_opt();
        let vl_bc_cofins = get_decimal(15, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(16, "ALIQ_COFINS")?;
        let vl_cofins = get_decimal(17, "VL_COFINS")?;
        let cod_cta = fields.get(18).to_compact_string();
        let cod_ccus = fields.get(19).to_compact_string();
        let desc_compl = fields.get(20).to_compact_string();
        let per_escrit = fields.get(21).to_compact_string();
        let cnpj = fields.get(22).to_compact_string();

        let reg = Registro1501 {
            nivel: 3,
            bloco: '1',
            registro: REGISTRO.into(),
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
