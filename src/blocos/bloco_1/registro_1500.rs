use crate::{EFDError, EFDResult, SpedParser, StringParser, ToDecimal, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "1500";

#[derive(Debug, Clone)]
pub struct Registro1500 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub per_apu_cred: Option<Arc<str>>,        // 2
    pub orig_cred: Option<Arc<str>>,           // 3
    pub cnpj_suc: Option<Arc<str>>,            // 4
    pub cod_cred: Option<u16>,                 // 5
    pub vl_cred_apu: Option<Decimal>,          // 6
    pub vl_cred_ext_apu: Option<Decimal>,      // 7
    pub vl_tot_cred_apu: Option<Decimal>,      // 8
    pub vl_cred_desc_pa_ant: Option<Decimal>,  // 9
    pub vl_cred_per_pa_ant: Option<Decimal>,   // 10
    pub vl_cred_dcomp_pa_ant: Option<Decimal>, // 11
    pub sd_cred_disp_efd: Option<Arc<str>>,    // 12
    pub vl_cred_desc_efd: Option<Decimal>,     // 13
    pub vl_cred_per_efd: Option<Decimal>,      // 14
    pub vl_cred_dcomp_efd: Option<Decimal>,    // 15
    pub vl_cred_trans: Option<Decimal>,        // 16
    pub vl_cred_out: Option<Decimal>,          // 17
    pub sld_cred_fim: Option<Decimal>,         // 18
}

impl_sped_record_trait!(Registro1500);

impl SpedParser for Registro1500 {
    type Output = Registro1500;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro 1500 possui 18 campos de dados + 2 delimitadores = 20.
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

        let per_apu_cred = fields.get(2).to_arc();
        let orig_cred = fields.get(3).to_arc();
        let cnpj_suc = fields.get(4).to_arc();
        let cod_cred = fields.get(5).parse_opt();
        let vl_cred_apu = get_decimal_field(6, "VL_CRED_APU")?;
        let vl_cred_ext_apu = get_decimal_field(7, "VL_CRED_EXT_APU")?;
        let vl_tot_cred_apu = get_decimal_field(8, "VL_TOT_CRED_APU")?;
        let vl_cred_desc_pa_ant = get_decimal_field(9, "VL_CRED_DESC_PA_ANT")?;
        let vl_cred_per_pa_ant = get_decimal_field(10, "VL_CRED_PER_PA_ANT")?;
        let vl_cred_dcomp_pa_ant = get_decimal_field(11, "VL_CRED_DCOMP_PA_ANT")?;
        let sd_cred_disp_efd = fields.get(12).to_arc();
        let vl_cred_desc_efd = get_decimal_field(13, "VL_CRED_DESC_EFD")?;
        let vl_cred_per_efd = get_decimal_field(14, "VL_CRED_PER_EFD")?;
        let vl_cred_dcomp_efd = get_decimal_field(15, "VL_CRED_DCOMP_EFD")?;
        let vl_cred_trans = get_decimal_field(16, "VL_CRED_TRANS")?;
        let vl_cred_out = get_decimal_field(17, "VL_CRED_OUT")?;
        let sld_cred_fim = get_decimal_field(18, "SLD_CRED_FIM")?;

        let reg = Registro1500 {
            nivel: 2,
            bloco: '1',
            registro: REGISTRO.to_string(),
            line_number,
            per_apu_cred,
            orig_cred,
            cnpj_suc,
            cod_cred,
            vl_cred_apu,
            vl_cred_ext_apu,
            vl_tot_cred_apu,
            vl_cred_desc_pa_ant,
            vl_cred_per_pa_ant,
            vl_cred_dcomp_pa_ant,
            sd_cred_disp_efd,
            vl_cred_desc_efd,
            vl_cred_per_efd,
            vl_cred_dcomp_efd,
            vl_cred_trans,
            vl_cred_out,
            sld_cred_fim,
        };

        Ok(reg)
    }
}
