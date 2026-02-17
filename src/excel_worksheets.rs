use chrono::NaiveDate;
use claudiofsr_lib::{digit_count, get_style};
use compact_str::CompactString;
use indicatif::{MultiProgress, ProgressBar};
use itertools::Itertools;
use rayon::prelude::*;
use rust_decimal::Decimal;
use rust_xlsxwriter::{Color, Format, FormatAlign, Table, Worksheet};
use serde::{Deserialize, Serialize};
use serde_aux::prelude::serde_introspect;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use struct_iterable::Iterable;

use crate::{
    CodigoDoCredito, CodigoSituacaoTributaria, EFDResult, FORMAT_REGEX_SET, IndicadorDeOrigem,
    NaturezaBaseCalculo, TipoDeCredito, TipoDeOperacao, TipoDoItem, display_cst,
};

// --- Macros ---

/// Realiza o "downcast" dinâmico de um `dyn Any` para tipos específicos.
///
/// Esta macro percorre os tipos fornecidos e executa o corpo da expressão
/// para o primeiro tipo que coincidir com o valor em tempo de execução.
macro_rules! match_cast {
    ($any:ident { $( $bind:ident as $patt:ty => $body:expr ),+ $(,)? }) => {{
        $(
            if let Some($bind) = $any.downcast_ref::<$patt>() {
                Some($body)
            } else
        )+
        { None }
    }};
}

// --- Constantes Estéticas ---

const FONT_SIZE: f64 = 11.0;
const HEADER_FONT_SIZE: f64 = 10.0;
const MAX_NUMBER_OF_ROWS: usize = 1_000_000;
const WIDTH_MIN: usize = 8;
const WIDTH_MAX: usize = 100;
const ADJUSTMENT: f64 = 1.2;

// cores: 0xBFBFBF, 0xE6B8B7, 0xF8CBAD, 0xCCC0DA
const COLOR_SOMA: Color = Color::RGB(0xBFBFBF);
const COLOR_SALDO: Color = Color::RGB(0xE6B8B7);
const COLOR_DESCONTO: Color = Color::RGB(0xCCC0DA);

// --- Traits e Enums ---

/// Extensão para obter metadados de headers via Serde Introspection.
pub trait InfoExtension {
    fn get_headers<'de>() -> &'static [&'static str]
    where
        Self: Deserialize<'de>,
    {
        serde_introspect::<Self>()
    }
}

/// Define as variantes de estilo visual que uma linha pode assumir.
/// O `repr(usize)` permite conversão direta para índice de array em tempo recorde.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum RowStyle {
    Normal = 0,
    Soma = 1,
    Desconto = 2,
    Saldo = 3,
}

/// Trait para permitir que registros individuais decidam seu próprio estilo visual no Excel.
pub trait ExcelCustomFormatter {
    fn row_style(&self) -> RowStyle {
        RowStyle::Normal
    }
}

/// Identificadores internos para tipos de formatação de coluna.
#[derive(Debug, Clone, Copy)]
#[repr(usize)]
enum FormatKey {
    Default = 0,
    Center = 1,
    Value = 2,
    Aliq = 3,
    Date = 4,
}

/// Armazena as 4 variações de cores para um tipo específico de dado.
struct FormatGroup {
    formats: [Format; 4],
}

impl FormatGroup {
    fn new(base: Format) -> Self {
        Self {
            formats: [
                base.clone(),                                      // Normal
                base.clone().set_background_color(COLOR_SOMA),     // Soma
                base.clone().set_background_color(COLOR_DESCONTO), // Desconto
                base.clone().set_background_color(COLOR_SALDO),    // Saldo
            ],
        }
    }

    #[inline]
    fn get_format(&self, style: RowStyle) -> &Format {
        &self.formats[style as usize]
    }
}

/// Gerenciador central de formatos. Reduz drasticamente a alocação de memória
/// ao pré-calcular a matriz de (Estilo de Coluna x Estilo de Linha).
struct FormatRegistry {
    groups: [FormatGroup; 5],
}

impl FormatRegistry {
    /// Inicializa a matriz de formatos (Tipos de Coluna x Estilos de Linha).
    fn new() -> Self {
        let base_c = Format::new()
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_font_size(FONT_SIZE);
        let base_l = Format::new()
            .set_align(FormatAlign::Left)
            .set_align(FormatAlign::VerticalCenter)
            .set_font_size(FONT_SIZE);

        let keys = [
            base_l.clone(),                              // Default
            base_c.clone(),                              // Center
            base_l.clone().set_num_format("#,##0.00"),   // Value
            base_c.clone().set_num_format("#,##0.0000"), // Aliq
            base_c.clone().set_num_format("dd/mm/yyyy"), // Date
        ];

        Self {
            groups: keys.map(FormatGroup::new),
        }
    }

    /// Obtém o grupo de formatação correspondente à chave da coluna em O(1).
    #[inline]
    fn get_group(&self, key: FormatKey) -> &FormatGroup {
        &self.groups[key as usize]
    }
}

// --- Funções Auxiliares de Formatação ---

/// Identifica a chave de formatação baseada no nome da coluna (Regex).
fn get_format_key(col_name: &str, sheet_name: &str) -> FormatKey {
    // 1. Casos específicos com short-circuit (mais rápidos que Regex)
    if col_name.starts_with("Código de Situação Tributária")
        || col_name.starts_with("Tipo de Crédito")
    {
        return if sheet_name.contains("Itens") {
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
            1 => FormatKey::Aliq,
            2 => FormatKey::Date,
            3 => FormatKey::Center,
            _ => FormatKey::Default,
        })
        .unwrap_or(FormatKey::Default)
}

// --- Lógica Principal ---

/// Gera uma ou mais Worksheets a partir de uma coleção de dados,
/// respeitando o limite de linhas do Excel.
pub fn get_worksheets<'de, T>(
    lines: &[T],
    sheet_name: &str,
    multiprogressbar: &MultiProgress,
    index: usize,
) -> EFDResult<Vec<Worksheet>>
where
    T: Serialize + Deserialize<'de> + InfoExtension + Iterable + ExcelCustomFormatter + Sync,
{
    if lines.is_empty() {
        return Ok(Vec::new());
    }

    let pb = multiprogressbar.insert(index, ProgressBar::new(lines.len() as u64));
    pb.set_style(get_style(0, 0, 35)?);
    pb.set_message(format!("Excel: {sheet_name}"));

    let worksheets = lines
        .chunks(MAX_NUMBER_OF_ROWS)
        .enumerate()
        .par_bridge() // Rayon: Processa os chunks em paralelo
        .map(|(k, data)| {
            let name = if k == 0 {
                sheet_name.to_owned()
            } else {
                format!("{sheet_name} {}", k + 1)
            };
            get_worksheet(data, &name, &pb)
        })
        .collect::<EFDResult<Vec<_>>>()?;

    pb.finish();
    Ok(worksheets)
}

/// Constrói uma Worksheet individual, aplicando serialização automática e estilos customizados.
fn get_worksheet<'de, T>(
    lines: &[T],
    sheet_name: &str,
    progressbar: &ProgressBar,
) -> EFDResult<Worksheet>
where
    T: Serialize + Deserialize<'de> + InfoExtension + Iterable + ExcelCustomFormatter + Sync,
{
    let headers = T::get_headers();
    let registry = FormatRegistry::new();

    let num_cols = headers.len();
    let num_lines = lines.len();

    // Otimização: Pre-cache dos grupos de formatação para evitar lookup no loop
    let col_configs: Vec<(u16, &FormatGroup)> = headers
        .par_iter()
        .enumerate()
        .map(|(i, &col_name)| {
            (
                i as u16,
                registry.get_group(get_format_key(col_name, sheet_name)),
            )
        })
        .collect();

    let mut worksheet = Worksheet::new();

    setup_worksheet_styles(
        &mut worksheet,
        sheet_name,
        num_cols as u16,
        num_lines as u32,
        &col_configs,
    )?;

    // Aplica cabeçalhos e formatos de coluna padrão
    worksheet.deserialize_headers::<T>(0, 0)?;

    // Processamento funcional das linhas
    lines
        .iter()
        .enumerate()
        .try_for_each(|(i, line)| -> EFDResult<()> {
            let row_idx = (i + 1) as u32;
            let style = line.row_style();

            // 1. Serialização base (escreve os dados conforme o #[serde])
            // Escreve os dados (Serialização é a parte mais pesada)
            worksheet.serialize(line)?;

            // 2. Aplicação de Estilo de Linha (Cores de Fundo: Soma, Saldo, etc.)
            if style != RowStyle::Normal {
                for (col_idx, group) in &col_configs {
                    worksheet.set_cell_format(row_idx, *col_idx, group.get_format(style))?;
                }
            }

            // Incrementar Progresso
            if i.is_multiple_of(100) {
                progressbar.inc(100);
            }

            Ok(())
        })?;

    auto_fit(&mut worksheet, lines, headers)?;
    Ok(worksheet)
}

/// Helper para configurar o esqueleto da worksheet
fn setup_worksheet_styles(
    ws: &mut Worksheet,
    name: &str,
    num_cols: u16,
    num_lines: u32,
    configs: &[(u16, &FormatGroup)],
) -> EFDResult<()> {
    let header_fmt = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_text_wrap()
        .set_font_size(HEADER_FONT_SIZE);

    ws.set_name(name)?
        .set_row_height(0, 64)?
        .set_row_format(0, &header_fmt)?
        .set_freeze_panes(1, 0)?;

    // Aplica formatos base às colunas
    for (i, group) in configs {
        ws.set_column_format(*i, group.get_format(RowStyle::Normal))?;
    }

    let table = Table::new().set_autofilter(true);
    ws.add_table(0, 0, num_lines, num_cols - 1, &table)?;

    Ok(())
}

/// Ajusta a largura das colunas dinamicamente em paralelo usando Rayon.
fn auto_fit<'de, T>(worksheet: &mut Worksheet, lines: &[T], headers: &[&str]) -> EFDResult<()>
where
    T: Serialize + Deserialize<'de> + InfoExtension + Iterable + Sync,
{
    let widths: Vec<_> = headers
        .iter()
        .map(|h| AtomicUsize::new(WIDTH_MIN.max(h.len().div_ceil(5))))
        .collect();

    let sheet_name = worksheet.name();

    lines.par_iter().for_each(|line| {
        line.iter().enumerate().for_each(|(i, (_name, val))| {
            if let Some(atomic) = widths.get(i) {
                atomic.fetch_max(calculate_value_len(&sheet_name, val), Ordering::Relaxed);
            }
        });
    });

    for (i, atomic) in widths.into_iter().enumerate() {
        let width = (atomic.load(Ordering::Relaxed).min(WIDTH_MAX) as f64) * ADJUSTMENT;
        worksheet.set_column_width(i as u16, width)?;
    }
    Ok(())
}

/// Calcula o comprimento visual de um Decimal.
#[inline]
fn decimal_len(d: &Decimal) -> usize {
    d.to_string().len()
}

/// Calcula o comprimento visual de qualquer valor suportado para ajuste automático de coluna.
fn calculate_value_len(sheet_name: &str, field_value: &dyn std::any::Any) -> usize {
    let len = match_cast!(field_value {
        // Tratamento do CST
        val as Option<CodigoSituacaoTributaria> => {
            val.as_ref().map_or(2, |cst| {
                if sheet_name.contains("Itens") {
                    // Planilha Detalhada: "01 - Operação Tributável..."
                    // Multiplicamos por 110% porque fontes proporcionais (Calibri)
                    // são levemente mais largas que a contagem de caracteres monoespaçados.
                    cst.descricao_com_codigo().len() * 72 / 100
                } else {
                    // Planilha Compacta: Apenas os dígitos "01"
                    display_cst(&Some(*cst)).len() * 82 / 100
                }
            })
        },

        val as Option<NaturezaBaseCalculo> => {
            val.as_ref().map_or(0, |n| {
                n.descricao_com_codigo().len() * 74 / 100
            })
        },

        // Inteiros e Decimais
        val as Option<u16> => val.map_or(0, |v| digit_count(v as usize)),
        val as Option<u32> => val.map_or(0, |v| digit_count(v as usize)),
        val as Option<usize> => val.map_or(0, digit_count),
        val as Decimal => decimal_len(val),
        val as Option<Decimal> => val.as_ref().map_or(0, decimal_len),

        // Datas (Fixo: YYYY-MM-DD ou DD/MM/YYYY)
        val as Option<NaiveDate> => val.map_or(0, |_| 10),

        // Enums formatados
        //val as Option<CodigoSituacaoTributaria> => display_cst(val).len() * 82 / 100,
        val as Option<CodigoDoCredito> => val.as_ref().map_or(0, |_| 5),
        val as Option<IndicadorDeOrigem> => val.as_ref().map_or(0, |s| s.to_string().len() * 88 / 100),
        val as Option<TipoDeCredito> => val.as_ref().map_or(0, |s| s.to_string().len() * 86 / 100),
        val as Option<TipoDeOperacao> => val.as_ref().map_or(0, |s| s.to_string().len()),
        val as Option<TipoDoItem> => val.as_ref().map_or(0, |s| s.to_string().len() * 88 / 100),

        // Strings e Wrappers
        val as String => val.len(),
        val as Arc<str> => val.len(),
        val as CompactString => val.len(),
        val as Option<String> => val.as_ref().map_or(0, |s| s.len()),
        val as Option<Arc<str>> => val.as_deref().map_or(0, |s| s.len()),
        val as Option<CompactString> => val.as_deref().map_or(0, |s| s.len()),

        // Coleções
        val as Vec<String> => val.iter().join(", ").len(),
    });

    len.unwrap_or(WIDTH_MIN)
}
