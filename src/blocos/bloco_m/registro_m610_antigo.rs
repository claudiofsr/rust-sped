use crate::{EFDError, EFDResult, SpedParser, ToDecimal, ToOptionalString, impl_sped_record_trait};
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "M610";

#[derive(Debug, Clone)]
pub struct RegistroM610Antigo {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: String,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_cont: Option<String>,           // 2
    pub vl_rec_brt: Option<Decimal>,        // 3
    pub vl_bc_cont: Option<Decimal>,        // 4
    pub aliq_cofins: Option<Decimal>,       // 5
    pub quant_bc_cofins: Option<String>,    // 6 (Pode ser String ou Decimal)
    pub aliq_cofins_quant: Option<Decimal>, // 7
    pub vl_cont_apur: Option<Decimal>,      // 8
    pub vl_ajus_acres: Option<Decimal>,     // 9
    pub vl_ajus_reduc: Option<Decimal>,     // 10
    pub vl_cont_difer: Option<Decimal>,     // 11
    pub vl_cont_difer_ant: Option<Decimal>, // 12
    pub vl_cont_per: Option<Decimal>,       // 13
}

impl_sped_record_trait!(RegistroM610Antigo);

impl SpedParser for RegistroM610Antigo {
    type Output = RegistroM610Antigo;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro M610_Antigo possui 13 campos de dados + 2 delimitadores = 15.
        if len != 15 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.to_string(),
                tamanho_esperado: 15,
                tamanho_encontrado: len,
            });
        }

        // Closure para campos decimais (Option<Decimal>)
        let get_decimal_field = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cod_cont = fields.get(2).to_optional_string();
        let vl_rec_brt = get_decimal_field(3, "VL_REC_BRT")?;
        let vl_bc_cont = get_decimal_field(4, "VL_BC_CONT")?;
        let aliq_cofins = get_decimal_field(5, "ALIQ_COFINS")?;
        let quant_bc_cofins = fields.get(6).to_optional_string();
        let aliq_cofins_quant = get_decimal_field(7, "ALIQ_COFINS_QUANT")?;
        let vl_cont_apur = get_decimal_field(8, "VL_CONT_APUR")?;
        let vl_ajus_acres = get_decimal_field(9, "VL_AJUS_ACRES")?;
        let vl_ajus_reduc = get_decimal_field(10, "VL_AJUS_REDUC")?;
        let vl_cont_difer = get_decimal_field(11, "VL_CONT_DIFER")?;
        let vl_cont_difer_ant = get_decimal_field(12, "VL_CONT_DIFER_ANT")?;
        let vl_cont_per = get_decimal_field(13, "VL_CONT_PER")?;

        let reg = RegistroM610Antigo {
            nivel: 3,
            bloco: 'M',
            registro: REGISTRO.to_string(),
            line_number,
            cod_cont,
            vl_rec_brt,
            vl_bc_cont,
            aliq_cofins,
            quant_bc_cofins,
            aliq_cofins_quant,
            vl_cont_apur,
            vl_ajus_acres,
            vl_ajus_reduc,
            vl_cont_difer,
            vl_cont_difer_ant,
            vl_cont_per,
        };

        Ok(reg)
    }
}
