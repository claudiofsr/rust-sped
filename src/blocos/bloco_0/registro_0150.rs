use crate::{EFDError, EFDResult, ResultExt, SpedParser, StringParser, impl_reg_methods};
use std::{path::Path, sync::Arc};

const EXPECTED_FIELDS: usize = 15;
const REGISTRO: &str = "0150";

#[derive(Debug, Clone)]
pub struct Registro0150 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_part: Option<Arc<str>>, // 2
    pub nome: Option<Arc<str>>,     // 3
    pub cod_pais: Option<Arc<str>>, // 4
    pub cnpj: Option<Arc<str>>,     // 5
    pub cpf: Option<Arc<str>>,      // 6
    pub ie: Option<Arc<str>>,       // 7
    pub cod_mun: Option<Arc<str>>,  // 8
    pub suframa: Option<Arc<str>>,  // 9
    pub end: Option<Arc<str>>,      // 10
    pub num: Option<Arc<str>>,      // 11
    pub compl: Option<Arc<str>>,    // 12
    pub bairro: Option<Arc<str>>,   // 13
}

impl_reg_methods!(Registro0150);

impl SpedParser for Registro0150 {
    type Output = Registro0150;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != EXPECTED_FIELDS {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: EXPECTED_FIELDS,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let cod_part = fields.get(2).to_arc();
        let nome = fields.get(3).to_arc();
        let cod_pais = fields.get(4).to_arc();
        let cnpj = fields.get(5).to_arc();
        let cpf = fields.get(6).to_arc();
        let ie = fields.get(7).to_arc();
        let cod_mun = fields.get(8).to_arc();
        let suframa = fields.get(9).to_arc();
        let end = fields.get(10).to_arc();
        let num = fields.get(11).to_arc();
        let compl = fields.get(12).to_arc();
        let bairro = fields.get(13).to_arc();

        let reg = Registro0150 {
            nivel: 3,
            bloco: '0',
            registro: REGISTRO.into(),
            line_number,
            cod_part,
            nome,
            cod_pais,
            cnpj,
            cpf,
            ie,
            cod_mun,
            suframa,
            end,
            num,
            compl,
            bairro,
        };

        Ok(reg)
    }
}
