use crate::{EFDError, EFDResult, SpedParser, StringParser, ToDecimal, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "C199";

#[derive(Debug, Clone)]
pub struct RegistroC199 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_doc_imp: Option<Arc<str>>,  // 2
    pub num_doc_imp: Option<Arc<str>>,  // 3
    pub vl_pis_imp: Option<Decimal>,    // 4
    pub vl_cofins_imp: Option<Decimal>, // 5
    pub num_acdraw: Option<Arc<str>>,   // 6
}

impl_sped_record_trait!(RegistroC199);

impl SpedParser for RegistroC199 {
    type Output = RegistroC199;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C199 possui 6 campos de dados + 2 delimitadores = 8.
        if len != 8 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 8,
                tamanho_encontrado: len,
            });
        }

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cod_doc_imp = fields.get(2).to_arc();
        let num_doc_imp = fields.get(3).to_arc();
        let vl_pis_imp = get_decimal_field(4, "VL_PIS_IMP")?;
        let vl_cofins_imp = get_decimal_field(5, "VL_COFINS_IMP")?;
        let num_acdraw = fields.get(6).to_arc();

        let reg = RegistroC199 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.to_string(),
            line_number,
            cod_doc_imp,
            num_doc_imp,
            vl_pis_imp,
            vl_cofins_imp,
            num_acdraw,
        };

        Ok(reg)
    }
}
