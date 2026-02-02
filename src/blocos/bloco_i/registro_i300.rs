use crate::{EFDError, EFDResult, ResultExt, SpedParser, StringParser, impl_reg_methods};
use compact_str::CompactString;
use std::path::Path;

const REGISTRO: &str = "I300";

#[derive(Debug, Clone)]
pub struct RegistroI300 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_comp: Option<CompactString>,   // 2
    pub det_valor: Option<CompactString>,  // 3
    pub cod_cta: Option<CompactString>,    // 4
    pub info_compl: Option<CompactString>, // 5
}

impl_reg_methods!(RegistroI300);

impl SpedParser for RegistroI300 {
    type Output = RegistroI300;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro I300 possui 5 campos de dados + 2 delimitadores = 7.
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

        let cod_comp = fields.get(2).to_compact_string();
        let det_valor = fields.get(3).to_compact_string();
        let cod_cta = fields.get(4).to_compact_string();
        let info_compl = fields.get(5).to_compact_string();

        let reg = RegistroI300 {
            nivel: 5,
            bloco: 'I',
            registro: REGISTRO.into(),
            line_number,
            cod_comp,
            det_valor,
            cod_cta,
            info_compl,
        };

        Ok(reg)
    }
}
