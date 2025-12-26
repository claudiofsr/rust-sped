use crate::{DECIMAL_VALOR, DecimalExt, EFDResult};
use crate::{Mes, MesesDoAno};
use rayon::prelude::*;
use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde::Serializer;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::AddAssign;
use std::{fmt::Display, fs, io::Write};
use tempfile::NamedTempFile;

/// Create a named temporary file and write some data into it
pub fn create_a_temp_file(contents: &str, read_file: bool) -> EFDResult<NamedTempFile> {
    // Create a file inside of `env::temp_dir()`.
    let mut file = NamedTempFile::new()?;

    // Write some test data to the file handle.
    file.write_all(contents.as_bytes())?;

    if read_file {
        // Reading an entire file into a String:
        let string = fs::read_to_string(file.path())?; // The '?' operator propagates errors
        println!(
            "Conteúdo do arquivo temporário [{:?}]:\n{}",
            file.path(),
            string
        );
    }

    Ok(file)
}

// ==============================================================================
// Helpers de Arquivo e Serialização
// ==============================================================================

/// Helper function to serialize Decimal as f64 (Excel Number)
pub fn serialize_decimal<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if value.is_zero() {
        serializer.serialize_none()
    } else {
        let float_val = value.to_f64().unwrap_or_default();
        serializer.serialize_f64(float_val)
    }
}

/// Helper para exibir Decimal formatado em tabelas
///
/// Ver em traits.rs fn despise_small_values
pub fn display_decimal(valor: &Decimal) -> String {
    // Se quiser manter o comportamento de ocultar zeros:
    if valor.is_zero() {
        String::new()
    } else {
        // Ajuste a formatação conforme sua necessidade (ex: 2 casas decimais)
        valor.to_formatted_string(DECIMAL_VALOR)
    }
}

pub fn display_mes(mes: &Option<MesesDoAno>) -> String {
    match mes {
        Some(MesesDoAno::Soma) => String::new(),
        Some(m) => (*m as u8).to_string(),
        _ => String::new(),
    }
}

pub fn display_value<T: Display>(valor: &Option<T>) -> String {
    valor.as_ref().map(|v| v.to_string()).unwrap_or_default()
}

// ==============================================================================
// Agregações e Cálculos Genéricos
// ==============================================================================

/// Consolidação Genérica (Map-Reduce) usando Rayon.
///
/// Abstrai o padrão `filter -> map -> fold -> reduce` repetido nos arquivos.
pub fn consolidar_registros<T, K, V>(
    itens: &[T],
    filter_fn: impl Fn(&T) -> bool + Sync + Send,
    mapper_fn: impl Fn(&T) -> (K, V) + Sync + Send,
) -> HashMap<K, V>
where
    T: Sync,
    K: Hash + Eq + Send,
    V: Send + AddAssign + Default,
{
    itens
        .into_par_iter()
        //.filter(|&linha| linha.operacoes_de_entrada_ou_saida()) // 1: Entrada, 2: Saída
        //.filter(|&line| !(line.cst == Some(9) && line.registro == "C170") ) // desconsiderar: CST 9 && Registro C170
        .filter(|&x| filter_fn(x))
        .map(mapper_fn)
        .fold(HashMap::new, |mut acc, (k, v)| {
            *acc.entry(k).or_default() += v;
            acc
        })
        .reduce(HashMap::new, |mut acc, map| {
            for (k, v) in map {
                *acc.entry(k).or_default() += v;
            }
            acc
        })
}

/// Realizar somas trimestrais em paralelo de forma genérica.
///
/// Funciona tanto para ConsolidacaoCST quanto para AnaliseDosCreditos
pub fn realizar_somas_trimestrais<K, V>(mapa_original: &mut HashMap<K, V>)
where
    // Send/Sync necessários para Rayon
    K: Mes + Eq + Hash + Clone + Send + Sync, // K = Chaves
    V: Clone + AddAssign + Send + Sync,       // V = Valores
{
    // Passo 1: Calcular as somas (Funcional e Imutável até o fold)
    let somas_mensais = mapa_original
        .par_iter() // 1. Itera em paralelo (várias threads)
        // Filtra para não somar linhas que JÁ SÃO somas (caso rode a função 2x)
        .filter(|(chave, _)| !chave.is_soma())
        .map(|(chave, valor)| {
            let mut chave_soma = chave.clone();
            // Mês fictício 13 para fins de soma e ordenação.
            // Muda o mês de "Janeiro" para "Soma"
            chave_soma.set_mes_para_soma();
            (chave_soma, valor)
        })
        // 2. FOLD: Cada thread constrói um HashMap local acumulado
        .fold(HashMap::new, |mut acc, (chave, valor)| {
            // Se a chave já existe (mesmo trimestre/ano/cnpj), soma os valores.
            // Se não, insere o novo valor.
            acc.entry(chave)
                .and_modify(|v| *v += valor.clone())
                .or_insert_with(|| valor.clone());
            acc
        })
        // 3. REDUCE: Funde os HashMaps de todas as threads em um só
        .reduce(HashMap::new, |mut mapa_a, mapa_b| {
            for (k, v) in mapa_b {
                mapa_a
                    .entry(k)
                    .and_modify(|val_a| *val_a += v.clone())
                    .or_insert(v);
            }
            mapa_a
        });

    // Merge final no mapa original
    mapa_original.extend(somas_mensais);
}
