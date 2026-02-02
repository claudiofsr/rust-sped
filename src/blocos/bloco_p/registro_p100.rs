use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, ToNaiveDate,
    impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "P100";

#[derive(Debug, Clone)]
pub struct RegistroP100 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub dt_ini: Option<NaiveDate>,            // 2
    pub dt_fin: Option<NaiveDate>,            // 3
    pub vl_rec_tot_est: Option<Decimal>,      // 4
    pub cod_ativ_econ: Option<CompactString>, // 5
    pub vl_rec_ativ_estab: Option<Decimal>,   // 6
    pub vl_exc: Option<Decimal>,              // 7
    pub vl_bc_cont: Option<Decimal>,          // 8
    pub aliq_cont: Option<Decimal>,           // 9
    pub vl_cont_apu: Option<Decimal>,         // 10
    pub cod_cta: Option<CompactString>,       // 11
    pub info_compl: Option<CompactString>,    // 12
}

impl_reg_methods!(RegistroP100);

impl SpedParser for RegistroP100 {
    type Output = RegistroP100;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro P100 possui 12 campos de dados + 2 delimitadores = 14.
        if len != 14 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 14,
                tamanho_encontrado: len,
            })
            .loc();
        }

        // --- Closures auxiliares para campos comuns ---

        // Closure para campos de data (Option<NaiveDate>)
        let get_date = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let dt_ini = get_date(2, "DT_INI")?;
        let dt_fin = get_date(3, "DT_FIN")?;
        let vl_rec_tot_est = get_decimal(4, "VL_REC_TOT_EST")?;
        let cod_ativ_econ = fields.get(5).to_compact_string();
        let vl_rec_ativ_estab = get_decimal(6, "VL_REC_ATIV_ESTAB")?;
        let vl_exc = get_decimal(7, "VL_EXC")?;
        let vl_bc_cont = get_decimal(8, "VL_BC_CONT")?;
        let aliq_cont = get_decimal(9, "ALIQ_CONT")?;
        let vl_cont_apu = get_decimal(10, "VL_CONT_APU")?;
        let cod_cta = fields.get(11).to_compact_string();
        let info_compl = fields.get(12).to_compact_string();

        let reg = RegistroP100 {
            nivel: 3,
            bloco: 'P',
            registro: REGISTRO.into(),
            line_number,
            dt_ini,
            dt_fin,
            vl_rec_tot_est,
            cod_ativ_econ,
            vl_rec_ativ_estab,
            vl_exc,
            vl_bc_cont,
            aliq_cont,
            vl_cont_apu,
            cod_cta,
            info_compl,
        };

        Ok(reg)
    }
}
