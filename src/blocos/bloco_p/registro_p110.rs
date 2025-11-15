use crate::{EFDError, EFDResult, SpedParser, ToOptionalString, impl_sped_record_trait};
use std::path::Path;

#[derive(Debug)]
pub struct RegistroP110 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub num_campo: Option<String>, // 2
    pub cod_det: Option<String>,   // 3
    pub det_valor: Option<String>, // 4
    pub inf_compl: Option<String>, // 5
}

impl_sped_record_trait!(RegistroP110);

impl SpedParser for RegistroP110 {
    type Output = RegistroP110;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro P110 possui 5 campos de dados + 2 delimitadores = 7.
        if len != 7 {
            return Err(EFDError::InvalidLength {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 7,
                tamanho_encontrado: len,
            });
        }

        let num_campo = fields.get(2).to_optional_string();
        let cod_det = fields.get(3).to_optional_string();
        let det_valor = fields.get(4).to_optional_string();
        let inf_compl = fields.get(5).to_optional_string();

        let reg = RegistroP110 {
            nivel: 4,
            bloco: 'P',
            registro,
            line_number,
            num_campo,
            cod_det,
            det_valor,
            inf_compl,
        };

        Ok(reg)
    }
}
