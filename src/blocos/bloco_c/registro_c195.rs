use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct RegistroC195 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cnpj_cpf_part: Option<String>,      // 2
    pub cst_cofins: Option<String>,         // 3
    pub cfop: Option<String>,               // 4
    pub vl_item: Option<Decimal>,           // 5
    pub vl_desc: Option<Decimal>,           // 6
    pub vl_bc_cofins: Option<Decimal>,      // 7
    pub aliq_cofins: Option<Decimal>,       // 8
    pub quant_bc_cofins: Option<String>,    // 9
    pub aliq_cofins_quant: Option<Decimal>, // 10
    pub vl_cofins: Option<Decimal>,         // 11
    pub cod_cta: Option<String>,            // 12
}

impl_sped_record_trait!(RegistroC195);

impl SpedParser for RegistroC195 {
    type Output = RegistroC195;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro C195 possui 12 campos de dados + 2 delimitadores = 14.
        if len != 14 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 14,
                tamanho_encontrado: len,
            });
        }

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let cnpj_cpf_part = fields.get(2).to_optional_string();
        let cst_cofins = fields.get(3).to_optional_string();
        let cfop = fields.get(4).to_optional_string();
        let vl_item = get_decimal_field(5, "VL_ITEM")?;
        let vl_desc = get_decimal_field(6, "VL_DESC")?;
        let vl_bc_cofins = get_decimal_field(7, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal_field(8, "ALIQ_COFINS")?;
        let quant_bc_cofins = fields.get(9).to_optional_string();
        let aliq_cofins_quant = get_decimal_field(10, "ALIQ_COFINS_QUANT")?;
        let vl_cofins = get_decimal_field(11, "VL_COFINS")?;
        let cod_cta = fields.get(12).to_optional_string();

        let reg = RegistroC195 {
            nivel: 4,
            bloco: 'C',
            registro,
            line_number,
            cnpj_cpf_part,
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
