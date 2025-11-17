use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug)]
pub struct Registro1900 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cnpj: Option<String>,        // 2
    pub cod_mod: Option<String>,     // 3
    pub ser: Option<String>,         // 4
    pub sub_ser: Option<String>,     // 5
    pub cod_sit: Option<String>,     // 6
    pub vl_tot_rec: Option<Decimal>, // 7
    pub quant_doc: Option<String>,   // 8 (Assumindo que pode ser string para quantidade ou Decimal)
    pub cst_pis: Option<String>,     // 9
    pub cst_cofins: Option<String>,  // 10
    pub cfop: Option<String>,        // 11
    pub inf_compl: Option<String>,   // 12
    pub cod_cta: Option<String>,     // 13
}

impl_sped_record_trait!(Registro1900);

impl SpedParser for Registro1900 {
    type Output = Registro1900;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro 1900 possui 13 campos de dados + 2 delimitadores = 15.
        if len != 15 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 15,
                tamanho_encontrado: len,
            });
        }

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let cnpj = fields.get(2).to_optional_string();
        let cod_mod = fields.get(3).to_optional_string();
        let ser = fields.get(4).to_optional_string();
        let sub_ser = fields.get(5).to_optional_string();
        let cod_sit = fields.get(6).to_optional_string();
        let vl_tot_rec = get_decimal_field(7, "VL_TOT_REC")?;
        let quant_doc = fields.get(8).to_optional_string(); // Pode ser String se a quantidade for tratada como tal, ou Decimal
        let cst_pis = fields.get(9).to_optional_string();
        let cst_cofins = fields.get(10).to_optional_string();
        let cfop = fields.get(11).to_optional_string();
        let inf_compl = fields.get(12).to_optional_string();
        let cod_cta = fields.get(13).to_optional_string();

        let reg = Registro1900 {
            nivel: 2,
            bloco: '1',
            registro,
            line_number,
            cnpj,
            cod_mod,
            ser,
            sub_ser,
            cod_sit,
            vl_tot_rec,
            quant_doc,
            cst_pis,
            cst_cofins,
            cfop,
            inf_compl,
            cod_cta,
        };

        Ok(reg)
    }
}
