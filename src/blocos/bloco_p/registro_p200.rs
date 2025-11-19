use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct RegistroP200 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub per_ref: Option<String>,          // 2
    pub vl_tot_cont_apu: Option<Decimal>, // 3
    pub vl_tot_aj_reduc: Option<Decimal>, // 4
    pub vl_tot_aj_acres: Option<Decimal>, // 5
    pub vl_tot_cont_dev: Option<Decimal>, // 6
    pub cod_rec: Option<String>,          // 7
}

impl_sped_record_trait!(RegistroP200);

impl SpedParser for RegistroP200 {
    type Output = RegistroP200;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro P200 possui 7 campos de dados + 2 delimitadores = 9.
        if len != 9 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 9,
                tamanho_encontrado: len,
            });
        }

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let per_ref = fields.get(2).to_optional_string();
        let vl_tot_cont_apu = get_decimal_field(3, "VL_TOT_CONT_APU")?;
        let vl_tot_aj_reduc = get_decimal_field(4, "VL_TOT_AJ_REDUC")?;
        let vl_tot_aj_acres = get_decimal_field(5, "VL_TOT_AJ_ACRES")?;
        let vl_tot_cont_dev = get_decimal_field(6, "VL_TOT_CONT_DEV")?;
        let cod_rec = fields.get(7).to_optional_string();

        let reg = RegistroP200 {
            nivel: 2,
            bloco: 'P',
            registro,
            line_number,
            per_ref,
            vl_tot_cont_apu,
            vl_tot_aj_reduc,
            vl_tot_aj_acres,
            vl_tot_cont_dev,
            cod_rec,
        };

        Ok(reg)
    }
}
