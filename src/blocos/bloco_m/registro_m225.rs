use crate::{
    EFDError, EFDResult, SpedParser, ToNaiveDate, ToOptionalString, impl_sped_record_trait,
};
use chrono::NaiveDate;
use std::path::Path;

#[derive(Debug)]
pub struct RegistroM225 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub det_valor_aj: Option<String>,  // 2
    pub cst_pis: Option<String>,       // 3
    pub det_bc_cred: Option<String>,   // 4
    pub det_aliq: Option<String>,      // 5 (Pode ser String ou Decimal)
    pub dt_oper_aj: Option<NaiveDate>, // 6
    pub desc_aj: Option<String>,       // 7
    pub cod_cta: Option<String>,       // 8
    pub info_compl: Option<String>,    // 9
}

impl_sped_record_trait!(RegistroM225);

impl SpedParser for RegistroM225 {
    type Output = RegistroM225;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro M225 possui 9 campos de dados + 2 delimitadores = 11.
        if len != 11 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 11,
                tamanho_encontrado: len,
            });
        }

        // Closure para campos de data (Option<NaiveDate>)
        let get_date_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_optional_date(file_path.to_path_buf(), line_number, field_name)
        };

        let det_valor_aj = fields.get(2).to_optional_string();
        let cst_pis = fields.get(3).to_optional_string();
        let det_bc_cred = fields.get(4).to_optional_string();
        let det_aliq = fields.get(5).to_optional_string();
        let dt_oper_aj = get_date_field(6, "DT_OPER_AJ")?;
        let desc_aj = fields.get(7).to_optional_string();
        let cod_cta = fields.get(8).to_optional_string();
        let info_compl = fields.get(9).to_optional_string();

        let reg = RegistroM225 {
            nivel: 5,
            bloco: 'M',
            registro,
            line_number,
            det_valor_aj,
            cst_pis,
            det_bc_cred,
            det_aliq,
            dt_oper_aj,
            desc_aj,
            cod_cta,
            info_compl,
        };

        Ok(reg)
    }
}
