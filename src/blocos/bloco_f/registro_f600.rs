use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, ToNaiveDate,
    impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "F600";

#[derive(Debug, Clone)]
pub struct RegistroF600 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_nat_ret: Option<CompactString>, // 2
    pub dt_ret: Option<NaiveDate>,          // 3
    pub vl_bc_ret: Option<Decimal>,         // 4
    pub vl_ret: Option<Decimal>,            // 5
    pub cod_rec: Option<CompactString>,     // 6
    pub ind_nat_rec: Option<CompactString>, // 7
    pub cnpj: Option<CompactString>,        // 8
    pub vl_ret_pis: Option<Decimal>,        // 9
    pub vl_ret_cofins: Option<Decimal>,     // 10
    pub ind_dec: Option<CompactString>,     // 11
}

impl_reg_methods!(RegistroF600);

impl SpedParser for RegistroF600 {
    type Output = RegistroF600;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro F600 possui 11 campos de dados + 2 delimitadores = 13.
        if len != 13 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 13,
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

        let ind_nat_ret = fields.get(2).to_compact_string();
        let dt_ret = get_date(3, "DT_RET")?;
        let vl_bc_ret = get_decimal(4, "VL_BC_RET")?;
        let vl_ret = get_decimal(5, "VL_RET")?;
        let cod_rec = fields.get(6).to_compact_string();
        let ind_nat_rec = fields.get(7).to_compact_string();
        let cnpj = fields.get(8).to_compact_string();
        let vl_ret_pis = get_decimal(9, "VL_RET_PIS")?;
        let vl_ret_cofins = get_decimal(10, "VL_RET_COFINS")?;
        let ind_dec = fields.get(11).to_compact_string();

        let reg = RegistroF600 {
            nivel: 3,
            bloco: 'F',
            registro: REGISTRO.into(),
            line_number,
            ind_nat_ret,
            dt_ret,
            vl_bc_ret,
            vl_ret,
            cod_rec,
            ind_nat_rec,
            cnpj,
            vl_ret_pis,
            vl_ret_cofins,
            ind_dec,
        };

        Ok(reg)
    }
}
