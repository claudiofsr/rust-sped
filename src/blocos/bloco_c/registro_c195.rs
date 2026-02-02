use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C195";

#[derive(Debug, Clone)]
pub struct RegistroC195 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cnpj_cpf_part: Option<CompactString>,   // 2
    pub cst_cofins: Option<u16>,                // 3
    pub cfop: Option<u16>,                      // 4
    pub vl_item: Option<Decimal>,               // 5
    pub vl_desc: Option<Decimal>,               // 6
    pub vl_bc_cofins: Option<Decimal>,          // 7
    pub aliq_cofins: Option<Decimal>,           // 8
    pub quant_bc_cofins: Option<CompactString>, // 9
    pub aliq_cofins_quant: Option<Decimal>,     // 10
    pub vl_cofins: Option<Decimal>,             // 11
    pub cod_cta: Option<CompactString>,         // 12
}

impl_reg_methods!(RegistroC195);

impl SpedParser for RegistroC195 {
    type Output = RegistroC195;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C195 possui 12 campos de dados + 2 delimitadores = 14.
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

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cnpj_cpf_part = fields.get(2).to_compact_string();
        let cst_cofins = fields.get(3).parse_opt();
        let cfop = fields.get(4).parse_opt();
        let vl_item = get_decimal(5, "VL_ITEM")?;
        let vl_desc = get_decimal(6, "VL_DESC")?;
        let vl_bc_cofins = get_decimal(7, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(8, "ALIQ_COFINS")?;
        let quant_bc_cofins = fields.get(9).to_compact_string();
        let aliq_cofins_quant = get_decimal(10, "ALIQ_COFINS_QUANT")?;
        let vl_cofins = get_decimal(11, "VL_COFINS")?;
        let cod_cta = fields.get(12).to_compact_string();

        let reg = RegistroC195 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cnpj_cpf_part,
            cst_cofins,
            cfop,
            vl_item,
            vl_desc,
            vl_bc_cofins,
            aliq_cofins,
            quant_bc_cofins,
            aliq_cofins_quant,
            vl_cofins,
            cod_cta,
        };

        Ok(reg)
    }
}
