use crate::{
    CodigoSituacaoTributaria, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal,
    impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C185";

#[derive(Debug, Clone)]
pub struct RegistroC185 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cst_cofins: Option<CodigoSituacaoTributaria>, // 2
    pub cfop: Option<u16>,                            // 3
    pub vl_item: Option<Decimal>,                     // 4
    pub vl_desc: Option<Decimal>,                     // 5
    pub vl_bc_cofins: Option<Decimal>,                // 6
    pub aliq_cofins: Option<Decimal>,                 // 7
    pub quant_bc_cofins: Option<CompactString>,       // 8
    pub aliq_cofins_quant: Option<Decimal>,           // 9
    pub vl_cofins: Option<Decimal>,                   // 10
    pub cod_cta: Option<CompactString>,               // 11
}

impl_reg_methods!(RegistroC185);

impl SpedParser for RegistroC185 {
    type Output = RegistroC185;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C185 possui 11 campos de dados + 2 delimitadores = 13.
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

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cst_cofins = fields
            .get(2)
            .to_efd_field(file_path, line_number, "CST_COFINS")?;
        let cfop = fields.get(3).parse_opt();
        let vl_item = get_decimal(4, "VL_ITEM")?;
        let vl_desc = get_decimal(5, "VL_DESC")?;
        let vl_bc_cofins = get_decimal(6, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(7, "ALIQ_COFINS")?;
        let quant_bc_cofins = fields.get(8).to_compact_string();
        let aliq_cofins_quant = get_decimal(9, "ALIQ_COFINS_QUANT")?;
        let vl_cofins = get_decimal(10, "VL_COFINS")?;
        let cod_cta = fields.get(11).to_compact_string();

        let reg = RegistroC185 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cst_cofins,
            cfop,
            vl_item,
            vl_desc,
            vl_bc_cofins,
            aliq_cofins,
            quant_bc_cofins,
            aliq_cofins_quant,
            vl_cofins,
            cod_cta,
        };

        Ok(reg)
    }
}
