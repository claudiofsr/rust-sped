use crate::{
    CodigoDoCredito, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToCodigoDoCredito,
    ToDecimal, impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "M230";

#[derive(Debug, Clone)]
pub struct RegistroM230 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cnpj: Option<CompactString>,       // 2
    pub vl_vend: Option<Decimal>,          // 3
    pub vl_nao_receb: Option<Decimal>,     // 4
    pub vl_cont_dif: Option<Decimal>,      // 5
    pub vl_cred_dif: Option<Decimal>,      // 6
    pub cod_cred: Option<CodigoDoCredito>, // 7
}

impl_reg_methods!(RegistroM230);

impl SpedParser for RegistroM230 {
    type Output = RegistroM230;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro M230 possui 7 campos de dados + 2 delimitadores = 9.
        if len != 9 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 9,
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

        let cnpj = fields.get(2).to_compact_string();
        let vl_vend = get_decimal(3, "VL_VEND")?;
        let vl_nao_receb = get_decimal(4, "VL_NAO_RECEB")?;
        let vl_cont_dif = get_decimal(5, "VL_CONT_DIF")?;
        let vl_cred_dif = get_decimal(6, "VL_CRED_DIF")?;
        let cod_cred = fields
            .get(7)
            .to_codigo_do_credito(file_path, line_number, "COD_CRED")?;

        let reg = RegistroM230 {
            nivel: 4,
            bloco: 'M',
            registro: REGISTRO.into(),
            line_number,
            cnpj,
            vl_vend,
            vl_nao_receb,
            vl_cont_dif,
            vl_cred_dif,
            cod_cred,
        };

        Ok(reg)
    }
}
