use crate::{EFDError, EFDResult, ResultExt, SpedParser, StringParser, impl_reg_methods};
use compact_str::CompactString;
use std::path::Path;

const REGISTRO: &str = "I200";

#[derive(Debug, Clone)]
pub struct RegistroI200 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub num_campo: Option<CompactString>,  // 2
    pub cod_det: Option<CompactString>,    // 3
    pub det_valor: Option<CompactString>,  // 4
    pub cod_cta: Option<CompactString>,    // 5
    pub info_compl: Option<CompactString>, // 6
}

impl_reg_methods!(RegistroI200);

impl SpedParser for RegistroI200 {
    type Output = RegistroI200;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro I200 possui 6 campos de dados + 2 delimitadores = 8.
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

        let num_campo = fields.get(2).to_compact_string();
        let cod_det = fields.get(3).to_compact_string();
        let det_valor = fields.get(4).to_compact_string();
        let cod_cta = fields.get(5).to_compact_string();
        let info_compl = fields.get(6).to_compact_string();

        let reg = RegistroI200 {
            nivel: 4,
            bloco: 'I',
            registro: REGISTRO.into(),
            line_number,
            num_campo,
            cod_det,
            det_valor,
            cod_cta,
            info_compl,
        };

        Ok(reg)
    }
}
