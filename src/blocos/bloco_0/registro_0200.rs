use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, impl_reg_methods,
};
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const EXPECTED_FIELDS: usize = 14;
const REGISTRO: &str = "0200";

#[derive(Debug, Clone)]
pub struct Registro0200 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_item: Option<Arc<str>>,     // 2
    pub descr_item: Option<Arc<str>>,   // 3
    pub cod_barra: Option<Arc<str>>,    // 4
    pub cod_ant_item: Option<Arc<str>>, // 5
    pub unid_inv: Option<Arc<str>>,     // 6
    pub tipo_item: Option<u8>,          // 7
    pub cod_ncm: Option<Arc<str>>,      // 8
    pub ex_ipi: Option<Arc<str>>,       // 9
    pub cod_gen: Option<Arc<str>>,      // 10
    pub cod_lst: Option<Arc<str>>,      // 11
    pub aliq_icms: Option<Decimal>,     // 12
}

impl_reg_methods!(Registro0200);

impl SpedParser for Registro0200 {
    type Output = Registro0200;

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

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cod_item = fields.get(2).to_arc();
        let descr_item = fields.get(3).to_upper_arc();
        let cod_barra = fields.get(4).to_arc();
        let cod_ant_item = fields.get(5).to_arc();
        let unid_inv = fields.get(6).to_arc();
        let tipo_item = fields.get(7).parse_opt();
        let cod_ncm = fields.get(8).to_arc();
        let ex_ipi = fields.get(9).to_arc();
        let cod_gen = fields.get(10).to_arc();
        let cod_lst = fields.get(11).to_arc();
        let aliq_icms = get_decimal(12, "ALIQ_ICMS")?;

        let reg = Registro0200 {
            nivel: 3,
            bloco: '0',
            registro: REGISTRO.into(),
            line_number,
            cod_item,
            descr_item,
            cod_barra,
            cod_ant_item,
            unid_inv,
            tipo_item,
            cod_ncm,
            ex_ipi,
            cod_gen,
            cod_lst,
            aliq_icms,
        };

        Ok(reg)
    }
}
