use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToNaiveDate, impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use std::path::Path;

const REGISTRO: &str = "C860";

#[derive(Debug, Clone)]
pub struct RegistroC860 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_mod: Option<CompactString>, // 2
    pub nr_sat: Option<CompactString>,  // 3
    pub dt_doc: Option<NaiveDate>,      // 4
    pub doc_ini: Option<CompactString>, // 5
    pub doc_fim: Option<CompactString>, // 6
}

impl_reg_methods!(RegistroC860);

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
            })
            .loc();
        }

        let get_date = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let cod_mod = fields.get(2).to_compact_string();
        let nr_sat = fields.get(3).to_compact_string();
        let dt_doc = get_date(4, "DT_DOC")?;
        let doc_ini = fields.get(5).to_compact_string();
        let doc_fim = fields.get(6).to_compact_string();

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
