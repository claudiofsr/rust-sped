use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "M410";

#[derive(Debug, Clone)]
pub struct RegistroM410 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub nat_rec: Option<CompactString>,    // 2
    pub vl_rec: Option<Decimal>,           // 3
    pub cod_cta: Option<CompactString>,    // 4
    pub desc_compl: Option<CompactString>, // 5
}

impl_reg_methods!(RegistroM410);

impl SpedParser for RegistroM410 {
    type Output = RegistroM410;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro M410 possui 5 campos de dados + 2 delimitadores = 7.
        if len != 7 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 7,
                tamanho_encontrado: len,
            })
            .loc();
        }

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let nat_rec = fields.get(2).to_compact_string();
        let vl_rec = get_decimal(3, "VL_REC")?;
        let cod_cta = fields.get(4).to_compact_string();
        let desc_compl = fields.get(5).to_compact_string();

        let reg = RegistroM410 {
            nivel: 3,
            bloco: 'M',
            registro: REGISTRO.into(),
            line_number,
            nat_rec,
            vl_rec,
            cod_cta,
            desc_compl,
        };

        Ok(reg)
    }
}
