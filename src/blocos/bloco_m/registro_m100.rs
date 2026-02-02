use crate::{
    CodigoDoCredito, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToCodigoDoCredito,
    ToDecimal, impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "M100";

/// Registro M100: Crédito de PIS/Pasep Relativo ao Período
#[derive(Debug, Clone)]
pub struct RegistroM100 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_cred: Option<CodigoDoCredito>,    // 2
    pub ind_cred_ori: Option<CompactString>,  // 3
    pub vl_bc_pis: Option<Decimal>,           // 4
    pub aliq_pis: Option<Decimal>,            // 5
    pub quant_bc_pis: Option<CompactString>,  // 6 (Pode ser String ou Decimal)
    pub aliq_pis_quant: Option<Decimal>,      // 7
    pub vl_cred: Option<Decimal>,             // 8
    pub vl_ajus_acres: Option<Decimal>,       // 9
    pub vl_ajus_reduc: Option<Decimal>,       // 10
    pub vl_cred_dif: Option<Decimal>,         // 11
    pub vl_cred_disp: Option<Decimal>,        // 12
    pub ind_desc_cred: Option<CompactString>, // 13
    pub vl_cred_desc: Option<Decimal>,        // 14
    pub sld_cred: Option<Decimal>,            // 15
}

impl_reg_methods!(RegistroM100);

impl SpedParser for RegistroM100 {
    type Output = RegistroM100;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro M100 possui 15 campos de dados + 2 delimitadores = 17.
        if len != 17 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 17,
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

        let cod_cred = fields
            .get(2)
            .to_codigo_do_credito(file_path, line_number, "COD_CRED")?;
        let ind_cred_ori = fields.get(3).to_compact_string();
        let vl_bc_pis = get_decimal(4, "VL_BC_PIS")?;
        let aliq_pis = get_decimal(5, "ALIQ_PIS")?;
        let quant_bc_pis = fields.get(6).to_compact_string();
        let aliq_pis_quant = get_decimal(7, "ALIQ_PIS_QUANT")?;
        let vl_cred = get_decimal(8, "VL_CRED")?;
        let vl_ajus_acres = get_decimal(9, "VL_AJUS_ACRES")?;
        let vl_ajus_reduc = get_decimal(10, "VL_AJUS_REDUC")?;
        let vl_cred_dif = get_decimal(11, "VL_CRED_DIF")?;
        let vl_cred_disp = get_decimal(12, "VL_CRED_DISP")?;
        let ind_desc_cred = fields.get(13).to_compact_string();
        let vl_cred_desc = get_decimal(14, "VL_CRED_DESC")?;
        let sld_cred = get_decimal(15, "SLD_CRED")?;

        let reg = RegistroM100 {
            nivel: 2,
            bloco: 'M',
            registro: REGISTRO.into(),
            line_number,
            cod_cred,
            ind_cred_ori,
            vl_bc_pis,
            aliq_pis,
            quant_bc_pis,
            aliq_pis_quant,
            vl_cred,
            vl_ajus_acres,
            vl_ajus_reduc,
            vl_cred_dif,
            vl_cred_disp,
            ind_desc_cred,
            vl_cred_desc,
            sld_cred,
        };

        Ok(reg)
    }
}
