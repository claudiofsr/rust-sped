use crate::EFDResult;
use rust_xlsxwriter::{Color, DocProperties, ExcelDateTime, Format, FormatAlign};
use serde::Deserialize;
use serde_aux::prelude::serde_introspect;

/*
pub const COLOR_HEADER: Color = Color::RGB(0xC5D9F1);
pub const COLOR_SOMA: Color = Color::RGB(0xBFBFBF);
pub const COLOR_SALDO: Color = Color::RGB(0xF8CBAD);
pub const COLOR_DESCONTO: Color = Color::RGB(0xCCC0DA);
pub const COLOR_AVISO: Color = Color::RGB(0xFFF2CC);
*/

// --- Constantes Estéticas Compartilhadas ---
pub const FONT_SIZE: f64 = 12.0;
pub const HEADER_FONT_SIZE: f64 = 11.0;
pub const MAX_NUMBER_OF_ROWS: usize = 1_000_000;

// cores: 0xBFBFBF, 0xE6B8B7, 0xF8CBAD, 0xCCC0DA
pub const COLOR_SOMA: Color = Color::RGB(0xBFBFBF);
pub const COLOR_SALDO: Color = Color::RGB(0xE6B8B7);
pub const COLOR_DESCONTO: Color = Color::RGB(0xCCC0DA);

/// Representa as diferentes abas (worksheets) geradas no arquivo Excel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetType {
    /// Detalhamento dos itens constantes nos documentos fiscais.
    ItensDocsFiscais,
    /// Resumo consolidado por Código de Situação Tributária.
    ConsolidacaoCST,
    /// Detalhamento da análise de naturezas de crédito.
    AnaliseCreditos,
}

impl SheetType {
    /// Retorna o nome amigável da aba que aparecerá no Excel.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ItensDocsFiscais => "Itens de Docs Fiscais",
            Self::ConsolidacaoCST => "Consolidação CST",
            Self::AnaliseCreditos => "Análise dos Créditos",
        }
    }

    /// Verifica se o tipo atual é o de Itens, usado para lógica de formatação específica.
    pub fn is_itens(&self) -> bool {
        matches!(self, Self::ItensDocsFiscais)
    }
}

impl std::fmt::Display for SheetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Extensão para obter metadados de headers via Serde Introspection.
pub trait InfoExtension {
    fn get_headers<'de>() -> &'static [&'static str]
    where
        Self: Deserialize<'de>,
    {
        serde_introspect::<Self>()
    }
}

/// Estrutura para gerenciar cores por estado da linha.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum RowStyle {
    Normal = 0,
    Soma = 1,
    Desconto = 2,
    Saldo = 3,
}

pub fn apply_style_to_format(format: Format, state: RowStyle) -> Format {
    match state {
        RowStyle::Normal => format,
        RowStyle::Soma => format.set_background_color(COLOR_SOMA),
        RowStyle::Desconto => format.set_background_color(COLOR_DESCONTO),
        RowStyle::Saldo => format.set_background_color(COLOR_SALDO),
    }
}

/// Trait para permitir que registros individuais decidam seu próprio estilo visual no Excel.
pub trait ExcelCustomFormatter {
    fn row_style(&self) -> RowStyle {
        RowStyle::Normal
    }
}

/// Gera as propriedades padrão do documento Excel para ambos os módulos.
pub fn get_properties() -> EFDResult<DocProperties> {
    // Create a datetime object.
    let date = ExcelDateTime::from_ymd(2026, 1, 1)?.and_hms(0, 0, 0)?;

    let properties = DocProperties::new()
        .set_title("SPED EFD Contribuições em Excel")
        .set_subject("Informações extraídas de arquivos de SPED EFD Contribuições")
        .set_author("Claudio FSR (https://github.com/claudiofsr/rust-sped)")
        .set_keywords("SPED EFD Contribuições, Rust, Excel")
        .set_comment("Created with Rust and rust_xlsxwriter")
        .set_hyperlink_base("https://github.com/claudiofsr/rust-sped")
        .set_creation_datetime(&date);

    Ok(properties)
}

/// Helper para criar formatos base comuns.
pub fn base_format() -> Format {
    Format::new()
        .set_align(FormatAlign::VerticalCenter)
        .set_font_size(FONT_SIZE)
}
