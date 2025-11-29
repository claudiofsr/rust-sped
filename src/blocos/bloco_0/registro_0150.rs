use crate::{EFDError, EFDResult, SpedParser, ToOptionalString, impl_sped_record_trait};
use std::path::Path;

const REGISTRO: &str = "0150";

#[derive(Debug, Clone)]
pub struct Registro0150 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_part: Option<String>, // 2
    pub nome: Option<String>,     // 3
    pub cod_pais: Option<String>, // 4
    pub cnpj: Option<String>,     // 5
    pub cpf: Option<String>,      // 6
    pub ie: Option<String>,       // 7
    pub cod_mun: Option<String>,  // 8
    pub suframa: Option<String>,  // 9
    pub end: Option<String>,      // 10
    pub num: Option<String>,      // 11
    pub compl: Option<String>,    // 12
    pub bairro: Option<String>,   // 13
}

impl_sped_record_trait!(Registro0150);

impl SpedParser for Registro0150 {
    type Output = Registro0150;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != 15 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 15,
                tamanho_encontrado: len,
            });
        }

        let cod_part = fields.get(2).to_optional_string();
        let nome = fields.get(3).to_optional_string();
        let cod_pais = fields.get(4).to_optional_string();
        let cnpj = fields.get(5).to_optional_string();
        let cpf = fields.get(6).to_optional_string();
        let ie = fields.get(7).to_optional_string();
        let cod_mun = fields.get(8).to_optional_string();
        let suframa = fields.get(9).to_optional_string();
        let end = fields.get(10).to_optional_string();
        let num = fields.get(11).to_optional_string();
        let compl = fields.get(12).to_optional_string();
        let bairro = fields.get(13).to_optional_string();

        let reg = Registro0150 {
            nivel: 3,
            bloco: '0',
            registro: REGISTRO.to_string(),
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
