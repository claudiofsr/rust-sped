use crate::{EFDError, EFDResult, SpedParser, ToOptionalString, impl_sped_record_trait};
use std::path::Path;

#[derive(Debug)]
pub struct RegistroI300 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_comp: Option<String>,   // 2
    pub det_valor: Option<String>,  // 3
    pub cod_cta: Option<String>,    // 4
    pub info_compl: Option<String>, // 5
}

impl_sped_record_trait!(RegistroI300);

impl SpedParser for RegistroI300 {
    type Output = RegistroI300;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro I300 possui 5 campos de dados + 2 delimitadores = 7.
        if len != 7 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 7,
                tamanho_encontrado: len,
            });
        }

        let cod_comp = fields.get(2).to_optional_string();
        let det_valor = fields.get(3).to_optional_string();
        let cod_cta = fields.get(4).to_optional_string();
        let info_compl = fields.get(5).to_optional_string();

        let reg = RegistroI300 {
            nivel: 5,
            bloco: 'I',
            registro,
            line_number,
            cod_comp,
            det_valor,
            cod_cta,
            info_compl,
        };

        Ok(reg)
    }
}
