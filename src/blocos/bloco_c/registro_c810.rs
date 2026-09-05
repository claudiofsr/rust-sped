use crate::{
    CodigoSituacaoTributaria, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal,
    impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C810";

#[derive(Debug, Clone)]
pub struct RegistroC810 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cfop: Option<u16>,                            // 2
    pub vl_item: Option<Decimal>,                     // 3
    pub cod_item: Option<CompactString>,              // 4
    pub cst_pis: Option<CodigoSituacaoTributaria>,    // 5
    pub vl_bc_pis: Option<Decimal>,                   // 6
    pub aliq_pis: Option<Decimal>,                    // 7
    pub vl_pis: Option<Decimal>,                      // 8
    pub cst_cofins: Option<CodigoSituacaoTributaria>, // 9
    pub vl_bc_cofins: Option<Decimal>,                // 10
    pub aliq_cofins: Option<Decimal>,                 // 11
    pub vl_cofins: Option<Decimal>,                   // 12
    pub cod_cta: Option<CompactString>,               // 13
}

impl_reg_methods!(RegistroC810);

impl SpedParser for RegistroC810 {
    type Output = RegistroC810;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C810 possui 13 campos de dados + 2 delimitadores = 15.
        if len != 15 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 15,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cfop = fields.get(2).parse_opt();
        let vl_item = get_decimal(3, "VL_ITEM")?;
        let cod_item = fields.get(4).to_compact_string();
        let cst_pis = fields
            .get(5)
            .to_efd_field(file_path, line_number, "CST_PIS")?;
        let vl_bc_pis = get_decimal(6, "VL_BC_PIS")?;
        let aliq_pis = get_decimal(7, "ALIQ_PIS")?;
        let vl_pis = get_decimal(8, "VL_PIS")?;
        let cst_cofins = fields
            .get(9)
            .to_efd_field(file_path, line_number, "CST_COFINS")?;
        let vl_bc_cofins = get_decimal(10, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(11, "ALIQ_COFINS")?;
        let vl_cofins = get_decimal(12, "VL_COFINS")?;
        let cod_cta = fields.get(13).to_compact_string();

        let reg = RegistroC810 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cfop,
            vl_item,
            cod_item,
            cst_pis,
            vl_bc_pis,
            aliq_pis,
            vl_pis,
            cst_cofins,
            vl_bc_cofins,
            aliq_cofins,
            vl_cofins,
            cod_cta,
        };

        Ok(reg)
    }
}
