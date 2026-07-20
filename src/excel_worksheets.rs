//! # Excel Worksheet Generation Module
//!
//! This module provides high-performance Excel generation utilities for fiscal documents (EFD).
//! It supports multiple memory allocation strategies (In-Memory parallel rendering, Low-Memory sharing,
//! and Constant-Memory streaming) to balance processing throughput and system resource constraints.
//!
//! Key performance features include:
//! - Parallel worksheet generation using Rayon scopes.
//! - Parallel column-width auto-fitting using lock-free atomic maximum operations (`AtomicUsize`).
//! - Structured serialization driving automated table formatting.

use chrono::NaiveDate;
use clap::ValueEnum;
use claudiofsr_lib::{digit_count, get_style};
use compact_str::CompactString;
use indicatif::{MultiProgress, ProgressBar};
use itertools::Itertools;
use rayon::prelude::*;
use rust_decimal::Decimal;
use rust_xlsxwriter::{Table, Worksheet, workbook::Workbook};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::BufWriter,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use struct_iterable::Iterable;

use crate::{
    AnaliseDosCreditos, BUFFER_CAPACITY, CodigoDoCredito, CodigoSituacaoTributaria,
    ConsolidacaoCST, DocsFiscais, EFDError, EFDResult, IndicadorDeOrigem, NaturezaBaseCalculo,
    ResultExt, TipoDeCredito, TipoDeOperacao, TipoDoItem, display_cst, excel_format::*,
};

// --- Macros ---

/// Dynamically casts a `dyn Any` reference to concrete types.
///
/// This macro iterates through a list of target types and executes the associated
/// expression for the first matching type found at runtime.
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

/// Memory consumption strategies for Excel file generation.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExcelMemoryMode {
    /// Constant memory profile that writes row data progressively to disk.
    /// Best for processing very large files under tight system memory constraints.
    ConstantMemory,
    /// Low memory profile focused on compact structures utilizing shared strings.
    LowMemory,
    /// Fully buffered in-memory structure processed in parallel via Rayon.
    /// Offers the highest throughput at the cost of transient memory usage.
    InMemory,
}

/// A unified context structure grouping the core datasets required for Excel processing.
///
/// This container holds borrow-slices of the raw data. It decouples high-level
/// orchestration from data ownership, allowing worksheet generation routines
/// to be attached as domain-specific methods.
pub struct AllData<'a> {
    /// Detailed list of fiscal documents and their items.
    pub efd: &'a [DocsFiscais],
    /// Consolidated records grouped by CST.
    pub cst: &'a [ConsolidacaoCST],
    /// Credit analysis data categorized by core calculations.
    pub nat: &'a [AnaliseDosCreditos],
}

impl<'a> AllData<'a> {
    /// Constructs a new `AllData` container from borrows of the input datasets.
    pub fn new(
        efd: &'a [DocsFiscais],
        cst: &'a [ConsolidacaoCST],
        nat: &'a [AnaliseDosCreditos],
    ) -> Self {
        Self { efd, cst, nat }
    }

    /// Generates all worksheets concurrently using a structured Rayon scope.
    ///
    /// This method uses concurrent worker scopes to split the rendering load
    /// of each sheet category across the CPU thread pool. The results are collected
    /// dynamically before propagating up.
    pub fn get_all_worksheets(
        &self,
        registry: &Arc<FormatRegistry>,
        multiprogressbar: &MultiProgress,
    ) -> EFDResult<Vec<Worksheet>> {
        let mut res_efd: EFDResult<Vec<Worksheet>> = Ok(Vec::new());
        let mut res_cst: EFDResult<Vec<Worksheet>> = Ok(Vec::new());
        let mut res_nat: EFDResult<Vec<Worksheet>> = Ok(Vec::new());

        // We use a Rayon scope to spawn detached logical tasks on separate threads.
        // This ensures the main thread coordinates thread-safety barriers.
        rayon::scope(|s| {
            s.spawn(|_| {
                res_efd = process_sheet_type(
                    self.efd,
                    SheetType::ItensDocsFiscais,
                    registry,
                    multiprogressbar,
                    0,
                );
            });
            s.spawn(|_| {
                res_cst = process_sheet_type(
                    self.cst,
                    SheetType::ConsolidacaoCST,
                    registry,
                    multiprogressbar,
                    1,
                );
            });
            s.spawn(|_| {
                res_nat = process_sheet_type(
                    self.nat,
                    SheetType::AnaliseCreditos,
                    registry,
                    multiprogressbar,
                    2,
                );
            });
        });

        // Safely propagate first-occurring thread errors and assemble the resulting sequence.
        let mut worksheets = res_efd?;
        worksheets.extend(res_cst?);
        worksheets.extend(res_nat?);

        Ok(worksheets)
    }

    /// Sequentially renders worksheet blocks directly into the target workbook.
    ///
    /// Intended for low-memory environments, this method processes each dataset segment
    /// sequentially, writing rows directly to streaming file targets to avoid holding
    /// fully serialized sheets in memory.
    pub fn write_sequentially_to_workbook(
        &self,
        workbook: &mut Workbook,
        registry: &Arc<FormatRegistry>,
        multiprogressbar: &MultiProgress,
        memory_mode: ExcelMemoryMode,
    ) -> EFDResult<()> {
        process_sheet_type_sequential(
            workbook,
            self.efd,
            SheetType::ItensDocsFiscais,
            registry,
            multiprogressbar,
            0,
            memory_mode,
        )?;
        process_sheet_type_sequential(
            workbook,
            self.cst,
            SheetType::ConsolidacaoCST,
            registry,
            multiprogressbar,
            1,
            memory_mode,
        )?;
        process_sheet_type_sequential(
            workbook,
            self.nat,
            SheetType::AnaliseCreditos,
            registry,
            multiprogressbar,
            2,
            memory_mode,
        )?;
        Ok(())
    }
}

// --- Core Execution Logic ---

/// Entrypoint to generate the principal Excel document.
///
/// Bundles input slices into an `AllData` context, builds physical workbook references,
/// and delegates performance pipelines depending on the requested memory profile.
pub fn create_xlsx(
    path_xlsx: &Path,
    data_efd: &[DocsFiscais],
    data_cst: &[ConsolidacaoCST],
    data_nat: &[AnaliseDosCreditos],
    memory_mode: ExcelMemoryMode,
) -> EFDResult<()> {
    let file = File::create(path_xlsx).map_loc(|e| EFDError::InOut {
        source: e,
        path: path_xlsx.to_path_buf(),
    })?;

    // Instantiate our unified data context
    let all_data = AllData::new(data_efd, data_cst, data_nat);

    let buffer = BufWriter::with_capacity(BUFFER_CAPACITY, file);
    let mut workbook = Workbook::new();
    workbook.set_properties(&get_properties()?);

    let multiprogressbar = MultiProgress::new();
    let registry = Arc::new(FormatRegistry::new());

    // Clean, idiomatic dispatch of the generation strategy.
    // In-memory layouts are parallelized, whereas streaming variants render sequentially.
    match memory_mode {
        ExcelMemoryMode::InMemory => {
            all_data
                .get_all_worksheets(&registry, &multiprogressbar)?
                .into_iter()
                .for_each(|worksheet| {
                    workbook.push_worksheet(worksheet);
                });
        }
        ExcelMemoryMode::ConstantMemory | ExcelMemoryMode::LowMemory => {
            all_data.write_sequentially_to_workbook(
                &mut workbook,
                &registry,
                &multiprogressbar,
                memory_mode,
            )?;
        }
    }

    workbook.save_to_writer(buffer)?;
    Ok(())
}

/// Processes a slice of structured data into formatted Excel worksheets in parallel.
///
/// Divides massive data arrays into standard chunks defined by the engine limits,
/// utilizing parallel chunk processing to build worksheets concurrently.
pub fn process_sheet_type<'de, T>(
    lines: &[T],
    sheet_type: SheetType,
    registry: &Arc<FormatRegistry>,
    multiprogressbar: &MultiProgress,
    index: usize,
) -> EFDResult<Vec<Worksheet>>
where
    T: Serialize + Deserialize<'de> + Iterable + ExcelExtension + Sync + Send,
{
    if lines.is_empty() {
        return Ok(Vec::new());
    }

    let progressbar = multiprogressbar.insert(index, ProgressBar::new(lines.len() as u64));
    progressbar.set_style(get_style(0, 0, 33)?);
    progressbar.set_message(format!("Excel: {}", sheet_type.as_str()));

    // par_chunks splits processing tasks safely across Rayon's thread pool
    let worksheets = lines
        .par_chunks(MAX_NUMBER_OF_ROWS)
        .enumerate()
        .map(|(k, data)| {
            let name = obter_nome_da_aba(sheet_type, k);
            generate_worksheet(data, sheet_type, &name, registry, &progressbar)
        })
        .collect::<EFDResult<Vec<_>>>()?;

    progressbar.finish();
    Ok(worksheets)
}

/// Standardized naming convention for worksheets divided into sequential blocks.
///
/// A pure, predictable function naming first sheets with standard labels and appending
/// a index count for overflow segments.
#[inline]
pub fn obter_nome_da_aba(sheet_type: SheetType, k: usize) -> String {
    if k == 0 {
        sheet_type.as_str().to_string()
    } else {
        format!("{} {}", sheet_type.as_str(), k + 1)
    }
}

/// Standardized worksheet initialization, data serialization, and post-processing.
fn generate_worksheet<'de, T>(
    lines: &[T],
    sheet_type: SheetType,
    sheet_name: &str,
    registry: &FormatRegistry,
    progressbar: &ProgressBar,
) -> EFDResult<Worksheet>
where
    T: Serialize + Deserialize<'de> + Iterable + ExcelExtension + Sync,
{
    let mut worksheet = Worksheet::new();
    worksheet.set_name(sheet_name)?;

    populate_worksheet_data(&mut worksheet, lines, sheet_type, registry, progressbar)?;

    Ok(worksheet)
}

/// Populates structural cells inside a worksheet, applying localized visual formats.
///
/// Designed to balance speed and structure:
/// 1. Extracts layouts and establishes basic formatting parameters.
/// 2. Iteratively serializes records using custom row styles.
/// 3. Safely reports completion progress.
fn populate_worksheet_data<'de, T>(
    worksheet: &mut Worksheet,
    lines: &[T],
    sheet_type: SheetType,
    registry: &FormatRegistry,
    progressbar: &ProgressBar,
) -> EFDResult<()>
where
    T: Serialize + Deserialize<'de> + Iterable + ExcelExtension + Sync,
{
    let headers = T::get_headers();

    // Pre-calculate column configurations once per worksheet chunk
    let col_configs: Vec<(u16, FormatKey)> = headers
        .iter()
        .enumerate()
        .map(|(i, &name)| (i as u16, get_format_key(name, sheet_type)))
        .collect();

    let num_cols = headers.len();
    let num_lines = lines.len();

    setup_worksheet(
        worksheet,
        num_cols as u16,
        num_lines as u32,
        &col_configs,
        registry,
    )?;

    // Writes headers and default column formats
    worksheet.deserialize_headers::<T>(0, 0)?;

    // Update progress bar at set intervals (e.g., every 1%) to reduce visual overhead
    let delta: usize = (num_lines / 100).max(1);
    let delta_u64: u64 = delta.try_into()?;

    // Functional traversal of lines to stream and style row contents
    lines
        .iter()
        .enumerate()
        .try_for_each(|(idx, line)| -> EFDResult<()> {
            let row_idx = (idx + 1) as u32;
            let style = line.row_style();

            // 1. Core serialization step
            worksheet.serialize(line)?;

            // 2. Format row override styling only if different from normal (Optimization)
            if style != RowStyle::Default {
                for (col_idx, f_key) in &col_configs {
                    if let Some(fmt) = registry.get_format(*f_key, style) {
                        worksheet.set_cell_format(row_idx, *col_idx, fmt)?;
                    }
                }
            }

            // Atualização incremental da barra de progresso
            if idx.is_multiple_of(delta) {
                progressbar.inc(delta_u64);
            }

            Ok(())
        })?;

    auto_fit(worksheet, lines, headers, sheet_type)?;

    Ok(())
}

/// Evaluates the workbook allocation strategy and registers sequential worksheets.
///
/// Avoids general in-memory buffering by using low-memory or constant-memory
/// constructs tailored specifically to disk-streaming workflows.
fn process_sheet_type_sequential<'de, T>(
    workbook: &mut Workbook,
    lines: &[T],
    sheet_type: SheetType,
    registry: &FormatRegistry,
    multiprogressbar: &MultiProgress,
    index: usize,
    memory_mode: ExcelMemoryMode,
) -> EFDResult<()>
where
    T: Serialize + Deserialize<'de> + Iterable + ExcelExtension + Sync,
{
    if lines.is_empty() {
        return Ok(());
    }

    let progressbar = multiprogressbar.insert(index, ProgressBar::new(lines.len() as u64));
    progressbar.set_style(get_style(0, 0, 33)?);
    progressbar.set_message(format!("Excel: {}", sheet_type.as_str()));

    for (k, chunk) in lines.chunks(MAX_NUMBER_OF_ROWS).enumerate() {
        let name = obter_nome_da_aba(sheet_type, k);

        // Instantiates specialized low-allocation worksheets depending on streaming configuration
        let worksheet = match memory_mode {
            ExcelMemoryMode::ConstantMemory => workbook.add_worksheet_with_constant_memory(),
            ExcelMemoryMode::LowMemory => workbook.add_worksheet_with_low_memory(),
            ExcelMemoryMode::InMemory => workbook.add_worksheet(),
        };

        worksheet.set_name(&name)?;
        populate_worksheet_data(worksheet, chunk, sheet_type, registry, &progressbar)?;
    }

    progressbar.finish();
    Ok(())
}

/// Sets up structural visual configurations for a worksheet.
///
/// Configures top header row dimensions, sets frozen navigation panes,
/// applies base column styling, and attaches an overall filterable Excel table.
fn setup_worksheet(
    worksheet: &mut Worksheet,
    num_cols: u16,
    num_lines: u32,
    configs: &[(u16, FormatKey)],
    registry: &FormatRegistry,
) -> EFDResult<()> {
    worksheet
        .set_row_height(0, 64)?
        .set_row_format(0, &FormatRegistry::header())?
        .set_freeze_panes(1, 0)?;

    for (i, f_key) in configs {
        if let Some(fmt) = registry.get_format(*f_key, RowStyle::Default) {
            worksheet.set_column_format(*i, fmt)?;
        }
    }

    let table = Table::new().set_autofilter(true);
    worksheet.add_table(0, 0, num_lines, num_cols - 1, &table)?;

    Ok(())
}

/// Adjusts column widths dynamically.
///
/// Iterates over data records across parallel workers. In order to dynamically compute
/// column layout dimensions safely without heavy lock primitives (like `Mutex`),
/// this function utilizes lock-free atomic `AtomicUsize` structures with `Relaxed`
/// ordering constraints.
fn auto_fit<'de, T>(
    worksheet: &mut Worksheet,
    lines: &[T],
    headers: &[&str],
    sheet_type: SheetType,
) -> EFDResult<()>
where
    T: Serialize + Deserialize<'de> + ExcelExtension + Iterable + Sync,
{
    // Initialize atomic storage with default limits derived from raw header lengths
    let widths: Vec<AtomicUsize> = headers
        .iter()
        .map(|h| AtomicUsize::new(WIDTH_MIN.max(h.len().div_ceil(5))))
        .collect();

    // Iterate through line records concurrently to calculate length extrema
    lines.par_iter().for_each(|line| {
        line.iter().enumerate().for_each(|(i, (_name, val))| {
            if let Some(atomic) = widths.get(i) {
                let len = calculate_value_len(sheet_type, val);
                atomic.fetch_max(len, Ordering::Relaxed);
            }
        });
    });

    // Write final resolved column widths down to the sheet structure
    for (i, atomic) in widths.into_iter().enumerate() {
        let width = (atomic.load(Ordering::Relaxed).min(WIDTH_MAX) as f64) * ADJUSTMENT;
        worksheet.set_column_width(i as u16, width)?;
    }
    Ok(())
}

/// Computes the layout width of a standard `Decimal` object.
#[inline]
fn decimal_len(d: &Decimal) -> usize {
    d.to_string().len()
}

/// Inspects field types dynamically to estimate layout widths.
///
/// Accounts for proportion differences in proportional typography (such as Calibri)
/// by applying safety factors to character counts.
fn calculate_value_len(sheet_type: SheetType, field_value: &dyn std::any::Any) -> usize {
    let len = match_cast!(field_value {
        // Tratamento do CST
        val as Option<CodigoSituacaoTributaria> => {
            val.as_ref().map_or(2, |cst| {
                if sheet_type.is_itens() {
                    // Planilha Detalhada: "01 - Operação Tributável..."
                    // Multiplicamos por 110% porque fontes proporcionais (Calibri)
                    // são levemente mais largas que a contagem de caracteres monoespaçados.
                    cst.descricao_com_codigo().len() * 70 / 100
                } else {
                    // Planilha Compacta: Apenas os dígitos "01"
                    display_cst(&Some(*cst)).len() * 80 / 100
                }
            })
        },

        val as Option<NaturezaBaseCalculo> => {
            val.as_ref().map_or(0, |n| {
                n.descricao_com_codigo().len() * 72 / 100
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
