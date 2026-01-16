use crate::{EFDError, EFDResult, SpedParser, StringParser, ToNaiveDate, impl_reg_methods};
use chrono::NaiveDate;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "1010";

#[derive(Debug, Clone)]
pub struct Registro1010 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub num_proc: Option<Arc<str>>,     // 2
    pub id_sec_jud: Option<Arc<str>>,   // 3
    pub id_vara: Option<Arc<str>>,      // 4
    pub ind_nat_acao: Option<Arc<str>>, // 5
    pub desc_dec_jud: Option<Arc<str>>, // 6
    pub dt_sent_jud: Option<NaiveDate>, // 7
}

impl_reg_methods!(Registro1010);

impl SpedParser for Registro1010 {
    type Output = Registro1010;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro 1010 possui 7 campos de dados + 2 delimitadores = 9.
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

        let num_proc = fields.get(2).to_arc();
        let id_sec_jud = fields.get(3).to_arc();
        let id_vara = fields.get(4).to_arc();
        let ind_nat_acao = fields.get(5).to_arc();
        let desc_dec_jud = fields.get(6).to_arc();
        let dt_sent_jud = get_date(7, "DT_SENT_JUD")?;

        let reg = Registro1010 {
            nivel: 2,
            bloco: '1',
            registro: REGISTRO.into(),
            line_number,
            num_proc,
            id_sec_jud,
            id_vara,
            ind_nat_acao,
            desc_dec_jud,
            dt_sent_jud,
        };

        Ok(reg)
    }
}
