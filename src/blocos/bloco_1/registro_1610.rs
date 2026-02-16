use crate::{
    CodigoSituacaoTributaria, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal,
    ToNaiveDate, impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "1610";

#[derive(Debug, Clone)]
pub struct Registro1610 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cnpj: Option<CompactString>,                  // 2
    pub cst_cofins: Option<CodigoSituacaoTributaria>, // 3
    pub cod_part: Option<CompactString>,              // 4
    pub dt_oper: Option<NaiveDate>,                   // 5
    pub vl_oper: Option<Decimal>,                     // 6
    pub vl_bc_cofins: Option<Decimal>,                // 7
    pub aliq_cofins: Option<Decimal>,                 // 8
    pub vl_cofins: Option<Decimal>,                   // 9
    pub cod_cta: Option<CompactString>,               // 10
    pub desc_compl: Option<CompactString>,            // 11
}

impl_reg_methods!(Registro1610);

impl SpedParser for Registro1610 {
    type Output = Registro1610;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro 1610 possui 11 campos de dados + 2 delimitadores = 13.
        if len != 13 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 13,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_date = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cnpj = fields.get(2).to_compact_string();
        let cst_cofins = fields
            .get(3)
            .to_efd_field(file_path, line_number, "CST_COFINS")?;
        let cod_part = fields.get(4).to_compact_string();
        let dt_oper = get_date(5, "DT_OPER")?;
        let vl_oper = get_decimal(6, "VL_OPER")?;
        let vl_bc_cofins = get_decimal(7, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(8, "ALIQ_COFINS")?;
        let vl_cofins = get_decimal(9, "VL_COFINS")?;
        let cod_cta = fields.get(10).to_compact_string();
        let desc_compl = fields.get(11).to_compact_string();

        let reg = Registro1610 {
            nivel: 3,
            bloco: '1',
            registro: REGISTRO.into(),
            line_number,
            cnpj,
            cst_cofins,
            cod_part,
            dt_oper,
            vl_oper,
            vl_bc_cofins,
            aliq_cofins,
            vl_cofins,
            cod_cta,
            desc_compl,
        };

        Ok(reg)
    }
}
