use crate::{EFDError, EFDResult, ResultExt, SpedParser, StringParser, impl_reg_methods};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "0208";

#[derive(Debug, Clone)]
pub struct Registro0208 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_tab: Option<Arc<str>>,   // 2
    pub cod_gru: Option<Arc<str>>,   // 3 (corrigido do HashMap que tinha 2x o índice 2)
    pub marca_com: Option<Arc<str>>, // 4 (corrigido do HashMap que tinha 2x o índice 2)
}

impl_reg_methods!(Registro0208);

impl SpedParser for Registro0208 {
    type Output = Registro0208;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // Considerando os campos corrigidos: REG, COD_TAB, COD_GRU, MARCA_COM + delimitadores
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

        let cod_tab = fields.get(2).to_arc();
        let cod_gru = fields.get(3).to_arc();
        let marca_com = fields.get(4).to_arc();

        let reg = Registro0208 {
            nivel: 4,
            bloco: '0',
            registro: REGISTRO.into(),
            line_number,
            cod_tab,
            cod_gru,
            marca_com,
        };

        Ok(reg)
    }
}
