use crate::{
    CodigoSituacaoTributaria, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal,
    impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "F500";

#[derive(Debug, Clone)]
pub struct RegistroF500 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub vl_rec_caixa: Option<Decimal>,                // 2
    pub cst_pis: Option<CodigoSituacaoTributaria>,    // 3
    pub vl_desc_pis: Option<Decimal>,                 // 4
    pub vl_bc_pis: Option<Decimal>,                   // 5
    pub aliq_pis: Option<Decimal>,                    // 6
    pub vl_pis: Option<Decimal>,                      // 7
    pub cst_cofins: Option<CodigoSituacaoTributaria>, // 8
    pub vl_desc_cofins: Option<Decimal>,              // 9
    pub vl_bc_cofins: Option<Decimal>,                // 10
    pub aliq_cofins: Option<Decimal>,                 // 11
    pub vl_cofins: Option<Decimal>,                   // 12
    pub cod_mod: Option<CompactString>,               // 13
    pub cfop: Option<u16>,                            // 14
    pub cod_cta: Option<CompactString>,               // 15
    pub info_compl: Option<CompactString>,            // 16
}

impl_reg_methods!(RegistroF500);

impl SpedParser for RegistroF500 {
    type Output = RegistroF500;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro F500 possui 16 campos de dados + 2 delimitadores = 18.
        if len != 18 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 18,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let vl_rec_caixa = get_decimal(2, "VL_REC_CAIXA")?;
        let cst_pis = fields
            .get(3)
            .to_efd_field(file_path, line_number, "CST_PIS")?;
        let vl_desc_pis = get_decimal(4, "VL_DESC_PIS")?;
        let vl_bc_pis = get_decimal(5, "VL_BC_PIS")?;
        let aliq_pis = get_decimal(6, "ALIQ_PIS")?;
        let vl_pis = get_decimal(7, "VL_PIS")?;
        let cst_cofins = fields
            .get(8)
            .to_efd_field(file_path, line_number, "CST_COFINS")?;
        let vl_desc_cofins = get_decimal(9, "VL_DESC_COFINS")?;
        let vl_bc_cofins = get_decimal(10, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(11, "ALIQ_COFINS")?;
        let vl_cofins = get_decimal(12, "VL_COFINS")?;
        let cod_mod = fields.get(13).to_compact_string();
        let cfop = fields.get(14).parse_opt();
        let cod_cta = fields.get(15).to_compact_string();
        let info_compl = fields.get(16).to_compact_string();

        let reg = RegistroF500 {
            nivel: 3,
            bloco: 'F',
            registro: REGISTRO.into(),
            line_number,
            vl_rec_caixa,
            cst_pis,
            vl_desc_pis,
            vl_bc_pis,
            aliq_pis,
            vl_pis,
            cst_cofins,
            vl_desc_cofins,
            vl_bc_cofins,
            aliq_cofins,
            vl_cofins,
            cod_mod,
            cfop,
            cod_cta,
            info_compl,
        };

        Ok(reg)
    }
}
