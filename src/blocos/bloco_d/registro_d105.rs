use crate::{
    CodigoSituacaoTributaria, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal,
    impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "D105";

#[derive(Debug, Clone)]
pub struct RegistroD105 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub ind_nat_frt: Option<CompactString>,           // 2
    pub vl_item: Option<Decimal>,                     // 3
    pub cst_cofins: Option<CodigoSituacaoTributaria>, // 4
    pub nat_bc_cred: Option<u16>,                     // 5
    pub vl_bc_cofins: Option<Decimal>,                // 6
    pub aliq_cofins: Option<Decimal>,                 // 7
    pub vl_cofins: Option<Decimal>,                   // 8
    pub cod_cta: Option<CompactString>,               // 9
}

impl_reg_methods!(RegistroD105);

impl SpedParser for RegistroD105 {
    type Output = RegistroD105;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro D105 possui 9 campos de dados + 2 delimitadores = 11.
        if len != 11 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 11,
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

        let ind_nat_frt = fields.get(2).to_compact_string();
        let vl_item = get_decimal(3, "VL_ITEM")?;
        let cst_cofins = fields
            .get(4)
            .to_efd_field(file_path, line_number, "CST_COFINS")?;
        let nat_bc_cred = fields.get(5).parse_opt();
        let vl_bc_cofins = get_decimal(6, "VL_BC_COFINS")?;
        let aliq_cofins = get_decimal(7, "ALIQ_COFINS")?;
        let vl_cofins = get_decimal(8, "VL_COFINS")?;
        let cod_cta = fields.get(9).to_compact_string();

        let reg = RegistroD105 {
            nivel: 4,
            bloco: 'D',
            registro: REGISTRO.into(),
            line_number,
            ind_nat_frt,
            vl_item,
            cst_cofins,
            nat_bc_cred,
            vl_bc_cofins,
            aliq_cofins,
            vl_cofins,
            cod_cta,
        };

        Ok(reg)
    }
}
