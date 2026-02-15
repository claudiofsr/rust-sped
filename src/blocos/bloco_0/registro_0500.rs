use crate::{
    EFDError, EFDResult, GrupoDeContas, ResultExt, SpedParser, StringParser, ToEFDField,
    ToNaiveDate, impl_reg_methods,
};
use chrono::NaiveDate;
use std::{path::Path, sync::Arc};

const EXPECTED_FIELDS: usize = 11;
const REGISTRO: &str = "0500";

#[derive(Debug, Clone)]
pub struct Registro0500 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub dt_alt: Option<NaiveDate>,         // 2
    pub cod_nat_cc: Option<GrupoDeContas>, // 3
    pub ind_cta: Option<Arc<str>>,         // 4
    pub nivel_conta: Option<Arc<str>>, // 5 (renomeado para evitar conflito com 'nivel' do struct)
    pub cod_cta: Option<Arc<str>>,     // 6
    pub nome_cta: Option<Arc<str>>,    // 7
    pub cod_cta_ref: Option<Arc<str>>, // 8
    pub cnpj_est: Option<Arc<str>>,    // 9
}

impl_reg_methods!(Registro0500);

impl SpedParser for Registro0500 {
    type Output = Registro0500;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != EXPECTED_FIELDS {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: EXPECTED_FIELDS,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_date = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let dt_alt = get_date(2, "DT_ALT")?;
        let cod_nat_cc = fields
            .get(3)
            .to_efd_field(file_path, line_number, "COD_NAT_CC")?;
        let ind_cta = fields.get(4).to_arc();
        let nivel_conta = fields.get(5).to_arc();
        let cod_cta = fields.get(6).to_arc();
        let nome_cta = fields.get(7).to_upper_arc();
        let cod_cta_ref = fields.get(8).to_arc();
        let cnpj_est = fields.get(9).to_arc();

        let reg = Registro0500 {
            nivel: 2,
            bloco: '0',
            registro: REGISTRO.into(),
            line_number,
            dt_alt,
            cod_nat_cc,
            ind_cta,
            nivel_conta,
            cod_cta,
            nome_cta,
            cod_cta_ref,
            cnpj_est,
        };

        Ok(reg)
    }
}
