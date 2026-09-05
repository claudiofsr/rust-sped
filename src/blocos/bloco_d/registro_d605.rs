use crate::{
    CodigoSituacaoTributaria, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal,
    impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "D605";

#[derive(Debug, Clone)]
pub struct RegistroD605 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_class: Option<CompactString>,             // 2
    pub vl_item: Option<Decimal>,                     // 3
    pub vl_desc: Option<Decimal>,                     // 4
    pub cst_cofins: Option<CodigoSituacaoTributaria>, // 5
    pub vl_bc_cofins: Option<Decimal>,                // 6
    pub aliq_cofins: Option<Decimal>,                 // 7
    pub vl_cofins: Option<Decimal>,                   // 8
    pub cod_cta: Option<CompactString>,               // 9
}

impl_reg_methods!(RegistroD605);

impl SpedParser for RegistroD605 {
    type Output = RegistroD605;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro D605 possui 9 campos de dados + 2 delimitadores = 11.
        if len != 11 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 11,
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

        let cod_class = fields.get(2).to_compact_string();
        let vl_item = get_decimal(3, "VL_ITEM")?;
        let vl_desc = get_decimal(4, "VL_DESC")?;
        let cst_cofins = fields
            .get(5)
            .to_efd_field(file_path, line_number, "CST_COFINS")?;
        let vl_bc_cofins = get_decimal(6, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(7, "ALIQ_COFINS")?;
        let vl_cofins = get_decimal(8, "VL_COFINS")?;
        let cod_cta = fields.get(9).to_compact_string();

        let reg = RegistroD605 {
            nivel: 4,
            bloco: 'D',
            registro: REGISTRO.into(),
            line_number,
            cod_class,
            vl_item,
            vl_desc,
            cst_cofins,
            vl_bc_cofins,
            aliq_cofins,
            vl_cofins,
            cod_cta,
        };

        Ok(reg)
    }
}
