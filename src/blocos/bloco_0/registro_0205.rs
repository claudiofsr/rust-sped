use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToNaiveDate, impl_reg_methods,
};
use chrono::NaiveDate;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "0205";

#[derive(Debug, Clone)]
pub struct Registro0205 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub descr_ant_item: Option<Arc<str>>, // 2
    pub dt_ini: Option<NaiveDate>,        // 3
    pub dt_fim: Option<NaiveDate>,        // 4
    pub cod_ant_item: Option<Arc<str>>,   // 5
}

impl_reg_methods!(Registro0205);

impl SpedParser for Registro0205 {
    type Output = Registro0205;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != 7 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 7,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_date = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let descr_ant_item = fields.get(2).to_arc();
        let dt_ini = get_date(3, "DT_INI")?;
        let dt_fim = get_date(4, "DT_FIM")?;
        let cod_ant_item = fields.get(5).to_arc();

        let reg = Registro0205 {
            nivel: 4,
            bloco: '0',
            registro: REGISTRO.into(),
            line_number,
            descr_ant_item,
            dt_ini,
            dt_fim,
            cod_ant_item,
        };

        Ok(reg)
    }
}
