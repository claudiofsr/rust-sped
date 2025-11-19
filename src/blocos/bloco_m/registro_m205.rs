use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct RegistroM205 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub num_campo: Option<String>,  // 2
    pub cod_rec: Option<String>,    // 3
    pub vl_debito: Option<Decimal>, // 4
}

impl_sped_record_trait!(RegistroM205);

impl SpedParser for RegistroM205 {
    type Output = RegistroM205;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro M205 possui 4 campos de dados + 2 delimitadores = 6.
        if len != 6 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 6,
                tamanho_encontrado: len,
            });
        }

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let num_campo = fields.get(2).to_optional_string();
        let cod_rec = fields.get(3).to_optional_string();
        let vl_debito = get_decimal_field(4, "VL_DEBITO")?;

        let reg = RegistroM205 {
            nivel: 3,
            bloco: 'M',
            registro,
            line_number,
            num_campo,
            cod_rec,
            vl_debito,
        };

        Ok(reg)
    }
}
