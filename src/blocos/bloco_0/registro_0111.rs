use crate::{
    DECIMAL_VALOR, DecimalExt, EFDError, EFDResult, ResultExt, SpedParser, ToDecimal,
    impl_reg_methods,
};
use rust_decimal::Decimal;
use std::{fmt::Write, path::Path, sync::Arc};

const REGISTRO: &str = "0111";

#[derive(Debug, Clone)]
pub struct Registro0111 {
    /// Nível hierárquico
    pub nivel: u16,

    /// Organização do Arquivo da EFD Contribuições - Blocos e Registros
    pub bloco: char,

    /// Código de 4 caracteres do Registro
    pub registro: Arc<str>,

    /// Número da linha do arquivo Sped EFD Contribuições
    pub line_number: usize,

    pub rec_bru_ncum_trib_mi: Option<Decimal>, // 2
    pub rec_bru_ncum_nt_mi: Option<Decimal>,   // 3
    pub rec_bru_ncum_exp: Option<Decimal>,     // 4
    pub rec_bru_cum: Option<Decimal>,          // 5
    pub rec_bru_total: Option<Decimal>,        // 6
}

impl Registro0111 {
    /// Gera o relatório formatado como uma String.
    pub fn generate_report(&self) -> String {
        // 1. Mapeamos os campos e suas descrições em um array
        let campos = [
            (
                self.rec_bru_ncum_trib_mi,
                "Receita Bruta Não-Cumulativa - Tributada no Mercado Interno",
            ),
            (
                self.rec_bru_ncum_nt_mi,
                "Receita Bruta Não-Cumulativa - Não Tributada no Mercado Interno",
            ),
            (
                self.rec_bru_ncum_exp,
                "Receita Bruta Não-Cumulativa - Exportação",
            ),
            (self.rec_bru_cum, "Receita Bruta Cumulativa"),
            (self.rec_bru_total, "Receita Bruta Total"),
        ];

        // Mapeia para strings formatadas primeiro
        let formatados: Vec<(String, &str)> = campos
            .into_iter()
            .map(|(v, desc)| {
                let s = v
                    .map(|v| v.to_formatted_string(DECIMAL_VALOR))
                    .unwrap_or_else(|| "?".to_string());
                (s, desc)
            })
            .collect();

        // Pega o comprimento real das strings (incluindo pontos e vírgulas)
        let num_char = formatados.iter().map(|(s, _)| s.len()).max().unwrap_or(0);

        formatados
            .iter()
            .fold(String::with_capacity(1024), |mut acc, (s, desc)| {
                writeln!(acc, "Registo {}: {:>num_char$} ({})", REGISTRO, s, desc).ok();
                acc
            })
    }
}

impl_reg_methods!(Registro0111);

impl SpedParser for Registro0111 {
    type Output = Registro0111;

    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output> {
        let len: usize = fields.len();

        if len != 8 {
            return Err(EFDError::InvalidFieldCount {
                arquivo: file_path.to_path_buf(),
                linha_num: line_number,
                registro: REGISTRO.into(),
                tamanho_esperado: 8,
                tamanho_encontrado: len,
            })
            .loc();
        }

        let get_decimal = |idx: usize, field_name: &str| {
            fields
                .get(idx)
                .to_decimal(file_path, line_number, field_name)
        };

        let rec_bru_ncum_trib_mi = get_decimal(2, "REC_BRU_NCUM_TRIB_MI")?;
        let rec_bru_ncum_nt_mi = get_decimal(3, "REC_BRU_NCUM_NT_MI")?;
        let rec_bru_ncum_exp = get_decimal(4, "REC_BRU_NCUM_EXP")?;
        let rec_bru_cum = get_decimal(5, "REC_BRU_CUM")?;
        let rec_bru_total = get_decimal(6, "REC_BRU_TOTAL")?;

        let reg = Registro0111 {
            nivel: 3,
            bloco: '0',
            registro: REGISTRO.into(),
            line_number,
            rec_bru_ncum_trib_mi,
            rec_bru_ncum_nt_mi,
            rec_bru_ncum_exp,
            rec_bru_cum,
            rec_bru_total,
        };

        Ok(reg)
    }
}
