use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToNaiveDate, impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use std::path::Path;

const REGISTRO: &str = "1010";

#[derive(Debug, Clone)]
pub struct Registro1010 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub num_proc: Option<CompactString>,     // 2
    pub id_sec_jud: Option<CompactString>,   // 3
    pub id_vara: Option<CompactString>,      // 4
    pub ind_nat_acao: Option<CompactString>, // 5
    pub desc_dec_jud: Option<CompactString>, // 6
    pub dt_sent_jud: Option<NaiveDate>,      // 7
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
            })
            .loc();
        }

        // Closure para campos de data (Option<NaiveDate>)
        let get_date = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let num_proc = fields.get(2).to_compact_string();
        let id_sec_jud = fields.get(3).to_compact_string();
        let id_vara = fields.get(4).to_compact_string();
        let ind_nat_acao = fields.get(5).to_compact_string();
        let desc_dec_jud = fields.get(6).to_compact_string();
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
