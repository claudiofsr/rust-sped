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

/// Modos de consumo de memória para a geração do arquivo Excel.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExcelMemoryMode {
    /// Consumo de memória constante, que grava os dados diretamente no disco de forma contínua.
    ConstantMemory,
    /// Baixa memória, focado na geração de arquivos compactos através do uso de strings compartilhadas.
    LowMemory,
    /// Armazena toda a estrutura em memória RAM (via processamento paralelo com Rayon).
    InMemory,
}

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

// --- Lógica Principal ---

/// Gera o arquivo Excel principal.
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

    let buffer = BufWriter::with_capacity(BUFFER_CAPACITY, file);
    let mut workbook = Workbook::new();
    workbook.set_properties(&get_properties()?);

    let multiprogressbar = MultiProgress::new();
    // Passamos o registro de formatos centralizado
    let registry = Arc::new(FormatRegistry::new());

    match memory_mode {
        ExcelMemoryMode::InMemory => {
            // Preserva a lógica de paralelização via Rayon
            let worksheets =
                get_all_worksheets(data_efd, data_cst, data_nat, &registry, &multiprogressbar)?;
            for worksheet in worksheets {
                workbook.push_worksheet(worksheet);
            }
        }
        ExcelMemoryMode::ConstantMemory | ExcelMemoryMode::LowMemory => {
            // Executa sequencialmente gravando diretamente no buffer do Workbook
            write_sequentially_to_workbook(
                &mut workbook,
                data_efd,
                data_cst,
                data_nat,
                &registry,
                &multiprogressbar,
                memory_mode,
            )?;
        }
    }

    workbook.save_to_writer(buffer)?;
    Ok(())
}

/// Gera todas as planilhas em paralelo usando Rayon scope.
///
/// This approach uses `rayon::scope` with safe default initialization.
/// Since `Vec::new()` does not perform heap allocations until elements are pushed,
/// initializing with `Ok(Vec::new())` is efficient and avoids `expect`/`unwrap` panics.
pub fn get_all_worksheets(
    data_efd: &[DocsFiscais],
    data_cst: &[ConsolidacaoCST],
    data_nat: &[AnaliseDosCreditos],
    registry: &Arc<FormatRegistry>,
    multiprogressbar: &MultiProgress,
) -> EFDResult<Vec<Worksheet>> {
    // Initializing with Ok(Vec::new()) has zero heap-allocation cost.
    let mut res_efd: EFDResult<Vec<Worksheet>> = Ok(Vec::new());
    let mut res_cst: EFDResult<Vec<Worksheet>> = Ok(Vec::new());
    let mut res_nat: EFDResult<Vec<Worksheet>> = Ok(Vec::new());

    // Process all sheets in parallel scopes
    rayon::scope(|s| {
        s.spawn(|_| {
            res_efd = process_sheet_type(
                data_efd,
                SheetType::ItensDocsFiscais,
                registry,
                multiprogressbar,
                0,
            )
        });
        s.spawn(|_| {
            res_cst = process_sheet_type(
                data_cst,
                SheetType::ConsolidacaoCST,
                registry,
                multiprogressbar,
                1,
            )
        });
        s.spawn(|_| {
            res_nat = process_sheet_type(
                data_nat,
                SheetType::AnaliseCreditos,
                registry,
                multiprogressbar,
                2,
            )
        });
    });

    // Safely propagate potential errors and extend the vectors
    let mut worksheets = res_efd?;
    worksheets.extend(res_cst?);
    worksheets.extend(res_nat?);

    Ok(worksheets)
}

/// Gera as worksheets aplicando a serialização automática do Serde (Usado Rayon)
pub fn process_sheet_type<'de, T>(
    lines: &[T],
    sheet_type: SheetType,
    registry: &Arc<FormatRegistry>,
    multiprogressbar: &MultiProgress,
    index: usize,
) -> EFDResult<Vec<Worksheet>>
where
    // T: Sync é necessário para par_chunks (referência compartilhada entre threads)
    // T: Send é necessário para mover o processamento para outras threads
    T: Serialize + Deserialize<'de> + Iterable + ExcelExtension + Sync + Send,
{
    if lines.is_empty() {
        return Ok(Vec::new());
    }

    let progressbar = multiprogressbar.insert(index, ProgressBar::new(lines.len() as u64));
    progressbar.set_style(get_style(0, 0, 33)?);
    progressbar.set_message(format!("Excel: {}", sheet_type.as_str()));

    // par_chunks é o método nativo do Rayon para slices.
    // Ele retorna um ParallelIterator diretamente.
    let worksheets = lines
        .par_chunks(MAX_NUMBER_OF_ROWS)
        .enumerate()
        .map(|(k, data)| {
            let name = obter_nome_da_aba(sheet_type, k);

            // generate_worksheet será executado em paralelo para cada chunk
            generate_worksheet(data, sheet_type, &name, registry, &progressbar)
        })
        .collect::<EFDResult<Vec<_>>>()?;

    progressbar.finish();
    Ok(worksheets)
}

/// Gera o nome formatado da aba (worksheet) com base no tipo de planilha e no índice do lote (chunk).
///
/// Esta é uma função pura (sem efeitos colaterais) que unifica a regra de nomenclatura
/// para divisões de lotes volumosos de dados.
#[inline]
pub fn obter_nome_da_aba(sheet_type: SheetType, k: usize) -> String {
    if k == 0 {
        sheet_type.as_str().to_string()
    } else {
        format!("{} {}", sheet_type.as_str(), k + 1)
    }
}

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

/// Rotina unificada para preenchimento dos dados (DRY)
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

    // Aplica cabeçalhos e formatos de coluna padrão
    worksheet.deserialize_headers::<T>(0, 0)?;

    // Define a frequência de atualização da barra de progresso (ex: a cada 1%)
    let delta: usize = (num_lines / 100).max(1);

    // Converte usize para u64 de forma segura uma única vez
    let delta_u64: u64 = delta.try_into()?;

    // Processamento funcional das linhas
    lines
        .iter()
        .enumerate()
        .try_for_each(|(idx, line)| -> EFDResult<()> {
            let row_idx = (idx + 1) as u32;
            let style = line.row_style();

            // 1. Serialização base (escreve os dados conforme o #[serde])
            // Escreve os dados (Serialização é a parte mais pesada)
            worksheet.serialize(line)?;

            // 2. Aplica cores apenas se não for estilo Normal (Otimização)
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

/// Rotina sequencial para contornar a limitação de instanciação do rust_xlsxwriter
fn write_sequentially_to_workbook(
    workbook: &mut Workbook,
    data_efd: &[DocsFiscais],
    data_cst: &[ConsolidacaoCST],
    data_nat: &[AnaliseDosCreditos],
    registry: &Arc<FormatRegistry>,
    multiprogressbar: &MultiProgress,
    memory_mode: ExcelMemoryMode,
) -> EFDResult<()> {
    process_sheet_type_sequential(
        workbook,
        data_efd,
        SheetType::ItensDocsFiscais,
        registry,
        multiprogressbar,
        0,
        memory_mode,
    )?;
    process_sheet_type_sequential(
        workbook,
        data_cst,
        SheetType::ConsolidacaoCST,
        registry,
        multiprogressbar,
        1,
        memory_mode,
    )?;
    process_sheet_type_sequential(
        workbook,
        data_nat,
        SheetType::AnaliseCreditos,
        registry,
        multiprogressbar,
        2,
        memory_mode,
    )?;
    Ok(())
}

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
    progressbar.set_message(format!("Excel (Streaming): {}", sheet_type.as_str()));

    for (k, chunk) in lines.chunks(MAX_NUMBER_OF_ROWS).enumerate() {
        let name = obter_nome_da_aba(sheet_type, k);

        // Instancia a planilha utilizando a API correta do Workbook correspondente
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

/// Helper para configurar o esqueleto da worksheet
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

    // Aplica formatos base às colunas
    for (i, f_key) in configs {
        if let Some(fmt) = registry.get_format(*f_key, RowStyle::Default) {
            worksheet.set_column_format(*i, fmt)?;
        }
    }

    let table = Table::new().set_autofilter(true);
    worksheet.add_table(0, 0, num_lines, num_cols - 1, &table)?;

    Ok(())
}

/// Ajusta a largura das colunas dinamicamente em paralelo usando Rayon.
fn auto_fit<'de, T>(
    worksheet: &mut Worksheet,
    lines: &[T],
    headers: &[&str],
    sheet_type: SheetType,
) -> EFDResult<()>
where
    T: Serialize + Deserialize<'de> + ExcelExtension + Iterable + Sync,
{
    let widths: Vec<AtomicUsize> = headers
        .iter()
        .map(|h| AtomicUsize::new(WIDTH_MIN.max(h.len().div_ceil(5))))
        .collect();

    lines.par_iter().for_each(|line| {
        line.iter().enumerate().for_each(|(i, (_name, val))| {
            if let Some(atomic) = widths.get(i) {
                let len = calculate_value_len(sheet_type, val);
                atomic.fetch_max(len, Ordering::Relaxed);
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
