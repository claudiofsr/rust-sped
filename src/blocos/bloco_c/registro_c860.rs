use crate::{EFDError, EFDResult, SpedParser, StringParser, ToNaiveDate, impl_sped_record_trait};
use chrono::NaiveDate;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "C860";

#[derive(Debug, Clone)]
pub struct RegistroC860 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_mod: Option<Arc<str>>, // 2
    pub nr_sat: Option<Arc<str>>,  // 3
    pub dt_doc: Option<NaiveDate>, // 4
    pub doc_ini: Option<Arc<str>>, // 5
    pub doc_fim: Option<Arc<str>>, // 6
}

impl_sped_record_trait!(RegistroC860);

impl SpedParser for RegistroC860 {
    type Output = RegistroC860;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C860 possui 6 campos de dados + 2 delimitadores = 8.
        if len != 8 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 8,
                tamanho_encontrado: len,
            });
        }

        let get_date = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let cod_mod = fields.get(2).to_arc();
        let nr_sat = fields.get(3).to_arc();
        let dt_doc = get_date(4, "DT_DOC")?;
        let doc_ini = fields.get(5).to_arc();
        let doc_fim = fields.get(6).to_arc();

        let reg = RegistroC860 {
            nivel: 3,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cod_mod,
            nr_sat,
            dt_doc,
            doc_ini,
            doc_fim,
        };

        Ok(reg)
    }
}
