use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, ToNaiveDate,
    impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C190";

#[derive(Debug, Clone)]
pub struct RegistroC190 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_mod: Option<CompactString>,  // 2
    pub dt_ref_ini: Option<NaiveDate>,   // 3
    pub dt_ref_fin: Option<NaiveDate>,   // 4
    pub cod_item: Option<CompactString>, // 5
    pub cod_ncm: Option<CompactString>,  // 6
    pub ex_ipi: Option<CompactString>,   // 7
    pub vl_tot_item: Option<Decimal>,    // 8
}

impl_reg_methods!(RegistroC190);

impl SpedParser for RegistroC190 {
    type Output = RegistroC190;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C190 possui 8 campos de dados + 2 delimitadores = 10.
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

        let get_date = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path, line_number, field_name)
        };

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cod_mod = fields.get(2).to_compact_string();
        let dt_ref_ini = get_date(3, "DT_REF_INI")?;
        let dt_ref_fin = get_date(4, "DT_REF_FIN")?;
        let cod_item = fields.get(5).to_compact_string();
        let cod_ncm = fields.get(6).to_compact_string();
        let ex_ipi = fields.get(7).to_compact_string();
        let vl_tot_item = get_decimal(8, "VL_TOT_ITEM")?;

        let reg = RegistroC190 {
            nivel: 3,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cod_mod,
            dt_ref_ini,
            dt_ref_fin,
            cod_item,
            cod_ncm,
            ex_ipi,
            vl_tot_item,
        };

        Ok(reg)
    }
}
