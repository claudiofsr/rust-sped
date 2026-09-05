use crate::{EFDError, EFDResult, ResultExt, SpedParser, ToDecimal, impl_reg_methods};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "M350";

#[derive(Debug, Clone)]
pub struct RegistroM350 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub vl_tot_fol: Option<Decimal>,      // 2
    pub vl_exc_bc: Option<Decimal>,       // 3
    pub vl_tot_bc: Option<Decimal>,       // 4
    pub aliq_pis_fol: Option<Decimal>,    // 5
    pub vl_tot_cont_fol: Option<Decimal>, // 6
}

impl_reg_methods!(RegistroM350);

impl SpedParser for RegistroM350 {
    type Output = RegistroM350;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro M350 possui 6 campos de dados + 2 delimitadores = 8.
        if len != 8 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 8,
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

        let vl_tot_fol = get_decimal(2, "VL_TOT_FOL")?;
        let vl_exc_bc = get_decimal(3, "VL_EXC_BC")?;
        let vl_tot_bc = get_decimal(4, "VL_TOT_BC")?;
        let aliq_pis_fol = get_decimal(5, "ALIQ_PIS_FOL")?;
        let vl_tot_cont_fol = get_decimal(6, "VL_TOT_CONT_FOL")?;

        let reg = RegistroM350 {
            nivel: 2,
            bloco: 'M',
            registro: REGISTRO.into(),
            line_number,
            vl_tot_fol,
            vl_exc_bc,
            vl_tot_bc,
            aliq_pis_fol,
            vl_tot_cont_fol,
        };

        Ok(reg)
    }
}
