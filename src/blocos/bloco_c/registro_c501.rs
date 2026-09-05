use crate::{
    CodigoSituacaoTributaria, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal,
    impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "C501";

#[derive(Debug, Clone)]
pub struct RegistroC501 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub cst_pis: Option<CodigoSituacaoTributaria>, // 2
    pub vl_item: Option<Decimal>,                  // 3
    pub nat_bc_cred: Option<u16>,                  // 4
    pub vl_bc_pis: Option<Decimal>,                // 5
    pub aliq_pis: Option<Decimal>,                 // 6
    pub vl_pis: Option<Decimal>,                   // 7
    pub cod_cta: Option<CompactString>,            // 8
}

impl_reg_methods!(RegistroC501);

impl SpedParser for RegistroC501 {
    type Output = RegistroC501;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro C501 possui 8 campos de dados + 2 delimitadores = 10.
        if len != 10 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 10,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let cst_pis = fields
            .get(2)
            .to_efd_field(file_path, line_number, "CST_PIS")?;
        let vl_item = get_decimal(3, "VL_ITEM")?;
        let nat_bc_cred = fields.get(4).parse_opt();
        let vl_bc_pis = get_decimal(5, "VL_BC_PIS")?;
        let aliq_pis = get_decimal(6, "ALIQ_PIS")?;
        let vl_pis = get_decimal(7, "VL_PIS")?;
        let cod_cta = fields.get(8).to_compact_string();

        let reg = RegistroC501 {
            nivel: 4,
            bloco: 'C',
            registro: REGISTRO.into(),
            line_number,
            cst_pis,
            vl_item,
            nat_bc_cred,
            vl_bc_pis,
            aliq_pis,
            vl_pis,
            cod_cta,
        };

        Ok(reg)
    }
}
