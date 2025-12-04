use crate::{EFDError, EFDResult, SpedParser, StringParser, ToDecimal, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "C175";

#[derive(Debug, Clone)]
pub struct RegistroC175 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cfop: Option<u16>,                  // 2
    pub vl_opr: Option<Decimal>,            // 3
    pub vl_desc: Option<Decimal>,           // 4
    pub cst_pis: Option<u16>,               // 5
    pub vl_bc_pis: Option<Decimal>,         // 6
    pub aliq_pis: Option<Decimal>,          // 7
    pub quant_bc_pis: Option<Arc<str>>,     // 8
    pub aliq_pis_quant: Option<Decimal>,    // 9
    pub vl_pis: Option<Decimal>,            // 10
    pub cst_cofins: Option<u16>,            // 11
    pub vl_bc_cofins: Option<Decimal>,      // 12
    pub aliq_cofins: Option<Decimal>,       // 13
    pub quant_bc_cofins: Option<Arc<str>>,  // 14
    pub aliq_cofins_quant: Option<Decimal>, // 15
    pub vl_cofins: Option<Decimal>,         // 16
    pub cod_cta: Option<Arc<str>>,          // 17
    pub info_compl: Option<Arc<str>>,       // 18
}

impl_sped_record_trait!(RegistroC175);

impl SpedParser for RegistroC175 {
    type Output = RegistroC175;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C175 possui 18 campos de dados + 2 delimitadores = 20.
        if len != 20 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 20,
                tamanho_encontrado: len,
            });
        }

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cfop = fields.get(2).parse_opt();
        let vl_opr = get_decimal_field(3, "VL_OPR")?;
        let vl_desc = get_decimal_field(4, "VL_DESC")?;
        let cst_pis = fields.get(5).parse_opt();
        let vl_bc_pis = get_decimal_field(6, "VL_BC_PIS")?;
        let aliq_pis = get_decimal_field(7, "ALIQ_PIS")?;
        let quant_bc_pis = fields.get(8).to_arc();
        let aliq_pis_quant = get_decimal_field(9, "ALIQ_PIS_QUANT")?;
        let vl_pis = get_decimal_field(10, "VL_PIS")?;
        let cst_cofins = fields.get(11).parse_opt();
        let vl_bc_cofins = get_decimal_field(12, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal_field(13, "ALIQ_COFINS")?;
        let quant_bc_cofins = fields.get(14).to_arc();
        let aliq_cofins_quant = get_decimal_field(15, "ALIQ_COFINS_QUANT")?;
        let vl_cofins = get_decimal_field(16, "VL_COFINS")?;
        let cod_cta = fields.get(17).to_arc();
        let info_compl = fields.get(18).to_arc();

        let reg = RegistroC175 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.to_string(),
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
