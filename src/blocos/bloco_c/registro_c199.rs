use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C199";

#[derive(Debug, Clone)]
pub struct RegistroC199 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_doc_imp: Option<CompactString>, // 2
    pub num_doc_imp: Option<CompactString>, // 3
    pub vl_pis_imp: Option<Decimal>,        // 4
    pub vl_cofins_imp: Option<Decimal>,     // 5
    pub num_acdraw: Option<CompactString>,  // 6
}

impl_reg_methods!(RegistroC199);

impl SpedParser for RegistroC199 {
    type Output = RegistroC199;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C199 possui 6 campos de dados + 2 delimitadores = 8.
        if len != 8 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 8,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cod_doc_imp = fields.get(2).to_compact_string();
        let num_doc_imp = fields.get(3).to_compact_string();
        let vl_pis_imp = get_decimal(4, "VL_PIS_IMP")?;
        let vl_cofins_imp = get_decimal(5, "VL_COFINS_IMP")?;
        let num_acdraw = fields.get(6).to_compact_string();

        let reg = RegistroC199 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.into(),
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
