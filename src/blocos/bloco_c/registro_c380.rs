use crate::{
    EFDError, EFDResult, SpedParser, StringParser, ToDecimal, ToNaiveDate, impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "C380";

#[derive(Debug, Clone)]
pub struct RegistroC380 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_mod: Option<Arc<str>>,     // 2
    pub dt_doc_ini: Option<NaiveDate>, // 3
    pub dt_doc_fin: Option<NaiveDate>, // 4
    pub num_doc_ini: Option<Arc<str>>, // 5
    pub num_doc_fin: Option<Arc<str>>, // 6
    pub vl_doc: Option<Decimal>,       // 7
    pub vl_doc_canc: Option<Decimal>,  // 8
}

impl_sped_record_trait!(RegistroC380);

impl SpedParser for RegistroC380 {
    type Output = RegistroC380;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C380 possui 8 campos de dados + 2 delimitadores = 10.
        if len != 10 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 10,
                tamanho_encontrado: len,
            });
        }

        let get_date_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cod_mod = fields.get(2).to_arc();
        let dt_doc_ini = get_date_field(3, "DT_DOC_INI")?;
        let dt_doc_fin = get_date_field(4, "DT_DOC_FIN")?;
        let num_doc_ini = fields.get(5).to_arc();
        let num_doc_fin = fields.get(6).to_arc();
        let vl_doc = get_decimal_field(7, "VL_DOC")?;
        let vl_doc_canc = get_decimal_field(8, "VL_DOC_CANC")?;

        let reg = RegistroC380 {
            nivel: 3,
            bloco: 'C',
            registro: REGISTRO.to_string(),
            line_number,
            cod_mod,
            dt_doc_ini,
            dt_doc_fin,
            num_doc_ini,
            num_doc_fin,
            vl_doc,
            vl_doc_canc,
        };

        Ok(reg)
    }
}
