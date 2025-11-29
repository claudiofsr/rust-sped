use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "M400";

#[derive(Debug, Clone)]
pub struct RegistroM400 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cst_pis: Option<String>,     // 2
    pub vl_tot_rec: Option<Decimal>, // 3
    pub cod_cta: Option<String>,     // 4
    pub desc_compl: Option<String>,  // 5
}

impl_sped_record_trait!(RegistroM400);

impl SpedParser for RegistroM400 {
    type Output = RegistroM400;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro M400 possui 5 campos de dados + 2 delimitadores = 7.
        if len != 7 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 7,
                tamanho_encontrado: len,
            });
        }

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cst_pis = fields.get(2).to_optional_string();
        let vl_tot_rec = get_decimal_field(3, "VL_TOT_REC")?;
        let cod_cta = fields.get(4).to_optional_string();
        let desc_compl = fields.get(5).to_optional_string();

        let reg = RegistroM400 {
            nivel: 2,
            bloco: 'M',
            registro: REGISTRO.to_string(),
            line_number,
            cst_pis,
            vl_tot_rec,
            cod_cta,
            desc_compl,
        };

        Ok(reg)
    }
}
