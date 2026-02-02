use crate::{EFDError, EFDResult, ResultExt, SpedParser, ToDecimal, impl_reg_methods};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "M200";

#[derive(Debug, Clone)]
pub struct RegistroM200 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

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

impl_reg_methods!(RegistroM200);

impl SpedParser for RegistroM200 {
    type Output = RegistroM200;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro M200 possui 13 campos de dados + 2 delimitadores = 15.
        if len != 15 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 15,
                tamanho_encontrado: len,
            })
            .loc();
        }

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let vl_tot_cont_nc_per = get_decimal(2, "VL_TOT_CONT_NC_PER")?;
        let vl_tot_cred_desc = get_decimal(3, "VL_TOT_CRED_DESC")?;
        let vl_tot_cred_desc_ant = get_decimal(4, "VL_TOT_CRED_DESC_ANT")?;
        let vl_tot_cont_nc_dev = get_decimal(5, "VL_TOT_CONT_NC_DEV")?;
        let vl_ret_nc = get_decimal(6, "VL_RET_NC")?;
        let vl_out_ded_nc = get_decimal(7, "VL_OUT_DED_NC")?;
        let vl_cont_nc_rec = get_decimal(8, "VL_CONT_NC_REC")?;
        let vl_tot_cont_cum_per = get_decimal(9, "VL_TOT_CONT_CUM_PER")?;
        let vl_ret_cum = get_decimal(10, "VL_RET_CUM")?;
        let vl_out_ded_cum = get_decimal(11, "VL_OUT_DED_CUM")?;
        let vl_cont_cum_rec = get_decimal(12, "VL_CONT_CUM_REC")?;
        let vl_tot_cont_rec = get_decimal(13, "VL_TOT_CONT_REC")?;

        let reg = RegistroM200 {
            nivel: 2,
            bloco: 'M',
            registro: REGISTRO.into(),
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
