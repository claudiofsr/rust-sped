use crate::{
    EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalNaiveDate, ToOptionalString,
    impl_sped_record_trait,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct RegistroA100 {
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
    pub cod_sit: Option<String>,        // 5
    pub ser: Option<String>,            // 6
    pub sub: Option<String>,            // 7
    pub num_doc: Option<String>,        // 8
    pub chv_nfse: Option<String>,       // 9
    pub dt_doc: Option<NaiveDate>,      // 10
    pub dt_exe_serv: Option<NaiveDate>, // 11
    pub vl_doc: Option<Decimal>,        // 12
    pub ind_pgto: Option<String>,       // 13
    pub vl_desc: Option<Decimal>,       // 14
    pub vl_bc_pis: Option<Decimal>,     // 15
    pub vl_pis: Option<Decimal>,        // 16
    pub vl_bc_cofins: Option<Decimal>,  // 17
    pub vl_cofins: Option<Decimal>,     // 18
    pub vl_pis_ret: Option<Decimal>,    // 19
    pub vl_cofins_ret: Option<Decimal>, // 20
    pub vl_iss: Option<Decimal>,        // 21
}

impl_sped_record_trait!(RegistroA100);

impl SpedParser for RegistroA100 {
    type Output = RegistroA100;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro A100 possui 21 campos de dados + 2 delimitadores = 23.
        if len != 23 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 23,
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
        let cod_sit = fields.get(5).to_optional_string();
        let ser = fields.get(6).to_optional_string();
        let sub = fields.get(7).to_optional_string();
        let num_doc = fields.get(8).to_optional_string();
        let chv_nfse = fields.get(9).to_optional_string();

        let dt_doc = get_date_field(10, "DT_DOC")?;
        let dt_exe_serv = get_date_field(11, "DT_EXE_SERV")?;

        let vl_doc = get_decimal_field(12, "VL_DOC")?;
        let ind_pgto = fields.get(13).to_optional_string();
        let vl_desc = get_decimal_field(14, "VL_DESC")?;
        let vl_bc_pis = get_decimal_field(15, "VL_BC_PIS")?;
        let vl_pis = get_decimal_field(16, "VL_PIS")?;
        let vl_bc_cofins = get_decimal_field(17, "VL_BC_COFINS")?;
        let vl_cofins = get_decimal_field(18, "VL_COFINS")?;
        let vl_pis_ret = get_decimal_field(19, "VL_PIS_RET")?;
        let vl_cofins_ret = get_decimal_field(20, "VL_COFINS_RET")?;
        let vl_iss = get_decimal_field(21, "VL_ISS")?;

        let reg = RegistroA100 {
            nivel: 3,
            bloco: 'A',
            registro,
            line_number,
            ind_oper,
            ind_emit,
            cod_part,
            cod_sit,
            ser,
            sub,
            num_doc,
            chv_nfse,
            dt_doc,
            dt_exe_serv,
            vl_doc,
            ind_pgto,
            vl_desc,
            vl_bc_pis,
            vl_pis,
            vl_bc_cofins,
            vl_cofins,
            vl_pis_ret,
            vl_cofins_ret,
            vl_iss,
        };

        Ok(reg)
    }
}
