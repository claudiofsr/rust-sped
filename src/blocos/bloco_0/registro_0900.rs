use crate::{EFDError, EFDResult, ResultExt, SpedParser, ToDecimal, impl_reg_methods};
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "0900";

#[derive(Debug, Clone)]
pub struct Registro0900 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub rec_total_bloco_a: Option<Decimal>,     // 2
    pub rec_nrb_bloco_a: Option<Decimal>,       // 3
    pub rec_total_bloco_c: Option<Decimal>,     // 4
    pub rec_nrb_bloco_c: Option<Decimal>,       // 5
    pub rec_total_bloco_d: Option<Decimal>,     // 6
    pub rec_nrb_bloco_d: Option<Decimal>,       // 7
    pub rec_total_bloco_f: Option<Decimal>,     // 8
    pub rec_nrb_bloco_f: Option<Decimal>,       // 9
    pub rec_total_bloco_i: Option<Decimal>,     // 10
    pub rec_nrb_bloco_i: Option<Decimal>,       // 11
    pub rec_total_bloco_1: Option<Decimal>,     // 12
    pub rec_nrb_bloco_1: Option<Decimal>,       // 13
    pub rec_total_periodo: Option<Decimal>,     // 14
    pub rec_total_nrb_periodo: Option<Decimal>, // 15
}

impl_reg_methods!(Registro0900);

impl SpedParser for Registro0900 {
    type Output = Registro0900;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != 17 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 17,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let rec_total_bloco_a = get_decimal(2, "REC_TOTAL_BLOCO_A")?;
        let rec_nrb_bloco_a = get_decimal(3, "REC_NRB_BLOCO_A")?;
        let rec_total_bloco_c = get_decimal(4, "REC_TOTAL_BLOCO_C")?;
        let rec_nrb_bloco_c = get_decimal(5, "REC_NRB_BLOCO_C")?;
        let rec_total_bloco_d = get_decimal(6, "REC_TOTAL_BLOCO_D")?;
        let rec_nrb_bloco_d = get_decimal(7, "REC_NRB_BLOCO_D")?;
        let rec_total_bloco_f = get_decimal(8, "REC_TOTAL_BLOCO_F")?;
        let rec_nrb_bloco_f = get_decimal(9, "REC_NRB_BLOCO_F")?;
        let rec_total_bloco_i = get_decimal(10, "REC_TOTAL_BLOCO_I")?;
        let rec_nrb_bloco_i = get_decimal(11, "REC_NRB_BLOCO_I")?;
        let rec_total_bloco_1 = get_decimal(12, "REC_TOTAL_BLOCO_1")?;
        let rec_nrb_bloco_1 = get_decimal(13, "REC_NRB_BLOCO_1")?;
        let rec_total_periodo = get_decimal(14, "REC_TOTAL_PERIODO")?;
        let rec_total_nrb_periodo = get_decimal(15, "REC_TOTAL_NRB_PERIODO")?;

        let reg = Registro0900 {
            nivel: 2,
            bloco: '0',
            registro: REGISTRO.into(),
            line_number,
            rec_total_bloco_a,
            rec_nrb_bloco_a,
            rec_total_bloco_c,
            rec_nrb_bloco_c,
            rec_total_bloco_d,
            rec_nrb_bloco_d,
            rec_total_bloco_f,
            rec_nrb_bloco_f,
            rec_total_bloco_i,
            rec_nrb_bloco_i,
            rec_total_bloco_1,
            rec_nrb_bloco_1,
            rec_total_periodo,
            rec_total_nrb_periodo,
        };

        Ok(reg)
    }
}
