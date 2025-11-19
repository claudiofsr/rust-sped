use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct RegistroM611 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_tip_coop: Option<String>,             // 2
    pub vl_bc_cont_ant_exc_coop: Option<Decimal>, // 3
    pub vl_exc_coop_ger: Option<Decimal>,         // 4
    pub vl_exc_esp_coop: Option<Decimal>,         // 5
    pub vl_bc_cont: Option<Decimal>,              // 6
}

impl_sped_record_trait!(RegistroM611);

impl SpedParser for RegistroM611 {
    type Output = RegistroM611;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let registro = fields[1].to_uppercase();
        let len: usize = fields.len();

        // O registro M611 possui 6 campos de dados + 2 delimitadores = 8.
        if len != 8 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: registro.clone(),
                tamanho_esperado: 8,
                tamanho_encontrado: len,
            });
        }

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path.to_path_buf(), line_number, field_name)
        };

        let ind_tip_coop = fields.get(2).to_optional_string();
        let vl_bc_cont_ant_exc_coop = get_decimal_field(3, "VL_BC_CONT_ANT_EXC_COOP")?;
        let vl_exc_coop_ger = get_decimal_field(4, "VL_EXC_COOP_GER")?;
        let vl_exc_esp_coop = get_decimal_field(5, "VL_EXC_ESP_COOP")?;
        let vl_bc_cont = get_decimal_field(6, "VL_BC_CONT")?;

        let reg = RegistroM611 {
            nivel: 4,
            bloco: 'M',
            registro,
            line_number,
            ind_tip_coop,
            vl_bc_cont_ant_exc_coop,
            vl_exc_coop_ger,
            vl_exc_esp_coop,
            vl_bc_cont,
        };

        Ok(reg)
    }
}
