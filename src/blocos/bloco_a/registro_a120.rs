use crate::{
    EFDError, EFDResult, SpedParser, StringParser, ToDecimal, ToNaiveDate, impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::{path::Path, sync::Arc};

const REGISTRO: &str = "A120";

#[derive(Debug, Clone)]
pub struct RegistroA120 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub vl_tot_serv: Option<Decimal>,     // 2
    pub vl_bc_pis: Option<Decimal>,       // 3
    pub vl_pis_imp: Option<Decimal>,      // 4
    pub dt_pag_pis: Option<NaiveDate>,    // 5 (Assumindo que é data)
    pub vl_bc_cofins: Option<Decimal>,    // 6
    pub vl_cofins_imp: Option<Decimal>,   // 7
    pub dt_pag_cofins: Option<NaiveDate>, // 8 (Assumindo que é data)
    pub loc_exe_serv: Option<Arc<str>>,   // 9
}

impl_sped_record_trait!(RegistroA120);

impl SpedParser for RegistroA120 {
    type Output = RegistroA120;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro A120 possui 9 campos de dados + 2 delimitadores = 11.
        if len != 11 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 11,
                tamanho_encontrado: len,
            });
        }

        // --- Closures auxiliares para campos comuns ---

        // Closure para campos de data (Option<NaiveDate>)
        let get_date_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let vl_tot_serv = get_decimal_field(2, "VL_TOT_SERV")?;
        let vl_bc_pis = get_decimal_field(3, "VL_BC_PIS")?;
        let vl_pis_imp = get_decimal_field(4, "VL_PIS_IMP")?;
        let dt_pag_pis = get_date_field(5, "DT_PAG_PIS")?;
        let vl_bc_cofins = get_decimal_field(6, "VL_BC_COFINS")?;
        let vl_cofins_imp = get_decimal_field(7, "VL_COFINS_IMP")?;
        let dt_pag_cofins = get_date_field(8, "DT_PAG_COFINS")?;
        let loc_exe_serv = fields.get(9).to_arc();

        let reg = RegistroA120 {
            nivel: 4,
            bloco: 'A',
            registro: REGISTRO.to_string(),
            line_number,
            vl_tot_serv,
            vl_bc_pis,
            vl_pis_imp,
            dt_pag_pis,
            vl_bc_cofins,
            vl_cofins_imp,
            dt_pag_cofins,
            loc_exe_serv,
        };

        Ok(reg)
    }
}
