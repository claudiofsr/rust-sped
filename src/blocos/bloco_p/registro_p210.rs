use crate::{
    EFDError, EFDResult, SpedParser, StringParser, ToDecimal, ToNaiveDate, impl_reg_methods,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "P210";

#[derive(Debug, Clone)]
pub struct RegistroP210 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_aj: Option<Arc<str>>,   // 2
    pub vl_aj: Option<Decimal>,     // 3
    pub cod_aj: Option<Arc<str>>,   // 4
    pub num_doc: Option<usize>,     // 5
    pub descr_aj: Option<Arc<str>>, // 6
    pub dt_ref: Option<NaiveDate>,  // 7
}

impl_reg_methods!(RegistroP210);

impl SpedParser for RegistroP210 {
    type Output = RegistroP210;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro P210 possui 7 campos de dados + 2 delimitadores = 9.
        if len != 9 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 9,
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

        let ind_aj = fields.get(2).to_arc();
        let vl_aj = get_decimal(3, "VL_AJ")?;
        let cod_aj = fields.get(4).to_arc();
        let num_doc = fields.get(5).parse_opt();
        let descr_aj = fields.get(6).map(|&s| Arc::from(s));
        let dt_ref = get_date(7, "DT_REF")?;

        let reg = RegistroP210 {
            nivel: 3,
            bloco: 'P',
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
