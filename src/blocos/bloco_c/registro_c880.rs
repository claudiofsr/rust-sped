use crate::{
    CodigoSituacaoTributaria, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal,
    impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C880";

#[derive(Debug, Clone)]
pub struct RegistroC880 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_item: Option<CompactString>,              // 2
    pub cfop: Option<u16>,                            // 3
    pub vl_item: Option<Decimal>,                     // 4
    pub vl_desc: Option<Decimal>,                     // 5
    pub cst_pis: Option<CodigoSituacaoTributaria>,    // 6
    pub quant_bc_pis: Option<CompactString>,          // 7 (Pode ser String ou Decimal)
    pub aliq_pis_quant: Option<Decimal>,              // 8
    pub vl_pis: Option<Decimal>,                      // 9
    pub cst_cofins: Option<CodigoSituacaoTributaria>, // 10
    pub quant_bc_cofins: Option<CompactString>,       // 11 (Pode ser String ou Decimal)
    pub aliq_cofins_quant: Option<Decimal>,           // 12
    pub vl_cofins: Option<Decimal>,                   // 13
    pub cod_cta: Option<CompactString>,               // 14
}

impl_reg_methods!(RegistroC880);

impl SpedParser for RegistroC880 {
    type Output = RegistroC880;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C880 possui 14 campos de dados + 2 delimitadores = 16.
        if len != 16 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 16,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cod_item = fields.get(2).to_compact_string();
        let cfop = fields.get(3).parse_opt();
        let vl_item = get_decimal(4, "VL_ITEM")?;
        let vl_desc = get_decimal(5, "VL_DESC")?;
        let cst_pis = fields
            .get(6)
            .to_efd_field(file_path, line_number, "CST_PIS")?;
        let quant_bc_pis = fields.get(7).to_compact_string();
        let aliq_pis_quant = get_decimal(8, "ALIQ_PIS_QUANT")?;
        let vl_pis = get_decimal(9, "VL_PIS")?;
        let cst_cofins = fields
            .get(10)
            .to_efd_field(file_path, line_number, "CST_COFINS")?;
        let quant_bc_cofins = fields.get(11).to_compact_string();
        let aliq_cofins_quant = get_decimal(12, "ALIQ_COFINS_QUANT")?;
        let vl_cofins = get_decimal(13, "VL_COFINS")?;
        let cod_cta = fields.get(14).to_compact_string();

        let reg = RegistroC880 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cod_item,
            cfop,
            vl_item,
            vl_desc,
            cst_pis,
            quant_bc_pis,
            aliq_pis_quant,
            vl_pis,
            cst_cofins,
            quant_bc_cofins,
            aliq_cofins_quant,
            vl_cofins,
            cod_cta,
        };

        Ok(reg)
    }
}
