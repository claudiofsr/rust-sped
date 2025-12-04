use crate::{
    EFDError, EFDResult, SpedParser, StringParser, ToDecimal, ToNaiveDate, impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "C190";

#[derive(Debug, Clone)]
pub struct RegistroC190 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_mod: Option<Arc<str>>,     // 2
    pub dt_ref_ini: Option<NaiveDate>, // 3
    pub dt_ref_fin: Option<NaiveDate>, // 4
    pub cod_item: Option<Arc<str>>,    // 5
    pub cod_ncm: Option<Arc<str>>,     // 6
    pub ex_ipi: Option<Arc<str>>,      // 7
    pub vl_tot_item: Option<Decimal>,  // 8
}

impl_sped_record_trait!(RegistroC190);

impl SpedParser for RegistroC190 {
    type Output = RegistroC190;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C190 possui 8 campos de dados + 2 delimitadores = 10.
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
        let dt_ref_ini = get_date_field(3, "DT_REF_INI")?;
        let dt_ref_fin = get_date_field(4, "DT_REF_FIN")?;
        let cod_item = fields.get(5).to_arc();
        let cod_ncm = fields.get(6).to_arc();
        let ex_ipi = fields.get(7).to_arc();
        let vl_tot_item = get_decimal_field(8, "VL_TOT_ITEM")?;

        let reg = RegistroC190 {
            nivel: 3,
            bloco: 'C',
            registro: REGISTRO.to_string(),
            line_number,
            cod_mod,
            dt_ref_ini,
            dt_ref_fin,
            cod_item,
            cod_ncm,
            ex_ipi,
            vl_tot_item,
        };

        Ok(reg)
    }
}
