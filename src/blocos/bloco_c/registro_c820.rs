use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct RegistroC820 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cfop: Option<String>,               // 2
    pub vl_item: Option<Decimal>,           // 3
    pub cod_item: Option<String>,           // 4
    pub cst_pis: Option<String>,            // 5
    pub quant_bc_pis: Option<String>,       // 6 (Pode ser String ou Decimal)
    pub aliq_pis_quant: Option<Decimal>,    // 7
    pub vl_pis: Option<Decimal>,            // 8
    pub cst_cofins: Option<String>,         // 9
    pub quant_bc_cofins: Option<String>,    // 10 (Pode ser String ou Decimal)
    pub aliq_cofins_quant: Option<Decimal>, // 11
    pub vl_cofins: Option<Decimal>,         // 12
    pub cod_cta: Option<String>,            // 13
}

impl_sped_record_trait!(RegistroC820);

impl SpedParser for RegistroC820 {
    type Output = RegistroC820;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro C820 possui 13 campos de dados + 2 delimitadores = 15.
        if len != 15 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 15,
                tamanho_encontrado: len,
            });
        }

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let cfop = fields.get(2).to_optional_string();
        let vl_item = get_decimal_field(3, "VL_ITEM")?;
        let cod_item = fields.get(4).to_optional_string();
        let cst_pis = fields.get(5).to_optional_string();
        let quant_bc_pis = fields.get(6).to_optional_string();
        let aliq_pis_quant = get_decimal_field(7, "ALIQ_PIS_QUANT")?;
        let vl_pis = get_decimal_field(8, "VL_PIS")?;
        let cst_cofins = fields.get(9).to_optional_string();
        let quant_bc_cofins = fields.get(10).to_optional_string();
        let aliq_cofins_quant = get_decimal_field(11, "ALIQ_COFINS_QUANT")?;
        let vl_cofins = get_decimal_field(12, "VL_COFINS")?;
        let cod_cta = fields.get(13).to_optional_string();

        let reg = RegistroC820 {
            nivel: 4,
            bloco: 'C',
            registro,
            line_number,
            cfop,
            vl_item,
            cod_item,
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
