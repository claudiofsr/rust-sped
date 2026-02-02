use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, ToNaiveDate,
    impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C395";

#[derive(Debug, Clone)]
pub struct RegistroC395 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_mod: Option<CompactString>,  // 2
    pub cod_part: Option<CompactString>, // 3
    pub ser: Option<CompactString>,      // 4
    pub sub_ser: Option<CompactString>,  // 5
    pub num_doc: Option<usize>,          // 6
    pub dt_doc: Option<NaiveDate>,       // 7
    pub vl_doc: Option<Decimal>,         // 8
}

impl_reg_methods!(RegistroC395);

impl SpedParser for RegistroC395 {
    type Output = RegistroC395;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C395 possui 8 campos de dados + 2 delimitadores = 10.
        if len != 10 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 10,
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
        let cod_part = fields.get(3).to_compact_string();
        let ser = fields.get(4).to_compact_string();
        let sub_ser = fields.get(5).to_compact_string();
        let num_doc = fields.get(6).parse_opt();
        let dt_doc = get_date(7, "DT_DOC")?;
        let vl_doc = get_decimal(8, "VL_DOC")?;

        let reg = RegistroC395 {
            nivel: 3,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cod_mod,
            cod_part,
            ser,
            sub_ser,
            num_doc,
            dt_doc,
            vl_doc,
        };

        Ok(reg)
    }
}
