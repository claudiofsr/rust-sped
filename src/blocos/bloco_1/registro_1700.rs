use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "1700";

#[derive(Debug, Clone)]
pub struct Registro1700 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_nat_ret: Option<CompactString>, // 2
    pub pr_rec_ret: Option<CompactString>,  // 3
    pub vl_ret_apu: Option<Decimal>,        // 4
    pub vl_ret_ded: Option<Decimal>,        // 5
    pub vl_ret_per: Option<Decimal>,        // 6
    pub vl_ret_dcomp: Option<Decimal>,      // 7
    pub sld_ret: Option<Decimal>,           // 8
}

impl_reg_methods!(Registro1700);

impl SpedParser for Registro1700 {
    type Output = Registro1700;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro 1700 possui 8 campos de dados + 2 delimitadores = 10.
        if len != 10 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 10,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let ind_nat_ret = fields.get(2).to_compact_string();
        let pr_rec_ret = fields.get(3).to_compact_string();
        let vl_ret_apu = get_decimal(4, "VL_RET_APU")?;
        let vl_ret_ded = get_decimal(5, "VL_RET_DED")?;
        let vl_ret_per = get_decimal(6, "VL_RET_PER")?;
        let vl_ret_dcomp = get_decimal(7, "VL_RET_DCOMP")?;
        let sld_ret = get_decimal(8, "SLD_RET")?;

        let reg = Registro1700 {
            nivel: 2,
            bloco: '1',
            registro: REGISTRO.into(),
            line_number,
            ind_nat_ret,
            pr_rec_ret,
            vl_ret_apu,
            vl_ret_ded,
            vl_ret_per,
            vl_ret_dcomp,
            sld_ret,
        };

        Ok(reg)
    }
}
