use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, ToNaiveDate,
    impl_reg_methods,
};
use chrono::NaiveDate;
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C100";

/// Documento - Nota Fiscal (Código 01), Nota Fiscal Avulsa (Código 1B),
/// Nota Fiscal de Produtor (Código 04), NF-e (Código 55) e NFC-e (Código 65)
#[derive(Debug, Clone)]
pub struct RegistroC100 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_oper: Option<CompactString>, // 2
    pub ind_emit: Option<CompactString>, // 3
    pub cod_part: Option<CompactString>, // 4
    pub cod_mod: Option<CompactString>,  // 5
    pub cod_sit: Option<CompactString>,  // 6
    pub serie: Option<CompactString>,    // 7
    pub num_doc: Option<usize>,          // 8
    pub chv_nfe: Option<CompactString>,  // 9
    pub dt_doc: Option<NaiveDate>,       // 10
    pub dt_e_s: Option<NaiveDate>,       // 11
    pub vl_doc: Option<Decimal>,         // 12
    pub ind_pgto: Option<CompactString>, // 13
    pub vl_desc: Option<Decimal>,        // 14
    pub vl_abat_nt: Option<Decimal>,     // 15
    pub vl_merc: Option<Decimal>,        // 16
    pub ind_frt: Option<CompactString>,  // 17
    pub vl_frt: Option<Decimal>,         // 18
    pub vl_seg: Option<Decimal>,         // 19
    pub vl_out_da: Option<Decimal>,      // 20
    pub vl_bc_icms: Option<Decimal>,     // 21
    pub vl_icms: Option<Decimal>,        // 22
    pub vl_bc_icms_st: Option<Decimal>,  // 23
    pub vl_icms_st: Option<Decimal>,     // 24
    pub vl_ipi: Option<Decimal>,         // 25
    pub vl_pis: Option<Decimal>,         // 26
    pub vl_cofins: Option<Decimal>,      // 27
    pub vl_pis_st: Option<Decimal>,      // 28
    pub vl_cofins_st: Option<Decimal>,   // 29
}

impl_reg_methods!(RegistroC100);

impl SpedParser for RegistroC100 {
    type Output = RegistroC100;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != 31 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 31,
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

        let ind_oper = fields.get(2).to_compact_string();
        let ind_emit = fields.get(3).to_compact_string();
        let cod_part = fields.get(4).to_compact_string();
        let cod_mod = fields.get(5).to_compact_string();
        let cod_sit = fields.get(6).to_compact_string();
        let serie = fields.get(7).to_compact_string();
        let num_doc = fields.get(8).parse_opt();
        let chv_nfe = fields.get(9).to_compact_string();

        // Usando ToNaiveDate para campos de data
        let dt_doc = get_date(10, "DT_DOC")?;
        let dt_e_s = get_date(11, "DT_E_S")?;

        // Usando ToDecimal para campos monetários (retornando Option<Decimal>)
        let vl_doc = get_decimal(12, "VL_DOC")?;
        let ind_pgto = fields.get(13).to_compact_string();
        let vl_desc = get_decimal(14, "VL_DESC")?;
        let vl_abat_nt = get_decimal(15, "VL_ABAT_NT")?;
        let vl_merc = get_decimal(16, "VL_MERC")?;
        let ind_frt = fields.get(17).to_compact_string();
        let vl_frt = get_decimal(18, "VL_FRT")?;
        let vl_seg = get_decimal(19, "VL_SEG")?;
        let vl_out_da = get_decimal(20, "VL_OUT_DA")?;
        let vl_bc_icms = get_decimal(21, "VL_BC_ICMS")?;
        let vl_icms = get_decimal(22, "VL_ICMS")?;
        let vl_bc_icms_st = get_decimal(23, "VL_BC_ICMS_ST")?;
        let vl_icms_st = get_decimal(24, "VL_ICMS_ST")?;
        let vl_ipi = get_decimal(25, "VL_IPI")?;
        let vl_pis = get_decimal(26, "VL_PIS")?;
        let vl_cofins = get_decimal(27, "VL_COFINS")?;
        let vl_pis_st = get_decimal(28, "VL_PIS_ST")?;
        let vl_cofins_st = get_decimal(29, "VL_COFINS_ST")?;

        let reg = RegistroC100 {
            nivel: 3,
            bloco: 'C',
            line_number,
            registro: REGISTRO.into(),
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
