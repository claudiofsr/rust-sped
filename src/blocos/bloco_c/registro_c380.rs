use crate::{
    EFDError, EFDResult, SpedParser, ToDecimal, ToNaiveDate, ToOptionalString,
    impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;

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

    pub cod_mod: Option<String>,       // 2
    pub dt_doc_ini: Option<NaiveDate>, // 3
    pub dt_doc_fin: Option<NaiveDate>, // 4
    pub num_doc_ini: Option<String>,   // 5
    pub num_doc_fin: Option<String>,   // 6
    pub vl_doc: Option<Decimal>,       // 7
    pub vl_doc_canc: Option<Decimal>,  // 8
}

impl_sped_record_trait!(RegistroC380);

impl SpedParser for RegistroC380 {
    type Output = RegistroC380;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro C380 possui 8 campos de dados + 2 delimitadores = 10.
        if len != 10 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 10,
                tamanho_encontrado: len,
            });
        }

        let get_date_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path.to_path_buf(), line_number, field_name)
        };

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let cod_mod = fields.get(2).to_optional_string();
        let dt_doc_ini = get_date_field(3, "DT_DOC_INI")?;
        let dt_doc_fin = get_date_field(4, "DT_DOC_FIN")?;
        let num_doc_ini = fields.get(5).to_optional_string();
        let num_doc_fin = fields.get(6).to_optional_string();
        let vl_doc = get_decimal_field(7, "VL_DOC")?;
        let vl_doc_canc = get_decimal_field(8, "VL_DOC_CANC")?;

        let reg = RegistroC380 {
            nivel: 3,
            bloco: 'C',
            registro,
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
