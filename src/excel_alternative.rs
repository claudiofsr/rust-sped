use chrono::NaiveDate;
use claudiofsr_lib::{IntegerDigits, OptionExtension, get_style};
use csv::StringRecord;
use indicatif::{MultiProgress, ProgressBar};
use rayon::prelude::*;
use rust_decimal::{Decimal, prelude::ToPrimitive};
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::BufWriter,
    path::Path,
    sync::Arc,
    thread,
};

use rust_xlsxwriter::{
    Color, DocProperties, ExcelDateTime, Format, FormatAlign, Workbook, Worksheet,
};

use crate::{
    AnaliseDosCreditos, BUFFER_CAPACITY, CSTOption, ConsolidacaoCST, DECIMAL_VALOR, DocsFiscais,
    EFDError, EFDResult, MesesDoAno, ResultExt, display_aliquota, display_cst,
    obter_descricao_do_cfop,
};

const FONT_SIZE: f64 = 12.0;
const HEADER_FONT_SIZE: f64 = 11.0;
const MAX_NUMBER_OF_ROWS: usize = 1_000_000;

// --------------- Trait --------------

/// Trait único para qualquer dado que vá para o Excel
pub trait ToExcel {
    fn write_to(self, sheet: &mut Worksheet, row: u32, col: u16, fmt: &Format) -> EFDResult<usize>;
}

// --- Implementação para Números ---
macro_rules! impl_to_excel_num {
    ($($t:ty),*) => {
        $(
            impl ToExcel for $t {
                fn write_to(self, sheet: &mut Worksheet, row: u32, col: u16, fmt: &Format) -> EFDResult<usize> {
                    if let Some(val_f64) = self.to_f64() {
                        sheet.write_number_with_format(row, col, val_f64, fmt)?;
                        Ok(self.digit_count())
                    } else {
                        sheet.write_blank(row, col, fmt)?;
                        Ok(1)
                    }
                }
            }
        )*
    };
}
impl_to_excel_num!(u8, u16, u32, u64, usize, i32, i64);

// --- Implementação para Strings ---
macro_rules! impl_to_excel_str {
    ($($t:ty),*) => {
        $(
            impl ToExcel for $t {
                fn write_to(self, sheet: &mut Worksheet, row: u32, col: u16, fmt: &Format) -> EFDResult<usize> {
                    let s: &str = self.as_ref();
                    sheet.write_string_with_format(row, col, s, fmt)?;
                    Ok(s.chars().count())
                }
            }
        )*
    };
}
impl_to_excel_str!(
    &str,
    String,
    &String,
    Arc<str>,
    &Arc<str>,
    compact_str::CompactString,
    &compact_str::CompactString
);

// --- Implementação Genérica para Option<T> ---
impl<T: ToExcel> ToExcel for Option<T> {
    fn write_to(self, sheet: &mut Worksheet, row: u32, col: u16, fmt: &Format) -> EFDResult<usize> {
        match self {
            Some(v) => v.write_to(sheet, row, col, fmt),
            None => {
                sheet.write_blank(row, col, fmt)?;
                Ok(1)
            }
        }
    }
}

// --------------- Struct --------------

/// Gerenciador dinâmico de linhas e colunas para Excel.
struct RowWriter<'a> {
    sheet: &'a mut Worksheet,
    row: u32,
    col: u16,
    width_map: &'a mut BTreeMap<u16, usize>,
}

impl<'a> RowWriter<'a> {
    fn new(sheet: &'a mut Worksheet, row: u32, width_map: &'a mut BTreeMap<u16, usize>) -> Self {
        Self {
            sheet,
            row: row + 1, // Compensar header
            col: 0,
            width_map,
        }
    }

    fn update_width(&mut self, len: usize) {
        self.width_map
            .entry(self.col)
            .and_modify(|prev| {
                if len > *prev {
                    *prev = len
                }
            })
            .or_insert(len);
    }

    // Método único para quase tudo (Strings, Inteiros, Options)
    fn cell<T: ToExcel>(&mut self, data: T, fmt: &Format) -> EFDResult<&mut Self> {
        let len = data.write_to(self.sheet, self.row, self.col, fmt)?;
        self.update_width(len);
        self.col += 1;
        Ok(self)
    }

    fn date(&mut self, date: Option<NaiveDate>, fmt: &Format) -> EFDResult<&mut Self> {
        match date {
            Some(dt) => self
                .sheet
                .write_datetime_with_format(self.row, self.col, dt, fmt)?,
            None => self.sheet.write_blank(self.row, self.col, fmt)?,
        };
        self.update_width(10);
        self.col += 1;
        Ok(self)
    }

    fn decimal<S>(&mut self, val: S, fmt: &Format) -> EFDResult<&mut Self>
    where
        S: Into<Option<Decimal>>,
    {
        // Converte automaticamente Decimal ou Option<Decimal> para Option<Decimal>
        let opt_val: Option<Decimal> = val.into();

        let len = match opt_val {
            Some(d) if !d.is_zero() => {
                // 1. Escrita segura do valor numérico f64
                if let Some(num_f64) = d.to_f64() {
                    self.sheet
                        .write_number_with_format(self.row, self.col, num_f64, fmt)?;
                } else {
                    self.sheet.write_blank(self.row, self.col, fmt)?;
                }

                // 2. Cálculo seguro da largura da coluna
                let integer_part = d.round().to_usize().unwrap_or(0);
                integer_part.digit_count() + 4 + DECIMAL_VALOR
            }
            _ => {
                // Caso seja None ou Zero, limpa a célula
                self.sheet.write_blank(self.row, self.col, fmt)?;
                1
            }
        };

        self.update_width(len);
        self.col += 1;
        Ok(self)
    }
}

/// Create excel xlsx file from data_efd, data_cst, data_nat.
///
/// data_efd can be divided into N Worksheets generated by N different threads.
pub fn create_xlsx(
    path_xlsx: &Path,
    data_efd: &[DocsFiscais],
    data_cst: &[ConsolidacaoCST],
    data_nat: &[AnaliseDosCreditos],
) -> EFDResult<()> {
    let file = File::create(path_xlsx).map_loc(|e| EFDError::InOut {
        source: e,
        path: path_xlsx.to_path_buf(),
    })?;

    let buffer = BufWriter::with_capacity(BUFFER_CAPACITY, file);

    let mut workbook = Workbook::new();
    let formats = create_formats()?;
    let properties = get_properties()?;
    workbook.set_properties(&properties);

    let multiprogressbar = MultiProgress::new();
    let worksheets = get_all_worksheets(data_efd, data_cst, data_nat, &formats, &multiprogressbar)?;

    for worksheet in worksheets {
        workbook.push_worksheet(worksheet);
    }

    workbook.save_to_writer(buffer)?;
    Ok(())
}

fn get_properties() -> EFDResult<DocProperties> {
    // Create a datetime object.
    let date = ExcelDateTime::from_ymd(2025, 1, 1)?.and_hms(0, 0, 0)?;

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

fn get_all_worksheets(
    data_efd: &[DocsFiscais],
    data_cst: &[ConsolidacaoCST],
    data_nat: &[AnaliseDosCreditos],
    formats: &HashMap<String, Format>,
    multiprogressbar: &MultiProgress,
) -> EFDResult<Vec<Worksheet>> {
    // Use std::thread in the following functions (these functions are independent of each other):
    let results = thread::scope(|s| {
        let thread_efd = s.spawn(|| add_worksheet_efd(data_efd, formats, multiprogressbar, 0));
        let thread_cst = s.spawn(|| add_worksheet_cst(data_cst, formats, multiprogressbar, 1));
        let thread_nat = s.spawn(|| add_worksheet_nat(data_nat, formats, multiprogressbar, 2));

        // Wait for background thread to complete.
        // Call join() on each handle to make sure all the threads finish.
        // join() returns immediately when the associated thread completes.

        [thread_efd, thread_cst, thread_nat]
            .into_iter()
            .flat_map(|scoped_join_handle| scoped_join_handle.join())
            .collect::<Vec<_>>()
    });

    let worksheets: Vec<Worksheet> = results
        .into_par_iter() // rayon: parallel iterator
        .flat_map(|res| res.expect("Failed to generate worksheet!"))
        .collect();

    Ok(worksheets)
}

fn add_worksheet_efd(
    data_efd: &[DocsFiscais],
    formats: &HashMap<String, Format>,
    multiprogressbar: &MultiProgress,
    index: usize,
) -> EFDResult<Vec<Worksheet>> {
    let mut worksheets = Vec::new();
    let progressbar = multiprogressbar.insert(index, ProgressBar::new(data_efd.len() as u64));
    progressbar.set_style(get_style(0, 0, 35)?);

    for (k, data) in data_efd.chunks(MAX_NUMBER_OF_ROWS).enumerate() {
        let sheet_name = if k == 0 {
            "Itens de Docs Fiscais".to_string()
        } else {
            format!("Itens de Docs Fiscais {}", k + 1)
        };
        progressbar.set_message(format!("Write Excel: {sheet_name}"));

        let mut ws = Worksheet::new();
        ws.set_name(&sheet_name)?;
        let mut width_map = BTreeMap::new();

        create_headers(
            &DocsFiscais::get_headers(),
            &mut ws,
            &mut width_map,
            formats,
            "data_efd",
        )?;

        for (j, colunas) in data.iter().enumerate() {
            add_row_efd(j as u32, colunas, &mut ws, formats, &mut width_map)?;
            progressbar.inc(1);
        }

        set_max_width(&mut ws, &width_map, 1.04)?;
        worksheets.push(ws);
    }
    progressbar.finish();
    Ok(worksheets)
}

// CST: CONSOLIDAÇÃO DAS OPERAÇÕES POR CST
fn add_worksheet_cst(
    data_cst: &[ConsolidacaoCST],
    formats: &HashMap<String, Format>,
    multiprogressbar: &MultiProgress,
    index: usize,
) -> EFDResult<Vec<Worksheet>> {
    let mut worksheets = Vec::new();
    let progressbar = multiprogressbar.insert(index, ProgressBar::new(data_cst.len() as u64));
    progressbar.set_style(get_style(0, 0, 35)?);

    for (k, data) in data_cst.chunks(MAX_NUMBER_OF_ROWS).enumerate() {
        let sheet_name = if k == 0 {
            "Consolidação CST".to_string()
        } else {
            format!("Consolidação CST {}", k + 1)
        };
        progressbar.set_message(format!("Write Excel: {sheet_name}"));

        let mut ws = Worksheet::new();
        ws.set_name(&sheet_name)?;
        let mut width_map = BTreeMap::new();

        create_headers(
            &ConsolidacaoCST::get_headers(),
            &mut ws,
            &mut width_map,
            formats,
            "data_cst",
        )?;

        for (j, colunas) in data.iter().enumerate() {
            add_row_cst(j as u32, colunas, &mut ws, formats, &mut width_map)?;
            progressbar.inc(1);
        }

        set_max_width(&mut ws, &width_map, 1.00)?;
        worksheets.push(ws);
    }
    progressbar.finish();
    Ok(worksheets)
}

fn add_worksheet_nat(
    data_nat: &[AnaliseDosCreditos],
    formats: &HashMap<String, Format>,
    multiprogressbar: &MultiProgress,
    index: usize,
) -> EFDResult<Vec<Worksheet>> {
    let mut worksheets = Vec::new();
    let progressbar = multiprogressbar.insert(index, ProgressBar::new(data_nat.len() as u64));
    progressbar.set_style(get_style(0, 0, 35)?);

    for (k, data) in data_nat.chunks(MAX_NUMBER_OF_ROWS).enumerate() {
        let sheet_name = if k == 0 {
            "Análise dos Créditos".to_string()
        } else {
            format!("Análise dos Créditos {}", k + 1)
        };
        progressbar.set_message(format!("Write Excel: {sheet_name}"));

        let mut ws = Worksheet::new();
        ws.set_name(&sheet_name)?;
        let mut width_map = BTreeMap::new();

        create_headers(
            &AnaliseDosCreditos::get_headers(),
            &mut ws,
            &mut width_map,
            formats,
            "data_nat",
        )?;

        for (j, colunas) in data.iter().enumerate() {
            add_row_nat(j as u32, colunas, &mut ws, formats, &mut width_map)?;
            progressbar.inc(1);
        }

        set_max_width(&mut ws, &width_map, 1.00)?;
        worksheets.push(ws);
    }
    progressbar.finish();
    Ok(worksheets)
}

fn add_row_efd(
    row: u32,
    col: &DocsFiscais,
    sheet: &mut Worksheet,
    fmt: &HashMap<String, Format>,
    width_map: &mut BTreeMap<u16, usize>,
) -> EFDResult<()> {
    let f = |n: &str| {
        fmt.get(n)
            .map_loc(|_| EFDError::FormatNotFound(n.to_string()))
    };

    #[rustfmt::skip]
    RowWriter::new(sheet, row, width_map)
        .cell(row + 2, f("integer")?)? // Valor direto T
        .cell(col.arquivo_efd.clone(), f("default")?)?
        .cell(col.num_linha_efd, f("integer")?)? // Option<T>
        .cell(&col.estabelecimento_cnpj, f("center")?)?
        .cell(&col.estabelecimento_nome, f("default")?)?
        .date(col.periodo_de_apuracao, f("date")?)?
        .cell(col.ano, f("integer")?)?
        .cell(col.trimestre, f("integer")?)?
        .cell(month_to_str(&col.mes), f("center")?)?
        .cell(col.tipo_de_operacao.to_string(), f("default")?)?
        .cell(col.indicador_de_origem.to_string(), f("default")?)?
        .cell(col.cod_credito.map(|c| c.to_u16()), f("integer")?)?
        .cell(col.tipo_de_credito.map(|t| t.descricao_com_codigo()), f("default")?)?
        .cell(&col.registro, f("default")?)?
        .cell(col.cst.descricao(), f("default")?)?
        .cell(obter_descricao_do_cfop(col.cfop), f("default")?)?
        .cell(col.natureza_bc.map(|n| n.descricao_com_codigo()), f("default")?)?
        .cell(&col.participante_cnpj, f("center")?)?
        .cell(&col.participante_cpf, f("center")?)?
        .cell(&col.participante_nome, f("default")?)?
        .cell(col.num_doc, f("default")?)?
        .cell(&col.chave_doc, f("center")?)?
        .cell(&col.modelo_doc_fiscal, f("center")?)?
        .cell(col.num_item, f("integer")?)?
        .cell(col.tipo_item.map(|t| t.descricao_com_codigo()), f("default")?)?
        .cell(&col.descr_item, f("default")?)?
        .cell(&col.cod_ncm, f("center")?)?
        .cell(&col.nat_operacao, f("default")?)?
        .cell(&col.complementar, f("default")?)?
        .cell(&col.nome_da_conta, f("default")?)?
        .date(col.data_emissao, f("date")?)?
        .date(col.data_entrada, f("date")?)?
        .decimal(col.valor_item, f("number")?)?
        .decimal(col.valor_bc, f("number")?)?
        .decimal(col.aliq_pis, f("aliquota")?)?
        .decimal(col.aliq_cofins, f("aliquota")?)?
        .decimal(col.valor_pis, f("number")?)?
        .decimal(col.valor_cofins, f("number")?)?
        .decimal(col.valor_iss, f("number")?)?
        .decimal(col.valor_bc_icms, f("number")?)?
        .decimal(col.aliq_icms, f("aliquota")?)?
        .decimal(col.valor_icms, f("number")?)?;

    let height = if row == 0 {
        HEADER_FONT_SIZE + 40.0
    } else {
        FONT_SIZE + 3.0
    };
    sheet.set_row_height(row, height)?;
    Ok(())
}

fn add_row_cst(
    row: u32,
    col: &ConsolidacaoCST,
    sheet: &mut Worksheet,
    fmt: &HashMap<String, Format>,
    width_map: &mut BTreeMap<u16, usize>,
) -> EFDResult<()> {
    let suffix = if let Some(490 | 980) = col.cst.code() {
        "_bcsoma"
    } else {
        ""
    };
    let f = |n: &str| {
        fmt.get(&format!("{n}{suffix}"))
            .map_loc(|_| EFDError::FormatNotFound(n.to_string()))
    };

    #[rustfmt::skip]
    RowWriter::new(sheet, row, width_map)
        .cell(&col.cnpj_base, f("center")?)?
        .cell(col.ano, f("integer")?)?
        .cell(col.trimestre, f("integer")?)?
        .cell(month_to_str(&col.mes), f("center")?)?
        .cell(display_cst(&col.cst), f("center")?)?
        .decimal(Some(col.valor_item), f("number")?)?
        .decimal(Some(col.valor_bc), f("number")?)?
        .decimal(Some(col.valor_pis), f("number")?)?
        .decimal(Some(col.valor_cofins), f("number")?)?;

    let height = if row == 0 {
        HEADER_FONT_SIZE + 40.0
    } else {
        FONT_SIZE + 3.0
    };
    sheet.set_row_height(row, height)?;
    Ok(())
}

fn add_row_nat(
    row: u32,
    col: &AnaliseDosCreditos,
    sheet: &mut Worksheet,
    fmt: &HashMap<String, Format>,
    width_map: &mut BTreeMap<u16, usize>,
) -> EFDResult<()> {
    let suffix = match col.natureza_bc.map(|n| n.code()) {
        Some(101..=199 | 300) => "_bcsoma",
        Some(221 | 225) => "_descon",
        Some(301 | 305) => "_saldoc",
        _ => "",
    };
    let f = |n: &str| {
        fmt.get(&format!("{n}{suffix}"))
            .map_loc(|_| EFDError::FormatNotFound(n.to_string()))
    };

    #[rustfmt::skip]
    RowWriter::new(sheet, row, width_map)
        .cell(&col.cnpj_base, f("center")?)?
        .cell(col.ano, f("integer")?)?
        .cell(col.trimestre, f("integer")?)?
        .cell(month_to_str(&col.mes), f("center")?)?
        .cell(col.tipo_de_operacao.to_string(), f("default")?)?
        .cell(col.tipo_de_credito.map(|t| t.to_string()), f("center")?)?
        .cell(col.cst.code(), f("integer")?)?
        .cell(display_aliquota(&col.aliq_pis), f("center")?)?
        .cell(display_aliquota(&col.aliq_cofins), f("center")?)?
        .cell(col.natureza_bc.map(|n| n.descricao_com_codigo()), f("default")?)?
        .decimal(col.valor_bc, f("number")?)?
        .decimal(col.valor_rbnc_trib, f("number")?)?
        .decimal(col.valor_rbnc_ntrib, f("number")?)?
        .decimal(col.valor_rbnc_exp, f("number")?)?
        .decimal(col.valor_rb_cum, f("number")?)?;

    let height = if row == 0 {
        HEADER_FONT_SIZE + 50.0
    } else {
        FONT_SIZE + 3.0
    };
    sheet.set_row_height(row, height)?;
    Ok(())
}

fn create_formats() -> EFDResult<HashMap<String, Format>> {
    let parse_color = |hex: &str| {
        u32::from_str_radix(hex, 16).map_loc(|e| EFDError::ParseIntError(e, hex.to_string()))
    };

    let mut hash_map = HashMap::new();
    let fmt_header = Format::new()
        .set_text_wrap()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_font_size(HEADER_FONT_SIZE)
        .set_background_color(Color::RGB(parse_color("c5d9f1")?));

    hash_map.insert("header".to_string(), fmt_header);

    let base_formats = vec![
        (
            "default",
            Format::new()
                .set_align(FormatAlign::VerticalCenter)
                .set_font_size(FONT_SIZE),
        ),
        (
            "center",
            Format::new()
                .set_align(FormatAlign::Center)
                .set_align(FormatAlign::VerticalCenter)
                .set_font_size(FONT_SIZE),
        ),
        (
            "date",
            Format::new()
                .set_align(FormatAlign::Center)
                .set_align(FormatAlign::VerticalCenter)
                .set_num_format("dd/mm/yyyy")
                .set_font_size(FONT_SIZE),
        ),
        (
            "number",
            Format::new()
                .set_align(FormatAlign::VerticalCenter)
                .set_num_format("#,##0.00")
                .set_font_size(FONT_SIZE),
        ),
        (
            "aliquota",
            Format::new()
                .set_align(FormatAlign::VerticalCenter)
                .set_num_format("0.0000")
                .set_font_size(FONT_SIZE),
        ),
        (
            "integer",
            Format::new()
                .set_align(FormatAlign::Center)
                .set_align(FormatAlign::VerticalCenter)
                .set_num_format("#")
                .set_font_size(FONT_SIZE),
        ),
    ];

    let colors = vec![
        ("", None),
        ("_bcsoma", Some("bfbfbf")),
        ("_descon", Some("fff2cc")),
        ("_saldoc", Some("f8cbad")),
    ];

    for (suffix, hex) in colors {
        for (name, fmt) in &base_formats {
            let mut f = fmt.clone();
            if let Some(c) = hex {
                f = f.set_background_color(Color::RGB(parse_color(c)?));
            }
            hash_map.insert(format!("{name}{suffix}"), f);
        }
    }

    Ok(hash_map)
}

fn create_headers(
    headers: &StringRecord,
    sheet: &mut Worksheet,
    width_map: &mut BTreeMap<u16, usize>,
    fmt: &HashMap<String, Format>,
    tipo: &str,
) -> EFDResult<()> {
    let fmt_header = fmt.get("header").unwrap();
    let mut last_col = 0;

    for (idx, header) in headers.iter().enumerate() {
        let col = idx as u16;
        sheet.write_string_with_format(0, col, header, fmt_header)?;

        let mut width = header.len();
        match tipo {
            "data_efd" => {
                // definir largura mínima de colunas específicas
                let valor_ou_aliquota = header.contains("Valor") || header.contains("Alíquota");

                if idx == 2 || header.contains("Período de Apuração") || valor_ou_aliquota {
                    width = 12;
                } else if header.contains("CNPJ") || header.contains("Data") {
                    width = 18;
                }
            }
            "data_cst" => width = 12,
            "data_nat" => {
                // definir largura mínima
                width = 12;
                // definir largura mínima de colunas específicas
                if idx == 5 {
                    width = 6;
                } else if idx >= 9 {
                    width = 18;
                }
            }
            _ => {}
        }

        width_map.insert(col, width);
        last_col = col;
    }

    sheet.autofilter(0, 0, 0, last_col)?;
    sheet.set_freeze_panes(1, 0)?;
    Ok(())
}

fn set_max_width(
    sheet: &mut Worksheet,
    width_map: &BTreeMap<u16, usize>,
    fator: f64,
) -> EFDResult<()> {
    for (&k, &v) in width_map {
        sheet.set_column_width(k, (v as f64) * fator)?;
    }
    Ok(())
}

pub fn month_to_str(mes: &Option<MesesDoAno>) -> String {
    match mes {
        Some(m) => m.to_string(),
        None => "".to_string(),
    }
}
