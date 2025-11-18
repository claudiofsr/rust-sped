use crate::{
    EFDError, EFDResult, SpedParser, ToNaiveDate, ToOptionalString, impl_sped_record_trait,
};
use chrono::NaiveDate;
use std::path::Path;

#[derive(Debug)]
pub struct Registro1010 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub num_proc: Option<String>,       // 2
    pub id_sec_jud: Option<String>,     // 3
    pub id_vara: Option<String>,        // 4
    pub ind_nat_acao: Option<String>,   // 5
    pub desc_dec_jud: Option<String>,   // 6
    pub dt_sent_jud: Option<NaiveDate>, // 7
}

impl_sped_record_trait!(Registro1010);

impl SpedParser for Registro1010 {
    type Output = Registro1010;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro 1010 possui 7 campos de dados + 2 delimitadores = 9.
        if len != 9 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 9,
                tamanho_encontrado: len,
            });
        }

        // Closure para campos de data (Option<NaiveDate>)
        let get_date_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path.to_path_buf(), line_number, field_name)
        };

        let num_proc = fields.get(2).to_optional_string();
        let id_sec_jud = fields.get(3).to_optional_string();
        let id_vara = fields.get(4).to_optional_string();
        let ind_nat_acao = fields.get(5).to_optional_string();
        let desc_dec_jud = fields.get(6).to_optional_string();
        let dt_sent_jud = get_date_field(7, "DT_SENT_JUD")?;

        let reg = Registro1010 {
            nivel: 2,
            bloco: '1',
            registro,
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
