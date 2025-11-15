use crate::{
    EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalNaiveDate, ToOptionalString,
    impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct RegistroC100 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_oper: Option<String>,       // 2
    pub ind_emit: Option<String>,       // 3
    pub cod_part: Option<String>,       // 4
    pub cod_mod: Option<String>,        // 5
    pub cod_sit: Option<String>,        // 6
    pub serie: Option<String>,          // 7
    pub num_doc: Option<String>,        // 8
    pub chv_nfe: Option<String>,        // 9
    pub dt_doc: Option<NaiveDate>,      // 10
    pub dt_e_s: Option<NaiveDate>,      // 11
    pub vl_doc: Option<Decimal>,        // 12
    pub ind_pgto: Option<String>,       // 13
    pub vl_desc: Option<Decimal>,       // 14
    pub vl_abat_nt: Option<Decimal>,    // 15
    pub vl_merc: Option<Decimal>,       // 16
    pub ind_frt: Option<String>,        // 17
    pub vl_frt: Option<Decimal>,        // 18
    pub vl_seg: Option<Decimal>,        // 19
    pub vl_out_da: Option<Decimal>,     // 20
    pub vl_bc_icms: Option<Decimal>,    // 21
    pub vl_icms: Option<Decimal>,       // 22
    pub vl_bc_icms_st: Option<Decimal>, // 23
    pub vl_icms_st: Option<Decimal>,    // 24
    pub vl_ipi: Option<Decimal>,        // 25
    pub vl_pis: Option<Decimal>,        // 26
    pub vl_cofins: Option<Decimal>,     // 27
    pub vl_pis_st: Option<Decimal>,     // 28
    pub vl_cofins_st: Option<Decimal>,  // 29
}

impl_sped_record_trait!(RegistroC100);

impl SpedParser for RegistroC100 {
    type Output = RegistroC100;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        if len != 31 {
            return Err(EFDError::InvalidLength {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro,
                tamanho_esperado: 31,
                tamanho_encontrado: len,
            });
        }

        // --- Closures auxiliares para campos comuns ---

        // Closure para campos de data (Option<NaiveDate>)
        let get_date_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path.to_path_buf(), line_number, field_name)
        };

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let ind_oper = fields.get(2).to_optional_string();
        let ind_emit = fields.get(3).to_optional_string();
        let cod_part = fields.get(4).to_optional_string();
        let cod_mod = fields.get(5).to_optional_string();
        let cod_sit = fields.get(6).to_optional_string();
        let serie = fields.get(7).to_optional_string();
        let num_doc = fields.get(8).to_optional_string();
        let chv_nfe = fields.get(9).to_optional_string();

        // Usando ToNaiveDate para campos de data
        let dt_doc = get_date_field(10, "DT_DOC")?;
        let dt_e_s = get_date_field(11, "DT_E_S")?;

        // Usando ToDecimal para campos monetários (retornando Option<Decimal>)
        let vl_doc = get_decimal_field(12, "VL_DOC")?;
        let ind_pgto = fields.get(13).to_optional_string();
        let vl_desc = get_decimal_field(14, "VL_DESC")?;
        let vl_abat_nt = get_decimal_field(15, "VL_ABAT_NT")?;
        let vl_merc = get_decimal_field(16, "VL_MERC")?;
        let ind_frt = fields.get(17).to_optional_string();
        let vl_frt = get_decimal_field(18, "VL_FRT")?;
        let vl_seg = get_decimal_field(19, "VL_SEG")?;
        let vl_out_da = get_decimal_field(20, "VL_OUT_DA")?;
        let vl_bc_icms = get_decimal_field(21, "VL_BC_ICMS")?;
        let vl_icms = get_decimal_field(22, "VL_ICMS")?;
        let vl_bc_icms_st = get_decimal_field(23, "VL_BC_ICMS_ST")?;
        let vl_icms_st = get_decimal_field(24, "VL_ICMS_ST")?;
        let vl_ipi = get_decimal_field(25, "VL_IPI")?;
        let vl_pis = get_decimal_field(26, "VL_PIS")?;
        let vl_cofins = get_decimal_field(27, "VL_COFINS")?;
        let vl_pis_st = get_decimal_field(28, "VL_PIS_ST")?;
        let vl_cofins_st = get_decimal_field(29, "VL_COFINS_ST")?;

        let reg = RegistroC100 {
            nivel: 3,
            bloco: 'C',
            line_number,
            registro,
            ind_oper,
            ind_emit,
            cod_part,
            cod_mod,
            cod_sit,
            serie,
            num_doc,
            chv_nfe,
            dt_doc,
            dt_e_s,
            vl_doc,
            ind_pgto,
            vl_desc,
            vl_abat_nt,
            vl_merc,
            ind_frt,
            vl_frt,
            vl_seg,
            vl_out_da,
            vl_bc_icms,
            vl_icms,
            vl_bc_icms_st,
            vl_icms_st,
            vl_ipi,
            vl_pis,
            vl_cofins,
            vl_pis_st,
            vl_cofins_st,
        };

        Ok(reg)
    }
}
