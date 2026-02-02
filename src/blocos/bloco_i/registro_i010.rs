use crate::{EFDError, EFDResult, ResultExt, SpedParser, StringParser, impl_reg_methods};
use compact_str::CompactString;
use std::path::Path;

const REGISTRO: &str = "I010";

#[derive(Debug, Clone)]
pub struct RegistroI010 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cnpj: Option<CompactString>,       // 2
    pub ind_ativ: Option<CompactString>,   // 3
    pub info_compl: Option<CompactString>, // 4
}

impl_reg_methods!(RegistroI010);

impl SpedParser for RegistroI010 {
    type Output = RegistroI010;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro I010 possui 4 campos de dados + 2 delimitadores = 6.
        if len != 6 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 6,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let cnpj = fields.get(2).to_compact_string();
        let ind_ativ = fields.get(3).to_compact_string();
        let info_compl = fields.get(4).to_compact_string();

        let reg = RegistroI010 {
            nivel: 2,
            bloco: 'I',
            registro: REGISTRO.into(),
            line_number,
            cnpj,
            ind_ativ,
            info_compl,
        };

        Ok(reg)
    }
}
