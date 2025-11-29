mod analyze_all;
mod analyze_one;
mod analyze_one_new;
mod args;
mod dispatch_table;
mod error;
mod excel_worksheets;
mod line_iterator;
mod regex;
mod sped_efd;
mod sped_registros;
mod structures;
mod traits;
mod utils;
mod validate_line;

mod blocos;
mod macros;
mod model;
mod parser;

pub use self::{
    analyze_all::*, analyze_one::*, analyze_one_new::*, args::*, blocos::*, dispatch_table::*,
    error::*, excel_worksheets::*, line_iterator::*, model::*, parser::*, regex::*, sped_efd::*,
    sped_registros::*, structures::*, traits::*, utils::*, validate_line::*,
};

// https://crates.io/crates/cfg-if
cfg_if::cfg_if! {
    if #[cfg(feature = "old")] {
        mod excel_alternative;
        pub use excel_alternative::*;
    } else {
        // default: "new"
        mod excel_workbook;
        pub use excel_workbook::*;
    }
}

// Definição da tolerância para comparações de ponto flutuante.
// 1e-8 (0.00000001) é suficiente para ignorar "ruído" numérico sem afetar centavos.
const PRECISAO_FLOAT: f64 = 1e-8;

pub const SMALL_VALUE: f64 = 0.009; // usado em fn despise_small_values; menor que um centavo
pub const DELIMITER_CHAR: char = '|';
pub const ALIQ_BASICA_PIS: f64 = 1.65;
pub const ALIQ_BASICA_COF: f64 = 7.60;
pub const DECIMAL_VALOR: usize = 2;
pub const DECIMAL_ALIQ: usize = 4;
pub const NEWLINE_BYTE: u8 = b'\n';
pub const OUTPUT_DIRECTORY: &str = "novo";
pub const OUTPUT_FILENAME: &str = "Info do Contribuinte EFD Contribuicoes";
