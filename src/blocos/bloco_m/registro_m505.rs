use crate::{
    CodigoSituacaoTributaria, EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal,
    impl_reg_methods,
};
use compact_str::CompactString;
use rust_decimal::Decimal;
use std::path::Path;

const REGISTRO: &str = "M505";

#[derive(Debug, Clone)]
pub struct RegistroM505 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub nat_bc_cred: Option<u16>,                     // 2
    pub cst_cofins: Option<CodigoSituacaoTributaria>, // 3
    pub vl_bc_cofins_tot: Option<Decimal>,            // 4
    pub vl_bc_cofins_cum: Option<Decimal>,            // 5
    pub vl_bc_cofins_nc: Option<Decimal>,             // 6
    pub vl_bc_cofins: Option<Decimal>,                // 7
    pub quant_bc_cofins_tot: Option<CompactString>,   // 8 (Pode ser String ou Decimal)
    pub quant_bc_cofins: Option<CompactString>,       // 9 (Pode ser String ou Decimal)
    pub desc_cred: Option<CompactString>,             // 10
}

impl_reg_methods!(RegistroM505);

impl SpedParser for RegistroM505 {
    type Output = RegistroM505;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro M505 possui 10 campos de dados + 2 delimitadores = 12.
        if len != 12 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 12,
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

        let nat_bc_cred = fields.get(2).parse_opt();
        let cst_cofins = fields
            .get(3)
            .to_efd_field(file_path, line_number, "CST_COFINS")?;
        let vl_bc_cofins_tot = get_decimal(4, "VL_BC_COFINS_TOT")?;
        let vl_bc_cofins_cum = get_decimal(5, "VL_BC_COFINS_CUM")?;
        let vl_bc_cofins_nc = get_decimal(6, "VL_BC_COFINS_NC")?;
        let vl_bc_cofins = get_decimal(7, "VL_BC_COFINS")?;
        let quant_bc_cofins_tot = fields.get(8).to_compact_string();
        let quant_bc_cofins = fields.get(9).to_compact_string();
        let desc_cred = fields.get(10).to_compact_string();

        let reg = RegistroM505 {
            nivel: 3,
            bloco: 'M',
            registro: REGISTRO.into(),
            line_number,
            nat_bc_cred,
            cst_cofins,
            vl_bc_cofins_tot,
            vl_bc_cofins_cum,
            vl_bc_cofins_nc,
            vl_bc_cofins,
            quant_bc_cofins_tot,
            quant_bc_cofins,
            desc_cred,
        };

        Ok(reg)
    }
}
