use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "M610";

#[derive(Debug, Clone)]
pub struct RegistroM610 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cod_cont: Option<CompactString>,          // 2
    pub vl_rec_brt: Option<Decimal>,              // 3
    pub vl_bc_cont: Option<Decimal>,              // 4
    pub vl_ajus_acres_bc_cofins: Option<Decimal>, // 5
    pub vl_ajus_reduc_bc_cofins: Option<Decimal>, // 6
    pub vl_bc_cont_ajus: Option<Decimal>,         // 7
    pub aliq_cofins: Option<Decimal>,             // 8
    pub quant_bc_cofins: Option<CompactString>,   // 9 (Pode ser String ou Decimal)
    pub aliq_cofins_quant: Option<Decimal>,       // 10
    pub vl_cont_apur: Option<Decimal>,            // 11
    pub vl_ajus_acres: Option<Decimal>,           // 12
    pub vl_ajus_reduc: Option<Decimal>,           // 13
    pub vl_cont_difer: Option<Decimal>,           // 14
    pub vl_cont_difer_ant: Option<Decimal>,       // 15
    pub vl_cont_per: Option<Decimal>,             // 16
}

impl_reg_methods!(RegistroM610);

impl SpedParser for RegistroM610 {
    type Output = RegistroM610;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro M610 possui 16 campos de dados + 2 delimitadores = 18.
        if len != 18 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 18,
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

        let cod_cont = fields.get(2).to_compact_string();
        let vl_rec_brt = get_decimal(3, "VL_REC_BRT")?;
        let vl_bc_cont = get_decimal(4, "VL_BC_CONT")?;
        let vl_ajus_acres_bc_cofins = get_decimal(5, "VL_AJUS_ACRES_BC_COFINS")?;
        let vl_ajus_reduc_bc_cofins = get_decimal(6, "VL_AJUS_REDUC_BC_COFINS")?;
        let vl_bc_cont_ajus = get_decimal(7, "VL_BC_CONT_AJUS")?;
        let aliq_cofins = get_decimal(8, "ALIQ_COFINS")?;
        let quant_bc_cofins = fields.get(9).to_compact_string();
        let aliq_cofins_quant = get_decimal(10, "ALIQ_COFINS_QUANT")?;
        let vl_cont_apur = get_decimal(11, "VL_CONT_APUR")?;
        let vl_ajus_acres = get_decimal(12, "VL_AJUS_ACRES")?;
        let vl_ajus_reduc = get_decimal(13, "VL_AJUS_REDUC")?;
        let vl_cont_difer = get_decimal(14, "VL_CONT_DIFER")?;
        let vl_cont_difer_ant = get_decimal(15, "VL_CONT_DIFER_ANT")?;
        let vl_cont_per = get_decimal(16, "VL_CONT_PER")?;

        let reg = RegistroM610 {
            nivel: 3,
            bloco: 'M',
            registro: REGISTRO.into(),
            line_number,
            cod_cont,
            vl_rec_brt,
            vl_bc_cont,
            vl_ajus_acres_bc_cofins,
            vl_ajus_reduc_bc_cofins,
            vl_bc_cont_ajus,
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
