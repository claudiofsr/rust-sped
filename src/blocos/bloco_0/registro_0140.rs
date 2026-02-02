use crate::{EFDError, EFDResult, ResultExt, SpedParser, StringParser, impl_reg_methods};
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "0140";

#[derive(Debug, Clone)]
pub struct Registro0140 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_est: Option<Arc<str>>, // 2
    pub nome: Option<Arc<str>>,    // 3
    pub cnpj: Option<Arc<str>>,    // 4
    pub uf: Option<Arc<str>>,      // 5
    pub ie: Option<Arc<str>>,      // 6
    pub cod_mun: Option<Arc<str>>, // 7
    pub im: Option<Arc<str>>,      // 8
    pub suframa: Option<Arc<str>>, // 9
}

impl_reg_methods!(Registro0140);

impl SpedParser for Registro0140 {
    type Output = Registro0140;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != 11 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 11,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let cod_est = fields.get(2).to_arc();
        let nome = fields.get(3).to_upper_arc(); // Normaliza nome para Uppercase
        let cnpj = fields.get(4).to_arc();
        let uf = fields.get(5).to_arc();
        let ie = fields.get(6).to_arc();
        let cod_mun = fields.get(7).to_arc();
        let im = fields.get(8).to_arc();
        let suframa = fields.get(9).to_arc();

        let reg = Registro0140 {
            nivel: 2,
            bloco: '0',
            registro: REGISTRO.into(),
            line_number,
            cod_est,
            nome,
            cnpj,
            uf,
            ie,
            cod_mun,
            im,
            suframa,
        };

        Ok(reg)
    }
}
