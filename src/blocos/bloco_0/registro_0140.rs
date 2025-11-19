use crate::{EFDError, EFDResult, SpedParser, ToOptionalString, impl_sped_record_trait};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Registro0140 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_est: Option<String>, // 2
    pub nome: Option<String>,    // 3
    pub cnpj: Option<String>,    // 4
    pub uf: Option<String>,      // 5
    pub ie: Option<String>,      // 6
    pub cod_mun: Option<String>, // 7
    pub im: Option<String>,      // 8
    pub suframa: Option<String>, // 9
}

impl_sped_record_trait!(Registro0140);

impl SpedParser for Registro0140 {
    type Output = Registro0140;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        if len != 11 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 11,
                tamanho_encontrado: len,
            });
        }

        let cod_est = fields.get(2).to_optional_string();
        let nome = fields.get(3).to_optional_string();
        let cnpj = fields.get(4).to_optional_string();
        let uf = fields.get(5).to_optional_string();
        let ie = fields.get(6).to_optional_string();
        let cod_mun = fields.get(7).to_optional_string();
        let im = fields.get(8).to_optional_string();
        let suframa = fields.get(9).to_optional_string();

        let reg = Registro0140 {
            nivel: 2,
            bloco: '0',
            registro,
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
