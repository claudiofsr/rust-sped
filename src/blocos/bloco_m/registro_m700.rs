use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, ToNaiveDate,
    impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "M700";

#[derive(Debug, Clone)]
pub struct RegistroM700 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_cont: Option<CompactString>,      // 2
    pub vl_cont_apur_difer: Option<Decimal>,  // 3
    pub nat_cred_desc: Option<CompactString>, // 4
    pub vl_cred_desc_difer: Option<Decimal>,  // 5
    pub vl_cont_difer_ant: Option<Decimal>,   // 6
    pub per_apur: Option<CompactString>, // 7 (Pode ser NaiveDate ou String dependendo do formato)
    pub dt_receb: Option<NaiveDate>,     // 8
}

impl_reg_methods!(RegistroM700);

impl SpedParser for RegistroM700 {
    type Output = RegistroM700;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro M700 possui 8 campos de dados + 2 delimitadores = 10.
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

        let cod_cont = fields.get(2).to_compact_string();
        let vl_cont_apur_difer = get_decimal(3, "VL_CONT_APUR_DIFER")?;
        let nat_cred_desc = fields.get(4).to_compact_string();
        let vl_cred_desc_difer = get_decimal(5, "VL_CRED_DESC_DIFER")?;
        let vl_cont_difer_ant = get_decimal(6, "VL_CONT_DIFER_ANT")?;
        let per_apur = fields.get(7).to_compact_string(); // Manter como String, se for um período sem formato de data
        let dt_receb = get_date(8, "DT_RECEB")?;

        let reg = RegistroM700 {
            nivel: 2,
            bloco: 'M',
            registro: REGISTRO.into(),
            line_number,
            cod_cont,
            vl_cont_apur_difer,
            nat_cred_desc,
            vl_cred_desc_difer,
            vl_cont_difer_ant,
            per_apur,
            dt_receb,
        };

        Ok(reg)
    }
}
