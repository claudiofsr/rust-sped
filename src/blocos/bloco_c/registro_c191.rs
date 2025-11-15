use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct RegistroC191 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cnpj_cpf_part: Option<String>,   // 2
    pub cst_pis: Option<String>,         // 3
    pub cfop: Option<String>,            // 4
    pub vl_item: Option<Decimal>,        // 5
    pub vl_desc: Option<Decimal>,        // 6
    pub vl_bc_pis: Option<Decimal>,      // 7
    pub aliq_pis: Option<Decimal>,       // 8
    pub quant_bc_pis: Option<String>,    // 9
    pub aliq_pis_quant: Option<Decimal>, // 10
    pub vl_pis: Option<Decimal>,         // 11
    pub cod_cta: Option<String>,         // 12
}

impl_sped_record_trait!(RegistroC191);

impl SpedParser for RegistroC191 {
    type Output = RegistroC191;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro C191 possui 12 campos de dados + 2 delimitadores = 14.
        if len != 14 {
            return Err(EFDError::InvalidLength {
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
        let cst_pis = fields.get(3).to_optional_string();
        let cfop = fields.get(4).to_optional_string();
        let vl_item = get_decimal_field(5, "VL_ITEM")?;
        let vl_desc = get_decimal_field(6, "VL_DESC")?;
        let vl_bc_pis = get_decimal_field(7, "VL_BC_PIS")?;
        let aliq_pis = get_decimal_field(8, "ALIQ_PIS")?;
        let quant_bc_pis = fields.get(9).to_optional_string();
        let aliq_pis_quant = get_decimal_field(10, "ALIQ_PIS_QUANT")?;
        let vl_pis = get_decimal_field(11, "VL_PIS")?;
        let cod_cta = fields.get(12).to_optional_string();

        let reg = RegistroC191 {
            nivel: 4,
            bloco: 'C',
            registro,
            line_number,
            cnpj_cpf_part,
            cst_pis,
            cfop,
            vl_item,
            vl_desc,
            vl_bc_pis,
            aliq_pis,
            quant_bc_pis,
            aliq_pis_quant,
            vl_pis,
            cod_cta,
        };

        Ok(reg)
    }
}
