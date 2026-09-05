use crate::{EFDError, EFDResult, ResultExt, SpedParser, StringParser, impl_reg_methods};
use compact_str::CompactString;
use std::path::Path;

const REGISTRO: &str = "A110";

#[derive(Debug, Clone)]
pub struct RegistroA110 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_inf: Option<CompactString>,   // 2
    pub txt_compl: Option<CompactString>, // 3
}

impl_reg_methods!(RegistroA110);

impl SpedParser for RegistroA110 {
    type Output = RegistroA110;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro A110 possui 3 campos de dados + 2 delimitadores = 5.
        if len != 5 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 5,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let cod_inf = fields.get(2).to_compact_string();
        let txt_compl = fields.get(3).to_compact_string();

        let reg = RegistroA110 {
            nivel: 4,
            bloco: 'A',
            registro: REGISTRO.into(),
            line_number,
            cod_inf,
            txt_compl,
        };

        Ok(reg)
    }
}
