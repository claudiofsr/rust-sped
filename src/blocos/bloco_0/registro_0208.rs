use crate::{EFDError, EFDResult, SpedParser, ToOptionalString, impl_sped_record_trait};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Registro0208 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_tab: Option<String>,   // 2
    pub cod_gru: Option<String>,   // 3 (corrigido do HashMap que tinha 2x o índice 2)
    pub marca_com: Option<String>, // 4 (corrigido do HashMap que tinha 2x o índice 2)
}

impl_sped_record_trait!(Registro0208);

impl SpedParser for Registro0208 {
    type Output = Registro0208;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // Considerando os campos corrigidos: REG, COD_TAB, COD_GRU, MARCA_COM + delimitadores
        if len != 6 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 6,
                tamanho_encontrado: len,
            });
        }

        let cod_tab = fields.get(2).to_optional_string();
        let cod_gru = fields.get(3).to_optional_string();
        let marca_com = fields.get(4).to_optional_string();

        let reg = Registro0208 {
            nivel: 4,
            bloco: '0',
            registro,
            line_number,
            cod_tab,
            cod_gru,
            marca_com,
        };

        Ok(reg)
    }
}
