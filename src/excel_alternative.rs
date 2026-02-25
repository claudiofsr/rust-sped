use chrono::NaiveDate;
use claudiofsr_lib::{IntegerDigits, OptionExtension, get_style};
use indicatif::{MultiProgress, ProgressBar};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use rust_xlsxwriter::{Format, Worksheet};

use crate::{
    AnaliseDosCreditos, CSTOption, ConsolidacaoCST, DECIMAL_VALOR, DocsFiscais, EFDError,
    EFDResult, MesesDoAno, ResultExt, display_aliquota, display_cst, excel_comum::*,
    obter_descricao_do_cfop,
};

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

pub fn get_all_worksheets(
    data_efd: &[DocsFiscais],
    data_cst: &[ConsolidacaoCST],
    data_nat: &[AnaliseDosCreditos],
    formats: &HashMap<String, Format>,
    multiprogressbar: &MultiProgress,
) -> EFDResult<Vec<Worksheet>> {
    // Inicializamos os resultados como Sucesso vazio.
    // O Rayon scope garantirá que as atribuições ocorram antes de lermos os valores.
    let mut res_efd: EFDResult<Vec<Worksheet>> = Ok(vec![]);
    let mut res_cst: EFDResult<Vec<Worksheet>> = Ok(vec![]);
    let mut res_nat: EFDResult<Vec<Worksheet>> = Ok(vec![]);

    // Rayon Scope: Aproveita o pool de threads global (mais eficiente que thread::spawn do SO)
    // Permite que as threads acessem dados na stack (slices e referências) sem Arc.
    rayon::scope(|s| {
        // Cada spawn executa uma tarefa heterogênea em paralelo.
        // Capturamos a referência mutável de cada resultado específico.
        s.spawn(|_| {
            res_efd = generate_worksheets(
                data_efd,
                SheetType::ItensDocsFiscais,
                formats,
                multiprogressbar,
                0,
                add_row_efd,
            );
        });

        s.spawn(|_| {
            res_cst = generate_worksheets(
                data_cst,
                SheetType::ConsolidacaoCST,
                formats,
                multiprogressbar,
                1,
                add_row_cst,
            );
        });

        s.spawn(|_| {
            res_nat = generate_worksheets(
                data_nat,
                SheetType::AnaliseCreditos,
                formats,
                multiprogressbar,
                2,
                add_row_nat,
            );
        });
    });

    // Processamento funcional dos resultados:
    // 1. Coletamos os resultados em um array.
    // 2. O collect para Result<Vec<Vec<T>>> implementa o "short-circuit":
    //    Se qualquer um for Erro, o resultado final será o primeiro Erro encontrado.
    // 3. Flatten achata a estrutura de Vec<Vec<Worksheet>> para Vec<Worksheet>.
    [res_efd, res_cst, res_nat]
        .into_iter()
        .collect::<EFDResult<Vec<Vec<Worksheet>>>>()
        .map(|v| v.into_iter().flatten().collect())
}

#[allow(clippy::type_complexity)]
/// Função Genérica que encapsula a lógica de chunking (divisão de abas),
/// barra de progresso e headers.
fn generate_worksheets<'de, T>(
    data: &[T],
    sheet_type: SheetType,
    formats: &HashMap<String, Format>,
    mpb: &MultiProgress,
    pb_idx: usize,
    row_fn: fn(
        u32,
        &T,
        &mut Worksheet,
        &HashMap<String, Format>,
        &mut BTreeMap<u16, usize>,
    ) -> EFDResult<()>,
) -> EFDResult<Vec<Worksheet>>
where
    T: InfoExtension + ExcelCustomFormatter + Serialize + Deserialize<'de>,
{
    let mut worksheets = Vec::new();

    if data.is_empty() {
        return Ok(worksheets);
    }

    let pb = mpb.insert(pb_idx, ProgressBar::new(data.len() as u64));
    pb.set_style(get_style(0, 0, 35)?);
    pb.set_message(format!("Excel: {}", sheet_type));

    for (k, chunk) in data.chunks(MAX_NUMBER_OF_ROWS).enumerate() {
        let sheet_name = if k == 0 {
            sheet_type.to_string()
        } else {
            format!("{} {}", sheet_type, k + 1)
        };
        let mut ws = Worksheet::new();
        ws.set_name(&sheet_name)?;

        let mut width_map = BTreeMap::new();
        let headers = T::get_headers();

        create_headers(headers, &mut ws, &mut width_map, formats, sheet_type)?;

        for (j, item) in chunk.iter().enumerate() {
            row_fn(j as u32, item, &mut ws, formats, &mut width_map)?;
            pb.inc(1);
        }

        // Ajuste fino do fator de largura baseado no tipo de aba
        let fator = if sheet_type.is_itens() { 1.05 } else { 1.0 };
        set_max_width(&mut ws, &width_map, fator)?;
        worksheets.push(ws);
    }
    pb.finish();
    Ok(worksheets)
}

fn create_headers(
    headers: &[&str],
    sheet: &mut Worksheet,
    width_map: &mut BTreeMap<u16, usize>,
    fmt: &HashMap<String, Format>,
    sheet_type: SheetType,
) -> EFDResult<()> {
    let fmt_header = fmt
        .get("header")
        .map_loc(|_| EFDError::FormatNotFound("header".into()))?;
    let mut last_col = 0;

    for (idx, &header) in headers.iter().enumerate() {
        let col = idx as u16;
        sheet.write_string_with_format(0, col, header, fmt_header)?;

        let mut width = header.len();
        match sheet_type {
            SheetType::ItensDocsFiscais => {
                // definir largura mínima de colunas específicas
                let valor_ou_aliquota = header.contains("Valor") || header.contains("Alíquota");

                if idx == 2 || header.contains("Período de Apuração") || valor_ou_aliquota {
                    width = 12;
                } else if header.contains("CNPJ") || header.contains("Data") {
                    width = 18;
                }
            }
            SheetType::ConsolidacaoCST => width = 12,
            SheetType::AnaliseCreditos => {
                // definir largura mínima
                width = 12;
                // definir largura mínima de colunas específicas
                if idx == 5 {
                    width = 6;
                } else if idx >= 9 {
                    width = 18;
                }
            }
        }

        width_map.insert(col, width);
        last_col = col;
    }

    sheet.autofilter(0, 0, 0, last_col)?;
    sheet.set_freeze_panes(1, 0)?;
    Ok(())
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
        .decimal(col.valor_item, f("value")?)?
        .decimal(col.valor_bc, f("value")?)?
        .decimal(col.aliq_pis, f("aliquota")?)?
        .decimal(col.aliq_cofins, f("aliquota")?)?
        .decimal(col.valor_pis, f("value")?)?
        .decimal(col.valor_cofins, f("value")?)?
        .decimal(col.valor_iss, f("value")?)?
        .decimal(col.valor_bc_icms, f("value")?)?
        .decimal(col.aliq_icms, f("aliquota")?)?
        .decimal(col.valor_icms, f("value")?)?;

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
        .decimal(Some(col.valor_item), f("value")?)?
        .decimal(Some(col.valor_bc), f("value")?)?
        .decimal(Some(col.valor_pis), f("value")?)?
        .decimal(Some(col.valor_cofins), f("value")?)?;

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
        .decimal(col.valor_bc, f("value")?)?
        .decimal(col.valor_rbnc_trib, f("value")?)?
        .decimal(col.valor_rbnc_ntrib, f("value")?)?
        .decimal(col.valor_rbnc_exp, f("value")?)?
        .decimal(col.valor_rb_cum, f("value")?)?;

    let height = if row == 0 {
        HEADER_FONT_SIZE + 50.0
    } else {
        FONT_SIZE + 3.0
    };
    sheet.set_row_height(row, height)?;
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
