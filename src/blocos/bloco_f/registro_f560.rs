use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct RegistroF560 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub vl_rec_comp: Option<Decimal>,       // 2
    pub cst_pis: Option<String>,            // 3
    pub vl_desc_pis: Option<Decimal>,       // 4
    pub quant_bc_pis: Option<String>,       // 5 (Pode ser String ou Decimal)
    pub aliq_pis_quant: Option<Decimal>,    // 6
    pub vl_pis: Option<Decimal>,            // 7
    pub cst_cofins: Option<String>,         // 8
    pub vl_desc_cofins: Option<Decimal>,    // 9
    pub quant_bc_cofins: Option<String>,    // 10 (Pode ser String ou Decimal)
    pub aliq_cofins_quant: Option<Decimal>, // 11
    pub vl_cofins: Option<Decimal>,         // 12
    pub cod_mod: Option<String>,            // 13
    pub cfop: Option<String>,               // 14
    pub cod_cta: Option<String>,            // 15
    pub info_compl: Option<String>,         // 16
}

impl_sped_record_trait!(RegistroF560);

impl SpedParser for RegistroF560 {
    type Output = RegistroF560;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro F560 possui 16 campos de dados + 2 delimitadores = 18.
        if len != 18 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 18,
                tamanho_encontrado: len,
            });
        }

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let vl_rec_comp = get_decimal_field(2, "VL_REC_COMP")?;
        let cst_pis = fields.get(3).to_optional_string();
        let vl_desc_pis = get_decimal_field(4, "VL_DESC_PIS")?;
        let quant_bc_pis = fields.get(5).to_optional_string();
        let aliq_pis_quant = get_decimal_field(6, "ALIQ_PIS_QUANT")?;
        let vl_pis = get_decimal_field(7, "VL_PIS")?;
        let cst_cofins = fields.get(8).to_optional_string();
        let vl_desc_cofins = get_decimal_field(9, "VL_DESC_COFINS")?;
        let quant_bc_cofins = fields.get(10).to_optional_string();
        let aliq_cofins_quant = get_decimal_field(11, "ALIQ_COFINS_QUANT")?;
        let vl_cofins = get_decimal_field(12, "VL_COFINS")?;
        let cod_mod = fields.get(13).to_optional_string();
        let cfop = fields.get(14).to_optional_string();
        let cod_cta = fields.get(15).to_optional_string();
        let info_compl = fields.get(16).to_optional_string();

        let reg = RegistroF560 {
            nivel: 3,
            bloco: 'F',
            registro,
            line_number,
            vl_rec_comp,
            cst_pis,
            vl_desc_pis,
            quant_bc_pis,
            aliq_pis_quant,
            vl_pis,
            cst_cofins,
            vl_desc_cofins,
            quant_bc_cofins,
            aliq_cofins_quant,
            vl_cofins,
            cod_mod,
            cfop,
            cod_cta,
            info_compl,
        };

        Ok(reg)
    }
}
