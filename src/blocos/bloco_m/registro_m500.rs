use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct RegistroM500 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_cred: Option<String>,           // 2
    pub ind_cred_ori: Option<String>,       // 3
    pub vl_bc_cofins: Option<Decimal>,      // 4
    pub aliq_cofins: Option<Decimal>,       // 5
    pub quant_bc_cofins: Option<String>,    // 6 (Pode ser String ou Decimal)
    pub aliq_cofins_quant: Option<Decimal>, // 7
    pub vl_cred: Option<Decimal>,           // 8
    pub vl_ajus_acres: Option<Decimal>,     // 9
    pub vl_ajus_reduc: Option<Decimal>,     // 10
    pub vl_cred_difer: Option<Decimal>,     // 11
    pub vl_cred_disp: Option<Decimal>,      // 12
    pub ind_desc_cred: Option<String>,      // 13
    pub vl_cred_desc: Option<Decimal>,      // 14
    pub sld_cred: Option<Decimal>,          // 15
}

impl_sped_record_trait!(RegistroM500);

impl SpedParser for RegistroM500 {
    type Output = RegistroM500;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro M500 possui 15 campos de dados + 2 delimitadores = 17.
        if len != 17 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 17,
                tamanho_encontrado: len,
            });
        }

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let cod_cred = fields.get(2).to_optional_string();
        let ind_cred_ori = fields.get(3).to_optional_string();
        let vl_bc_cofins = get_decimal_field(4, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal_field(5, "ALIQ_COFINS")?;
        let quant_bc_cofins = fields.get(6).to_optional_string();
        let aliq_cofins_quant = get_decimal_field(7, "ALIQ_COFINS_QUANT")?;
        let vl_cred = get_decimal_field(8, "VL_CRED")?;
        let vl_ajus_acres = get_decimal_field(9, "VL_AJUS_ACRES")?;
        let vl_ajus_reduc = get_decimal_field(10, "VL_AJUS_REDUC")?;
        let vl_cred_difer = get_decimal_field(11, "VL_CRED_DIFER")?;
        let vl_cred_disp = get_decimal_field(12, "VL_CRED_DISP")?;
        let ind_desc_cred = fields.get(13).to_optional_string();
        let vl_cred_desc = get_decimal_field(14, "VL_CRED_DESC")?;
        let sld_cred = get_decimal_field(15, "SLD_CRED")?;

        let reg = RegistroM500 {
            nivel: 2,
            bloco: 'M',
            registro,
            line_number,
            cod_cred,
            ind_cred_ori,
            vl_bc_cofins,
            aliq_cofins,
            quant_bc_cofins,
            aliq_cofins_quant,
            vl_cred,
            vl_ajus_acres,
            vl_ajus_reduc,
            vl_cred_difer,
            vl_cred_disp,
            ind_desc_cred,
            vl_cred_desc,
            sld_cred,
        };

        Ok(reg)
    }
}
