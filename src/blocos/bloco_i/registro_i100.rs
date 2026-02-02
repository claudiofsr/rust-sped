use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "I100";

#[derive(Debug, Clone)]
pub struct RegistroI100 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub vl_rec: Option<Decimal>,           // 2
    pub cst_pis_cofins: Option<u16>,       // 3
    pub vl_tot_ded_ger: Option<Decimal>,   // 4
    pub vl_tot_ded_esp: Option<Decimal>,   // 5
    pub vl_bc_pis: Option<Decimal>,        // 6
    pub aliq_pis: Option<Decimal>,         // 7
    pub vl_pis: Option<Decimal>,           // 8
    pub vl_bc_cofins: Option<Decimal>,     // 9
    pub aliq_cofins: Option<Decimal>,      // 10
    pub vl_cofins: Option<Decimal>,        // 11
    pub info_compl: Option<CompactString>, // 12
}

impl_reg_methods!(RegistroI100);

impl SpedParser for RegistroI100 {
    type Output = RegistroI100;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro I100 possui 12 campos de dados + 2 delimitadores = 14.
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

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let vl_rec = get_decimal(2, "VL_REC")?;
        let cst_pis_cofins = fields.get(3).parse_opt();
        let vl_tot_ded_ger = get_decimal(4, "VL_TOT_DED_GER")?;
        let vl_tot_ded_esp = get_decimal(5, "VL_TOT_DED_ESP")?;
        let vl_bc_pis = get_decimal(6, "VL_BC_PIS")?;
        let aliq_pis = get_decimal(7, "ALIQ_PIS")?;
        let vl_pis = get_decimal(8, "VL_PIS")?;
        let vl_bc_cofins = get_decimal(9, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(10, "ALIQ_COFINS")?;
        let vl_cofins = get_decimal(11, "VL_COFINS")?;
        let info_compl = fields.get(12).to_compact_string();

        let reg = RegistroI100 {
            nivel: 3,
            bloco: 'I',
            registro: REGISTRO.into(),
            line_number,
            vl_rec,
            cst_pis_cofins,
            vl_tot_ded_ger,
            vl_tot_ded_esp,
            vl_bc_pis,
            aliq_pis,
            vl_pis,
            vl_bc_cofins,
            aliq_cofins,
            vl_cofins,
            info_compl,
        };

        Ok(reg)
    }
}
