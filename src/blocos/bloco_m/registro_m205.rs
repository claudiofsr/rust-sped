use crate::{EFDError, EFDResult, SpedParser, StringParser, ToDecimal, impl_reg_methods};
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "M205";

#[derive(Debug, Clone)]
pub struct RegistroM205 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub num_campo: Option<Arc<str>>, // 2
    pub cod_rec: Option<Arc<str>>,   // 3
    pub vl_debito: Option<Decimal>,  // 4
}

impl_reg_methods!(RegistroM205);

impl SpedParser for RegistroM205 {
    type Output = RegistroM205;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro M205 possui 4 campos de dados + 2 delimitadores = 6.
        if len != 6 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 6,
                tamanho_encontrado: len,
            });
        }

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let num_campo = fields.get(2).to_arc();
        let cod_rec = fields.get(3).to_arc();
        let vl_debito = get_decimal(4, "VL_DEBITO")?;

        let reg = RegistroM205 {
            nivel: 3,
            bloco: 'M',
            registro: REGISTRO.into(),
            line_number,
            num_campo,
            cod_rec,
            vl_debito,
        };

        Ok(reg)
    }
}
