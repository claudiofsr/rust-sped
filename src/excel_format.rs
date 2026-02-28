use crate::{EFDResult, FORMAT_REGEX_SET};
use rust_xlsxwriter::{Color, DocProperties, ExcelDateTime, Format, FormatAlign};
use serde::{Deserialize, Serialize};
use serde_aux::prelude::serde_introspect;
use std::collections::HashMap;

/// Constantes estéticas para o Excel.
pub const FONT_SIZE: f64 = 12.0;
pub const HEADER_FONT_SIZE: f64 = 11.0;
pub const MAX_NUMBER_OF_ROWS: usize = 1_000_000;
pub const WIDTH_MIN: usize = 10;
pub const WIDTH_MAX: usize = 100;
pub const ADJUSTMENT: f64 = 1.2;

/// Cores de fundo para identificação visual de tipos de linha.
pub const COLOR_SOMA: Color = Color::RGB(0xBFBFBF);
pub const COLOR_DESCONTO: Color = Color::RGB(0xCCC0DA);
pub const COLOR_SALDO: Color = Color::RGB(0xE6B8B7);

// --- Traits & Enums ---

/// Extensão para obter metadados de headers via Serde Introspection.
pub trait InfoExtension {
    fn get_headers<'de>() -> &'static [&'static str]
    where
        Self: Serialize + Deserialize<'de>,
    {
        serde_introspect::<Self>()
    }
}

/// Trait para registros que decidem seu estilo visual (Cores de linha).
pub trait ExcelCustomFormatter {
    fn row_style(&self) -> RowStyle {
        RowStyle::Default
    }
}

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
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ItensDocsFiscais => "Itens de Docs Fiscais",
            Self::ConsolidacaoCST => "Consolidação CST",
            Self::AnaliseCreditos => "Análise dos Créditos",
        }
    }
    pub fn is_itens(&self) -> bool {
        matches!(self, Self::ItensDocsFiscais)
    }
}

impl std::fmt::Display for SheetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Identificadores para tipos de formatação de coluna.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormatKey {
    Default,
    Center,
    Value,
    Aliquota,
    Date,
}

impl FormatKey {
    pub fn new() -> [(FormatKey, FormatAlign, Option<&'static str>); 5] {
        [
            (FormatKey::Default, FormatAlign::Left, None),
            (FormatKey::Center, FormatAlign::Center, None),
            (FormatKey::Value, FormatAlign::Right, Some("#,##0.00")),
            (FormatKey::Aliquota, FormatAlign::Center, Some("0.0000")),
            (FormatKey::Date, FormatAlign::Center, Some("dd/mm/yyyy")),
        ]
    }
}

/// Estados de estilo para uma linha inteira.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RowStyle {
    Default,
    Soma,
    Desconto,
    Saldo,
}

impl RowStyle {
    pub fn new() -> [(RowStyle, Option<Color>); 4] {
        [
            (RowStyle::Default, None),
            (RowStyle::Soma, Some(COLOR_SOMA)),
            (RowStyle::Desconto, Some(COLOR_DESCONTO)),
            (RowStyle::Saldo, Some(COLOR_SALDO)),
        ]
    }
}

/// Gerenciador central de formatos que mapeia (Tipo de Coluna x Estilo de Linha).
#[derive(Debug, Default)]
pub struct FormatRegistry {
    matrix: HashMap<(FormatKey, RowStyle), Format>,
}

impl FormatRegistry {
    /// Cria um novo registro com todos os formatos pré-calculados.
    pub fn new() -> Self {
        let mut matrix = HashMap::new();
        let keys = FormatKey::new();
        let styles = RowStyle::new();

        for (f_key, align, num_fmt) in keys {
            for (r_style, color) in styles {
                let mut f = Format::new()
                    .set_align(align)
                    .set_align(FormatAlign::VerticalCenter)
                    .set_font_size(FONT_SIZE);

                if let Some(fmt) = num_fmt {
                    f = f.set_num_format(fmt);
                }
                if let Some(c) = color {
                    f = f.set_background_color(c);
                }

                matrix.insert((f_key, r_style), f);
            }
        }
        Self { matrix }
    }

    /// Obtém um formato específico da matriz.
    #[inline]
    pub fn get_format(&self, f_key: FormatKey, r_style: RowStyle) -> Option<&Format> {
        self.matrix.get(&(f_key, r_style))
    }

    /// Atalho para formato de cabeçalho.
    pub fn header() -> Format {
        Format::new()
            .set_text_wrap()
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_font_size(HEADER_FONT_SIZE)
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

// --- Funções Auxiliares de Formatação ---

/// Identifica a chave de formatação baseada no nome da coluna (Regex).
pub fn get_format_key(col_name: &str, sheet_type: SheetType) -> FormatKey {
    // 1. Casos específicos com short-circuit (mais rápidos que Regex)
    if col_name.starts_with("Código de Situação Tributária")
        || col_name.starts_with("Tipo de Crédito")
    {
        return if sheet_type.is_itens() {
            FormatKey::Default
        } else {
            FormatKey::Center
        };
    }

    // 2. Uso do RegexSet para categorias gerais
    // matches() retorna um iterador com os índices que deram match.
    // Usamos .into_iter().next() para pegar o primeiro match por prioridade.
    FORMAT_REGEX_SET
        .matches(col_name)
        .into_iter()
        .next()
        .map(|index| match index {
            0 => FormatKey::Value,
            1 => FormatKey::Aliquota,
            2 => FormatKey::Date,
            3 => FormatKey::Center,
            _ => FormatKey::Default,
        })
        .unwrap_or(FormatKey::Default)
}
