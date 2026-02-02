use crate::{
    EFDError, EFDResult, ResultExt, SpedParser, StringParser, ToDecimal, impl_reg_methods,
};
use compact_str::CompactString;
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
    pub registro: CompactString,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub vl_rec: Option<Decimal>,           // 2
    pub ind_rec: Option<CompactString>,    // 3
    pub cnpj_cpf: Option<CompactString>,   // 4
    pub num_doc: Option<usize>,            // 5
    pub cod_item: Option<CompactString>,   // 6
    pub vl_rec_det: Option<Decimal>,       // 7
    pub cst_pis: Option<u16>,              // 8
    pub cst_cofins: Option<u16>,           // 9
    pub info_compl: Option<CompactString>, // 10
    pub cod_cta: Option<CompactString>,    // 11
}

impl_reg_methods!(RegistroF525);

impl SpedParser for RegistroF525 {
    type Output = RegistroF525;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        // O registro F525 possui 11 campos de dados + 2 delimitadores = 13.
        if len != 13 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 13,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let vl_rec = get_decimal(2, "VL_REC")?;
        let ind_rec = fields.get(3).to_compact_string();
        let cnpj_cpf = fields.get(4).to_compact_string();
        let num_doc = fields.get(5).parse_opt();
        let cod_item = fields.get(6).to_compact_string();
        let vl_rec_det = get_decimal(7, "VL_REC_DET")?;
        let cst_pis = fields.get(8).parse_opt();
        let cst_cofins = fields.get(9).parse_opt();
        let info_compl = fields.get(10).to_compact_string();
        let cod_cta = fields.get(11).to_compact_string();

        let reg = RegistroF525 {
            nivel: 3,
            bloco: 'F',
            registro: REGISTRO.into(),
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
