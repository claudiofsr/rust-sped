use crate::{
    EFDError, EFDResult, SpedParser, StringParser, ToDecimal, ToNaiveDate, impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "M615";

#[derive(Debug, Clone)]
pub struct RegistroM615 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_aj_bc: Option<Arc<str>>,   // 2
    pub vl_aj_bc: Option<Decimal>,     // 3
    pub cod_aj_bc: Option<Arc<str>>,   // 4
    pub num_doc: Option<Arc<str>>,     // 5
    pub descr_aj_bc: Option<Arc<str>>, // 6
    pub dt_ref: Option<NaiveDate>,     // 7
    pub cod_cta: Option<Arc<str>>,     // 8
    pub cnpj: Option<Arc<str>>,        // 9
    pub info_compl: Option<Arc<str>>,  // 10
}

impl_sped_record_trait!(RegistroM615);

impl SpedParser for RegistroM615 {
    type Output = RegistroM615;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro M615 possui 10 campos de dados + 2 delimitadores = 12.
        if len != 12 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 12,
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

        let ind_aj_bc = fields.get(2).to_arc();
        let vl_aj_bc = get_decimal(3, "VL_AJ_BC")?;
        let cod_aj_bc = fields.get(4).to_arc();
        let num_doc = fields.get(5).to_arc();
        let descr_aj_bc = fields.get(6).to_arc();
        let dt_ref = get_date(7, "DT_REF")?;
        let cod_cta = fields.get(8).to_arc();
        let cnpj = fields.get(9).to_arc();
        let info_compl = fields.get(10).to_arc();

        let reg = RegistroM615 {
            nivel: 4,
            bloco: 'M',
            registro: REGISTRO.into(),
            line_number,
            ind_aj_bc,
            vl_aj_bc,
            cod_aj_bc,
            num_doc,
            descr_aj_bc,
            dt_ref,
            cod_cta,
            cnpj,
            info_compl,
        };

        Ok(reg)
    }
}
