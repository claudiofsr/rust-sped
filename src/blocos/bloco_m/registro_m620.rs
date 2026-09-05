use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, ToNaiveDate,
    impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "M620";

#[derive(Debug, Clone)]
pub struct RegistroM620 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_aj: Option<CompactString>,   // 2
    pub vl_aj: Option<Decimal>,          // 3
    pub cod_aj: Option<CompactString>,   // 4
    pub num_doc: Option<usize>,          // 5
    pub descr_aj: Option<CompactString>, // 6
    pub dt_ref: Option<NaiveDate>,       // 7
}

impl_reg_methods!(RegistroM620);

impl SpedParser for RegistroM620 {
    type Output = RegistroM620;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro M620 possui 7 campos de dados + 2 delimitadores = 9.
        if len != 9 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 9,
                tamanho_encontrado: len,
            })
            .loc();
        }

        // --- Closures auxiliares para campos comuns ---

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        // Closure para campos de data (Option<NaiveDate>)
        let get_date = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let ind_aj = fields.get(2).to_compact_string();
        let vl_aj = get_decimal(3, "VL_AJ")?;
        let cod_aj = fields.get(4).to_compact_string();
        let num_doc = fields.get(5).parse_opt();
        let descr_aj = fields.get(6).to_compact_string();
        let dt_ref = get_date(7, "DT_REF")?;

        let reg = RegistroM620 {
            nivel: 4,
            bloco: 'M',
            registro: REGISTRO.into(),
            line_number,
            ind_aj,
            vl_aj,
            cod_aj,
            num_doc,
            descr_aj,
            dt_ref,
        };

        Ok(reg)
    }
}
