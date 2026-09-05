use crate::{
    CodigoSituacaoTributaria, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal,
    impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C191";

#[derive(Debug, Clone)]
pub struct RegistroC191 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cnpj_cpf_part: Option<CompactString>,      // 2
    pub cst_pis: Option<CodigoSituacaoTributaria>, // 3
    pub cfop: Option<u16>,                         // 4
    pub vl_item: Option<Decimal>,                  // 5
    pub vl_desc: Option<Decimal>,                  // 6
    pub vl_bc_pis: Option<Decimal>,                // 7
    pub aliq_pis: Option<Decimal>,                 // 8
    pub quant_bc_pis: Option<CompactString>,       // 9
    pub aliq_pis_quant: Option<Decimal>,           // 10
    pub vl_pis: Option<Decimal>,                   // 11
    pub cod_cta: Option<CompactString>,            // 12
}

impl_reg_methods!(RegistroC191);

impl SpedParser for RegistroC191 {
    type Output = RegistroC191;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C191 possui 12 campos de dados + 2 delimitadores = 14.
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
        let cst_pis = fields
            .get(3)
            .to_efd_field(file_path, line_number, "CST_PIS")?;
        let cfop = fields.get(4).parse_opt();
        let vl_item = get_decimal(5, "VL_ITEM")?;
        let vl_desc = get_decimal(6, "VL_DESC")?;
        let vl_bc_pis = get_decimal(7, "VL_BC_PIS")?;
        let aliq_pis = get_decimal(8, "ALIQ_PIS")?;
        let quant_bc_pis = fields.get(9).to_compact_string();
        let aliq_pis_quant = get_decimal(10, "ALIQ_PIS_QUANT")?;
        let vl_pis = get_decimal(11, "VL_PIS")?;
        let cod_cta = fields.get(12).to_compact_string();

        let reg = RegistroC191 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cnpj_cpf_part,
            cst_pis,
            cfop,
            vl_item,
            vl_desc,
            vl_bc_pis,
            aliq_pis,
            quant_bc_pis,
            aliq_pis_quant,
            vl_pis,
            cod_cta,
        };

        Ok(reg)
    }
}
