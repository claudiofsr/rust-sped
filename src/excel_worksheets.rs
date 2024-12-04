use chrono::NaiveDate;
use claudiofsr_lib::{get_style, match_cast, StrExtension};
use indicatif::{ProgressBar, MultiProgress};
use itertools::{self, Itertools};
use rust_xlsxwriter::{
    //Color,
    Format,
    FormatAlign,
    Table,
    Worksheet,
};
use serde::{Serialize, Deserialize};
use serde_aux::prelude::serde_introspect;
use struct_iterable::Iterable;
use std::collections::HashMap;

// use rayon::prelude::*;

use crate::{
    display_cst,
    display_natureza,
    display_tipo_de_operacao,
    indicador_de_origem_to_str,
    obter_descricao_do_tipo_de_credito,
    MyResult,
    REGEX_ALIQ, REGEX_CENTER,
    REGEX_DATE, REGEX_VALUE,
};

const FONT_SIZE: f64 = 11.0;
const HEADER_FONT_SIZE: f64 = 10.0;
const MAX_NUMBER_OF_ROWS: usize = 1_000_000;
const WIDTH_MIN: usize = 8;
const WIDTH_MAX: usize = 90;
const ADJUSTMENT: f64 = 1.2;

/// Add some methods to Info struct
///
/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
pub trait InfoExtension {

    /**
    Gets the serialization names for structs and enums.

    use serde_aux::prelude::serde_introspect;

    <https://docs.rs/serde-aux/latest/src/serde_aux/serde_introspection.rs.html>
    */
    fn get_headers<'de>() -> &'static [&'static str]
    where
        Self: Deserialize<'de>
    {
        serde_introspect::<Self>()
    }
}

/// Get Worksheets according to some struct T
///
/// The lines (or rows) are given by `&[T]`
///
/// <https://docs.rs/rust_xlsxwriter/latest/rust_xlsxwriter/serializer/index.html>
pub fn get_worksheets<'de, T>(
    lines: &[T],
    sheet_name: &str,
    multiprogressbar: &MultiProgress,
    index: usize,
) -> MyResult<Vec<Worksheet>>
where
    T: Serialize + Deserialize<'de> + InfoExtension + Iterable // + Sync + Send
{
    if lines.is_empty() {
        return Ok(Vec::new());
    }

    let progressbar: ProgressBar = multiprogressbar.insert(
        index,
        ProgressBar::new(lines.len().try_into()?)
    );
    let style = get_style(0, 0, 35)?;
    progressbar.set_style(style);

    let msg: String = format!("Write Excel: {sheet_name}");
    progressbar.set_message(msg);

    let mut worksheets: Vec<Worksheet> = Vec::new();

    // Split a vector into smaller vectors of size N
    for (k, data) in lines.chunks(MAX_NUMBER_OF_ROWS).enumerate() {
        let mut new_name = sheet_name.to_string();

        if k >= 1 {
            new_name = format!("{} {}", sheet_name, k + 1);
        }

        // Get worksheet with sheet name.
        let worksheet: Worksheet = get_worksheet(data, &new_name, &progressbar)?;

        worksheets.push(worksheet);
    }

    progressbar.finish();
    Ok(worksheets)
}

/// Get Worksheet according to some struct T
fn get_worksheet<'de, T>(
    lines: &[T],
    sheet_name: &str,
    progressbar: &ProgressBar,
) -> MyResult<Worksheet>
where
    T: Serialize + Deserialize<'de> + InfoExtension + Iterable // + Sync + Send
{
    let column_names: &[&str] = T::get_headers(); // <-- InfoExtension
    let column_number: u16 = column_names.len().try_into()?;
    let row_number: u32 = lines.len().try_into()?;

    // println!("column_names: {column_names:#?}");

    // Add some formats to use with the serialization data.
    let fmt: HashMap<&str, Format> = create_formats();

    let mut worksheet = Worksheet::new();

    worksheet
        .set_name(sheet_name)?
        .set_row_height(0, 64)?
        .set_row_format(0, fmt.get("header").unwrap())?
        .set_freeze_panes(1, 0)?;

    // Set up the start location and headers of the data to be serialized.
    worksheet.deserialize_headers::<T>(0, 0)?;

    format_lines_and_columns(&mut worksheet, lines, column_names, &fmt)?;

    // Create and configure a new table.
    // Why LibreOffice Calc not recognize the table styles?
    let table = Table::new()
        .set_autofilter(true)
        .set_total_row(false);

    // Add the table to the worksheet.
    worksheet.add_table(0, 0, row_number, column_number - 1, &table)?;

    for line in lines {
        // Serialize the data.
        worksheet.serialize(line)?;
        progressbar.inc(1);
    }

    /*
    lines
        .iter()
        .try_for_each(|line| -> MyResult<()> {
            // Serialize the data.
            worksheet.serialize(line)?;
            Ok(())
        })?;
    */

    // worksheet.autofit();
    auto_fit(&mut worksheet, lines, column_names)?;

    Ok(worksheet)
}

/// Add some formats to use with the serialization data.
fn create_formats() -> HashMap<&'static str, Format> {

    let fmt_default: Format = Format::new()
        .set_align(FormatAlign::VerticalCenter)
        .set_font_size(FONT_SIZE);

    let fmt_header: Format = Format::new()
        .set_align(FormatAlign::VerticalCenter)
        .set_align(FormatAlign::Center) // horizontally
        .set_text_wrap()
        .set_font_size(HEADER_FONT_SIZE);

    let fmt_center = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_font_size(FONT_SIZE);

    let fmt_value = Format::new()
        .set_align(FormatAlign::VerticalCenter)
        .set_num_format("#,##0.00") // 2 digits after the decimal point
        .set_font_size(FONT_SIZE);

    let fmt_aliq = Format::new()
        .set_align(FormatAlign::VerticalCenter)
        .set_align(FormatAlign::Center) // horizontally
        .set_num_format("#,##0.0000") // 4 digits after the decimal point
        .set_font_size(FONT_SIZE);

    let fmt_date: Format = Format::new()
        .set_align(FormatAlign::VerticalCenter)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_num_format("dd/mm/yyyy")
        .set_font_size(FONT_SIZE);

    HashMap::from([
        ("default", fmt_default),
        ("header",  fmt_header),
        ("center",  fmt_center),
        ("value",   fmt_value),
        ("aliq",    fmt_aliq),
        ("date",    fmt_date),
    ])
}

/// Format columns by names using regex
/// 
/// and
/// 
/// Format lines by T Struct analysis.
fn format_lines_and_columns<'de, T>(
    worksheet: &mut Worksheet,
    _lines: &[T],
    column_names: &[&str],
    fmt: &HashMap<&str, Format>,
) -> MyResult<()>
where
    T: Serialize + Deserialize<'de> + InfoExtension + Iterable // + Sync + Send
{
    // Format columns by names using regex.
    'col: for (index, &col_name) in column_names.iter().enumerate() {
        let column_number: u16 = index.try_into()?;

        for (regex, fmt_name) in [
            (&REGEX_CENTER, "center"),
            (&REGEX_VALUE, "value"),
            (&REGEX_ALIQ, "aliq"),
            (&REGEX_DATE, "date"),
        ] {
            if let (true, Some(format)) = (regex.is_match(col_name), fmt.get(fmt_name)) {
                worksheet.set_column_format(column_number, format)?;
                continue 'col;
            };
        }
        
        if let Some(format) = fmt.get("default") {
            worksheet.set_column_format(column_number, format)?;
        }
    }

    /*    
    //let color_header: u32 = u32::from_str_radix("c5d9f1", 16)?;
    let color_bcsoma: u32 = u32::from_str_radix("bfbfbf", 16)?;
    //let color_descon: u32 = u32::from_str_radix("fff2cc", 16)?;
    let color_saldoc: u32 = u32::from_str_radix("f8cbad", 16)?; // rosa ('f8cbad') ou verde ('c4d79b')

    let format_bcsoma: Format = fmt
        .get("default")
        .unwrap()
        .clone()
        .set_background_color(Color::RGB(color_bcsoma));

    let format_saldoc: Format = fmt
        .get("default")
        .unwrap()
        .clone()
        .set_background_color(Color::RGB(color_saldoc));

    // Format lines by T Struct analysis.
    lines
        .iter()
        //.into_par_iter() // rayon parallel iterator
        .try_for_each(|line| -> MyResult<()> {
            line
                .iter()
                .enumerate()
                .try_for_each( |(index, (field_name, field_value))| -> MyResult<()> {
                    let column_number: u16 = index.try_into()?;
                    
                    match (field_name, field_value.downcast_ref::<Option<u16>>()) {
                        // BG Color: Base de Cálculo dos Créditos - Alíquota Básica (Soma)
                        ("natureza_bc", Some(Some(101..=199|300))) => {
                            worksheet.set_column_format(column_number, &format_bcsoma)?;
                        },
                        // BG Color: Crédito Disponível após Descontos
                        ("natureza_bc", Some(Some(221|225))) => {
                            worksheet.set_column_format(column_number, &format_saldoc)?;
                        },
                        // BG Color: Saldo de Crédito Passível de Desconto ou Ressarcimento
                        ("natureza_bc", Some(Some(301|305))) => {
                            worksheet.set_column_format(column_number, &format_saldoc)?;
                        },
                        _ => ()
                    };

                    Ok(())
                })
        })?;
    */

    Ok(())
}

/// Iterate over all data and find the max data width for each column.
fn auto_fit<'de, T>(
    worksheet: &mut Worksheet,
    lines: &[T],
    column_names: &[&str],
) -> MyResult<()>
where
    T: Serialize + Deserialize<'de> + InfoExtension + Iterable // + Sync + Send
{
    // HashMap<col index, col width>:
    let mut max_length: HashMap<usize, usize> = HashMap::new();

    column_names
        .iter()
        .enumerate()
        .for_each(|(col_index, col_name)| {
            // Init values: add column name lengths
            let col_len = col_name.chars_count().div_ceil(5);
            let col_width = WIDTH_MIN.max(col_len);
            //println!("col_len: {col_len} ; col_width: {col_width} ; col_name: {col_name}");
            max_length.insert(col_index, col_width);
        });

    lines
        .iter()
        //.into_par_iter() // rayon parallel iterator
        .for_each(|line| {
            get_length_of_column_values(line, &mut max_length)
        });

    for (index, len) in max_length {
        let width = WIDTH_MAX.min(len);
        //println!("{index:>2} {}: {width}", column_names[index]);
        worksheet.set_column_width(index as u16, (width as f64) * ADJUSTMENT)?;
    }

    Ok(())
}

/// Struct Iterable is a Rust library that provides a proc macro to make a struct iterable.
///
/// use struct_iterable::Iterable
///
/// <https://crates.io/crates/struct_iterable>
fn get_length_of_column_values<'de, T>(line: &T, max_length: &mut HashMap<usize, usize>)
where
    T: Serialize + Deserialize<'de> + InfoExtension + Iterable
{
    // let type_name = std::any::type_name::<T>();
    line
        .iter()
        .enumerate()
        .for_each( |(index, (field_name, field_value))| {
            // Get the length of field_value: &dyn Any.
            // <https://doc.rust-lang.org/beta/core/any/index.html>

            let field_value_len: usize = match_cast!( field_value {
                val as Option<u8> => {
                    val.as_ref().map(|s| s.to_string().chars_count())
                },
                val as Option<u16> => {
                    val.as_ref().map(|s| {
                        // Casos especiais para as colunas: natureza_bc e cst.
                        // println!("type_name: {type_name} ; field_name: {field_name} ; value: {s}");
                        match field_name {
                            "cst" => {
                                let string = display_cst(val);
                                string.chars_count() * 82 / 100  // Ajustes
                            },
                            "indicador_de_origem" => {
                                let string = indicador_de_origem_to_str(val);
                                string.chars_count() * 85 / 100  // Ajustes
                            },
                            "natureza_bc" => {
                                let string = display_natureza(val);
                                string.chars_count() * 72 / 100  // Ajustes
                            },
                            "tipo_de_credito" => {
                                let string = obter_descricao_do_tipo_de_credito(val, false);
                                string.chars_count() * 84 / 100  // Ajustes
                            },
                            "tipo_de_operacao" => {
                                let string = display_tipo_de_operacao(val);
                                string.chars_count()
                            },
                            _ => {
                                s.to_string().chars_count()
                            },
                        }
                    })
                },
                val as Option<u32> => {
                    val.as_ref().map(|s| s.to_string().chars_count())
                },
                val as Option<usize> => {
                    val.as_ref().map(|s| s.to_string().chars_count())
                },
                val as Option<f64> => {
                    val.as_ref().map(|f| f.to_string().chars_count())
                },
                val as Option<NaiveDate> => {
                    val.as_ref().map(|date| date.to_string().chars_count())
                },
                val as Option<String> => {
                    val.as_deref().map(|s| s.chars_count())
                },
                val as String => {
                    Some(val.chars_count())
                },
                val as Vec<String> => {
                    //Some(val.iter().map(|s| s.chars_count()).sum())
                    // use itertools;
                    Some(val.iter().join(", ").chars_count())
                },
            }).unwrap_or(format!("{field_value:?}").chars_count());

            let length: usize = *max_length.get(&index).unwrap_or(&0);

            if field_value_len > length {
                max_length.insert(index, field_value_len);
            }
        });
}
