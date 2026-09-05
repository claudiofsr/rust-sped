use crate::{EFDError, EFDResult, ResultExt, SpedParser, StringParser, impl_reg_methods};
use std::{path::Path, sync::Arc};

const EXPECTED_FIELDS: usize = 16;
const REGISTRO: &str = "0100";

#[derive(Debug, Clone)]
pub struct Registro0100 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub nome: Option<Arc<str>>,    // 2
    pub cpf: Option<Arc<str>>,     // 3
    pub crc: Option<Arc<str>>,     // 4
    pub cnpj: Option<Arc<str>>,    // 5
    pub cep: Option<Arc<str>>,     // 6
    pub end: Option<Arc<str>>,     // 7
    pub num: Option<Arc<str>>,     // 8
    pub compl: Option<Arc<str>>,   // 9
    pub bairro: Option<Arc<str>>,  // 10
    pub fone: Option<Arc<str>>,    // 11
    pub fax: Option<Arc<str>>,     // 12
    pub email: Option<Arc<str>>,   // 13
    pub cod_mun: Option<Arc<str>>, // 14
}

impl_reg_methods!(Registro0100);

impl SpedParser for Registro0100 {
    type Output = Registro0100;

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

        let nome = fields.get(2).to_arc();
        let cpf = fields.get(3).to_arc();
        let crc = fields.get(4).to_arc();
        let cnpj = fields.get(5).to_arc();
        let cep = fields.get(6).to_arc();
        let end = fields.get(7).to_arc();
        let num = fields.get(8).to_arc();
        let compl = fields.get(9).to_arc();
        let bairro = fields.get(10).to_arc();
        let fone = fields.get(11).to_arc();
        let fax = fields.get(12).to_arc();
        let email = fields.get(13).to_arc();
        let cod_mun = fields.get(14).to_arc();

        let reg = Registro0100 {
            nivel: 2,
            bloco: '0',
            line_number,
            registro: REGISTRO.into(),

            nome,
            cpf,
            crc,
            cnpj,
            cep,
            end,
            num,
            compl,
            bairro,
            fone,
            fax,
            email,
            cod_mun,
        };

        Ok(reg)
    }
}
