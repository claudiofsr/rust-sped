use crate::{EFDError, EFDResult, SpedParser, impl_reg_methods};
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
            });
        }

        let cod_comp = fields.get(2).map(|&s| s.into());
        let det_valor = fields.get(3).map(|&s| s.into());
        let cod_cta = fields.get(4).map(|&s| s.into());
        let info_compl = fields.get(5).map(|&s| s.into());

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
