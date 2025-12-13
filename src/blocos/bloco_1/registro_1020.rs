use crate::{EFDError, EFDResult, SpedParser, StringParser, ToNaiveDate, impl_sped_record_trait};
use chrono::NaiveDate;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "1020";

#[derive(Debug, Clone)]
pub struct Registro1020 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub num_proc: Option<Arc<str>>,     // 2
    pub ind_nat_acao: Option<Arc<str>>, // 3
    pub dt_dec_adm: Option<NaiveDate>,  // 4
}

impl_sped_record_trait!(Registro1020);

impl SpedParser for Registro1020 {
    type Output = Registro1020;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro 1020 possui 4 campos de dados + 2 delimitadores = 6.
        if len != 6 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 6,
                tamanho_encontrado: len,
            });
        }

        // Closure para campos de data (Option<NaiveDate>)
        let get_date = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let num_proc = fields.get(2).to_arc();
        let ind_nat_acao = fields.get(3).to_arc();
        let dt_dec_adm = get_date(4, "DT_DEC_ADM")?;

        let reg = Registro1020 {
            nivel: 2,
            bloco: '1',
            registro: REGISTRO.into(),
            line_number,
            num_proc,
            ind_nat_acao,
            dt_dec_adm,
        };

        Ok(reg)
    }
}
