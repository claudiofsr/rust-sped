use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "F525";

#[derive(Debug, Clone)]
pub struct RegistroF525 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub vl_rec: Option<Decimal>,     // 2
    pub ind_rec: Option<String>,     // 3
    pub cnpj_cpf: Option<String>,    // 4
    pub num_doc: Option<String>,     // 5
    pub cod_item: Option<String>,    // 6
    pub vl_rec_det: Option<Decimal>, // 7
    pub cst_pis: Option<String>,     // 8
    pub cst_cofins: Option<String>,  // 9
    pub info_compl: Option<String>,  // 10
    pub cod_cta: Option<String>,     // 11
}

impl_sped_record_trait!(RegistroF525);

impl SpedParser for RegistroF525 {
    type Output = RegistroF525;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro F525 possui 11 campos de dados + 2 delimitadores = 13.
        if len != 13 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 13,
                tamanho_encontrado: len,
            });
        }

        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let vl_rec = get_decimal_field(2, "VL_REC")?;
        let ind_rec = fields.get(3).to_optional_string();
        let cnpj_cpf = fields.get(4).to_optional_string();
        let num_doc = fields.get(5).to_optional_string();
        let cod_item = fields.get(6).to_optional_string();
        let vl_rec_det = get_decimal_field(7, "VL_REC_DET")?;
        let cst_pis = fields.get(8).to_optional_string();
        let cst_cofins = fields.get(9).to_optional_string();
        let info_compl = fields.get(10).to_optional_string();
        let cod_cta = fields.get(11).to_optional_string();

        let reg = RegistroF525 {
            nivel: 3,
            bloco: 'F',
            registro: REGISTRO.to_string(),
            line_number,
            vl_rec,
            ind_rec,
            cnpj_cpf,
            num_doc,
            cod_item,
            vl_rec_det,
            cst_pis,
            cst_cofins,
            info_compl,
            cod_cta,
        };

        Ok(reg)
    }
}
