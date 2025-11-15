use crate::{EFDError, EFDResult, SpedParser, ToDecimal, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct RegistroM600 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub vl_tot_cont_nc_per: Option<Decimal>,   // 2
    pub vl_tot_cred_desc: Option<Decimal>,     // 3
    pub vl_tot_cred_desc_ant: Option<Decimal>, // 4
    pub vl_tot_cont_nc_dev: Option<Decimal>,   // 5
    pub vl_ret_nc: Option<Decimal>,            // 6
    pub vl_out_ded_nc: Option<Decimal>,        // 7
    pub vl_cont_nc_rec: Option<Decimal>,       // 8
    pub vl_tot_cont_cum_per: Option<Decimal>,  // 9
    pub vl_ret_cum: Option<Decimal>,           // 10
    pub vl_out_ded_cum: Option<Decimal>,       // 11
    pub vl_cont_cum_rec: Option<Decimal>,      // 12
    pub vl_tot_cont_rec: Option<Decimal>,      // 13
}

impl_sped_record_trait!(RegistroM600);

impl SpedParser for RegistroM600 {
    type Output = RegistroM600;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro M600 possui 13 campos de dados + 2 delimitadores = 15.
        if len != 15 {
            return Err(EFDError::InvalidLength {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 15,
                tamanho_encontrado: len,
            });
        }

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let vl_tot_cont_nc_per = get_decimal_field(2, "VL_TOT_CONT_NC_PER")?;
        let vl_tot_cred_desc = get_decimal_field(3, "VL_TOT_CRED_DESC")?;
        let vl_tot_cred_desc_ant = get_decimal_field(4, "VL_TOT_CRED_DESC_ANT")?;
        let vl_tot_cont_nc_dev = get_decimal_field(5, "VL_TOT_CONT_NC_DEV")?;
        let vl_ret_nc = get_decimal_field(6, "VL_RET_NC")?;
        let vl_out_ded_nc = get_decimal_field(7, "VL_OUT_DED_NC")?;
        let vl_cont_nc_rec = get_decimal_field(8, "VL_CONT_NC_REC")?;
        let vl_tot_cont_cum_per = get_decimal_field(9, "VL_TOT_CONT_CUM_PER")?;
        let vl_ret_cum = get_decimal_field(10, "VL_RET_CUM")?;
        let vl_out_ded_cum = get_decimal_field(11, "VL_OUT_DED_CUM")?;
        let vl_cont_cum_rec = get_decimal_field(12, "VL_CONT_CUM_REC")?;
        let vl_tot_cont_rec = get_decimal_field(13, "VL_TOT_CONT_REC")?;

        let reg = RegistroM600 {
            nivel: 2,
            bloco: 'M',
            registro,
            line_number,
            vl_tot_cont_nc_per,
            vl_tot_cred_desc,
            vl_tot_cred_desc_ant,
            vl_tot_cont_nc_dev,
            vl_ret_nc,
            vl_out_ded_nc,
            vl_cont_nc_rec,
            vl_tot_cont_cum_per,
            vl_ret_cum,
            vl_out_ded_cum,
            vl_cont_cum_rec,
            vl_tot_cont_rec,
        };

        Ok(reg)
    }
}
