use crate::{EFDError, EFDResult, SpedParser, ToOptionalString, impl_sped_record_trait};
use std::path::Path;

const EXPECTED_FIELDS: usize = 16;
const REGISTRO: &str = "0100";

#[derive(Debug, Clone)]
pub struct Registro0100 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub nome: Option<String>,    // 2
    pub cpf: Option<String>,     // 3
    pub crc: Option<String>,     // 4
    pub cnpj: Option<String>,    // 5
    pub cep: Option<String>,     // 6
    pub end: Option<String>,     // 7
    pub num: Option<String>,     // 8
    pub compl: Option<String>,   // 9
    pub bairro: Option<String>,  // 10
    pub fone: Option<String>,    // 11
    pub fax: Option<String>,     // 12
    pub email: Option<String>,   // 13
    pub cod_mun: Option<String>, // 14
}

impl_sped_record_trait!(Registro0100);

impl SpedParser for Registro0100 {
    type Output = Registro0100;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != EXPECTED_FIELDS {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: EXPECTED_FIELDS,
                tamanho_encontrado: len,
            });
        }

        let nome = fields.get(2).to_optional_string();
        let cpf = fields.get(3).to_optional_string();
        let crc = fields.get(4).to_optional_string();
        let cnpj = fields.get(5).to_optional_string();
        let cep = fields.get(6).to_optional_string();
        let end = fields.get(7).to_optional_string();
        let num = fields.get(8).to_optional_string();
        let compl = fields.get(9).to_optional_string();
        let bairro = fields.get(10).to_optional_string();
        let fone = fields.get(11).to_optional_string();
        let fax = fields.get(12).to_optional_string();
        let email = fields.get(13).to_optional_string();
        let cod_mun = fields.get(14).to_optional_string();

        let reg = Registro0100 {
            nivel: 2,
            bloco: '0',
            line_number,
            registro: REGISTRO.to_string(),

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
