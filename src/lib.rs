mod analyze_all;
mod analyze_one;
mod args;
mod blocos;
mod config;
mod error;
mod excel_comum;
mod excel_worksheets;
mod extractor;
mod macros;
mod model;
mod parser;
mod pattern_adapter;
mod regex;
mod structures;
mod tabelas;
mod traits;
mod utils;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub use self::{
    analyze_all::*, analyze_one::*, args::*, blocos::*, config::*, error::*, excel_comum::*,
    excel_worksheets::*, extractor::*, model::*, parser::*, regex::*, structures::*, tabelas::*,
    traits::*, utils::*,
};

// Definição da tolerância para comparações de ponto flutuante.
// 1e-8 (0.00000001) é suficiente para ignorar "ruído" numérico sem afetar centavos.
const PRECISAO_FLOAT: f64 = 1e-8;

pub const SMALL_VALUE: Decimal = dec!(0.009); // usado em fn despise_small_values; menor que um centavo
pub const DELIMITER_CHAR: char = '|';
pub const ALIQ_BASICA_PIS: Decimal = dec!(1.65);
pub const ALIQ_BASICA_COF: Decimal = dec!(7.6);
pub const DECIMAL_VALOR: usize = 2;
pub const DECIMAL_ALIQ: usize = 4;
pub const NEWLINE_BYTE: u8 = b'\n';
pub const DATE_FORMAT: &str = "%d/%m/%Y %H:%M:%S";
pub const BUFFER_CAPACITY: usize = 8 * 1024 * 1024; // 8MB
