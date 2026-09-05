use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, ToNaiveDate,
    impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "D200";

#[derive(Debug, Clone)]
pub struct RegistroD200 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_mod: Option<CompactString>, // 2
    pub cod_sit: Option<CompactString>, // 3
    pub ser: Option<CompactString>,     // 4
    pub sub: Option<CompactString>,     // 5
    pub num_doc_ini: Option<usize>,     // 6
    pub num_doc_fin: Option<usize>,     // 7
    pub cfop: Option<u16>,              // 8
    pub dt_ref: Option<NaiveDate>,      // 9
    pub vl_doc: Option<Decimal>,        // 10
    pub vl_desc: Option<Decimal>,       // 11
}

impl_reg_methods!(RegistroD200);

impl SpedParser for RegistroD200 {
    type Output = RegistroD200;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro D200 possui 11 campos de dados + 2 delimitadores = 13.
        if len != 13 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 13,
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
        let cod_sit = fields.get(3).to_compact_string();
        let ser = fields.get(4).to_compact_string();
        let sub = fields.get(5).to_compact_string();
        let num_doc_ini = fields.get(6).parse_opt();
        let num_doc_fin = fields.get(7).parse_opt();
        let cfop = fields.get(8).parse_opt();
        let dt_ref = get_date(9, "DT_REF")?;
        let vl_doc = get_decimal(10, "VL_DOC")?;
        let vl_desc = get_decimal(11, "VL_DESC")?;

        let reg = RegistroD200 {
            nivel: 3,
            bloco: 'D',
            registro: REGISTRO.into(),
            line_number,
            cod_mod,
            cod_sit,
            ser,
            sub,
            num_doc_ini,
            num_doc_fin,
            cfop,
            dt_ref,
            vl_doc,
            vl_desc,
        };

        Ok(reg)
    }
}
