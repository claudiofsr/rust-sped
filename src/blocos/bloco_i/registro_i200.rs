use crate::{EFDError, EFDResult, SpedParser, ToOptionalString, impl_sped_record_trait};
use std::path::Path;

#[derive(Debug)]
pub struct RegistroI200 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub num_campo: Option<String>,  // 2
    pub cod_det: Option<String>,    // 3
    pub det_valor: Option<String>,  // 4
    pub cod_cta: Option<String>,    // 5
    pub info_compl: Option<String>, // 6
}

impl_sped_record_trait!(RegistroI200);

impl SpedParser for RegistroI200 {
    type Output = RegistroI200;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro I200 possui 6 campos de dados + 2 delimitadores = 8.
        if len != 8 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 8,
                tamanho_encontrado: len,
            });
        }

        let num_campo = fields.get(2).to_optional_string();
        let cod_det = fields.get(3).to_optional_string();
        let det_valor = fields.get(4).to_optional_string();
        let cod_cta = fields.get(5).to_optional_string();
        let info_compl = fields.get(6).to_optional_string();

        let reg = RegistroI200 {
            nivel: 4,
            bloco: 'I',
            registro,
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
