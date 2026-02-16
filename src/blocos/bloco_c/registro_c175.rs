use crate::{
    CodigoSituacaoTributaria, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal,
    impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C175";

#[derive(Debug, Clone)]
pub struct RegistroC175 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cfop: Option<u16>,                            // 2
    pub vl_opr: Option<Decimal>,                      // 3
    pub vl_desc: Option<Decimal>,                     // 4
    pub cst_pis: Option<CodigoSituacaoTributaria>,    // 5
    pub vl_bc_pis: Option<Decimal>,                   // 6
    pub aliq_pis: Option<Decimal>,                    // 7
    pub quant_bc_pis: Option<CompactString>,          // 8
    pub aliq_pis_quant: Option<Decimal>,              // 9
    pub vl_pis: Option<Decimal>,                      // 10
    pub cst_cofins: Option<CodigoSituacaoTributaria>, // 11
    pub vl_bc_cofins: Option<Decimal>,                // 12
    pub aliq_cofins: Option<Decimal>,                 // 13
    pub quant_bc_cofins: Option<CompactString>,       // 14
    pub aliq_cofins_quant: Option<Decimal>,           // 15
    pub vl_cofins: Option<Decimal>,                   // 16
    pub cod_cta: Option<CompactString>,               // 17
    pub info_compl: Option<CompactString>,            // 18
}

impl_reg_methods!(RegistroC175);

impl SpedParser for RegistroC175 {
    type Output = RegistroC175;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C175 possui 18 campos de dados + 2 delimitadores = 20.
        if len != 20 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 20,
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
        let vl_opr = get_decimal(3, "VL_OPR")?;
        let vl_desc = get_decimal(4, "VL_DESC")?;
        let cst_pis = fields
            .get(5)
            .to_efd_field(file_path, line_number, "CST_PIS")?;
        let vl_bc_pis = get_decimal(6, "VL_BC_PIS")?;
        let aliq_pis = get_decimal(7, "ALIQ_PIS")?;
        let quant_bc_pis = fields.get(8).to_compact_string();
        let aliq_pis_quant = get_decimal(9, "ALIQ_PIS_QUANT")?;
        let vl_pis = get_decimal(10, "VL_PIS")?;
        let cst_cofins = fields
            .get(11)
            .to_efd_field(file_path, line_number, "CST_COFINS")?;
        let vl_bc_cofins = get_decimal(12, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(13, "ALIQ_COFINS")?;
        let quant_bc_cofins = fields.get(14).to_compact_string();
        let aliq_cofins_quant = get_decimal(15, "ALIQ_COFINS_QUANT")?;
        let vl_cofins = get_decimal(16, "VL_COFINS")?;
        let cod_cta = fields.get(17).to_compact_string();
        let info_compl = fields.get(18).to_compact_string();

        let reg = RegistroC175 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cfop,
            vl_opr,
            vl_desc,
            cst_pis,
            vl_bc_pis,
            aliq_pis,
            quant_bc_pis,
            aliq_pis_quant,
            vl_pis,
            cst_cofins,
            vl_bc_cofins,
            aliq_cofins,
            quant_bc_cofins,
            aliq_cofins_quant,
            vl_cofins,
            cod_cta,
            info_compl,
        };

        Ok(reg)
    }
}
