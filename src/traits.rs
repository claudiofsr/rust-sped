use std::{
    any::Any,
    collections::{HashMap, HashSet},
    fmt::Debug,
    path::Path,
    str::FromStr,
    sync::Arc,
};

use chrono::NaiveDate;
use claudiofsr_lib::{FormatStyle, IteratorBack, StrExtension, thousands_separator};
use compact_str::CompactString;
use rust_decimal::Decimal;

use crate::{
    AnaliseDosCreditos, CodigoSituacaoTributaria, ConsolidacaoCST, DELIMITER_CHAR, EFDError,
    EFDResult, MesesDoAno, PRECISAO_FLOAT, SMALL_VALUE,
    structures::{analise_dos_creditos::Chaves, consolidacao_cst::Keys},
};

// --- Start: Definir traits para Ano, Mes, CST e CNPJBase ---
pub trait Ano {
    fn get_ano(&self) -> Option<i32>;
}

pub trait Mes {
    fn get_mes(&self) -> Option<MesesDoAno>;
    fn set_mes(&mut self, m: Option<MesesDoAno>);
    /// Alterar mês para MesesDoAno::Soma (índice 13)
    fn set_mes_para_soma(&mut self);
    fn is_soma(&self) -> bool;
}

pub trait Cst {
    fn get_cst(&self) -> Option<CodigoSituacaoTributaria>;
}

pub trait CNPJBase {
    fn get_cnpj_base(&self) -> CompactString;
}
// --- Final: Definir traits para Ano, Mes, CST e CNPJBase ---

// --- Start: Impl traits to Chaves ---
impl Ano for Chaves {
    fn get_ano(&self) -> Option<i32> {
        self.ano
    }
}

impl Mes for Chaves {
    fn get_mes(&self) -> Option<MesesDoAno> {
        self.mes
    }
    fn set_mes(&mut self, m: Option<MesesDoAno>) {
        self.mes = m;
    }
    fn set_mes_para_soma(&mut self) {
        // Define o mês como o enum "Soma" (Item 13)
        self.mes = Some(MesesDoAno::Soma);
    }
    fn is_soma(&self) -> bool {
        matches!(self.mes, Some(MesesDoAno::Soma))
    }
}

impl Cst for Chaves {
    fn get_cst(&self) -> Option<CodigoSituacaoTributaria> {
        self.cst
    }
}

impl CNPJBase for Chaves {
    fn get_cnpj_base(&self) -> CompactString {
        self.cnpj_base.clone()
    }
}
// --- Final: Impl traits to Chaves ---

// --- Start: Impl traits to keys ---
impl Ano for Keys {
    fn get_ano(&self) -> Option<i32> {
        self.ano
    }
}

impl Mes for Keys {
    fn get_mes(&self) -> Option<MesesDoAno> {
        self.mes
    }
    fn set_mes(&mut self, m: Option<MesesDoAno>) {
        self.mes = m;
    }
    fn set_mes_para_soma(&mut self) {
        // Define o mês como o enum "Soma" (Item 13)
        self.mes = Some(MesesDoAno::Soma);
    }
    fn is_soma(&self) -> bool {
        matches!(self.mes, Some(MesesDoAno::Soma))
    }
}

impl Cst for Keys {
    fn get_cst(&self) -> Option<CodigoSituacaoTributaria> {
        self.cst
    }
}

impl CNPJBase for Keys {
    fn get_cnpj_base(&self) -> CompactString {
        self.cnpj_base.clone()
    }
}
// --- Final: Impl traits to keys ---

pub fn verificar_periodo_multiplo<T, U>(resultado: &HashMap<T, U>) -> bool
where
    T: Ano + Mes + Cst + CNPJBase,
{
    // Mapa: CNPJ -> Conjunto de Períodos (AnoMes)
    let mut map_cnpj_periodos: HashMap<CompactString, HashSet<u32>> = HashMap::new();

    for chave in resultado.keys() {
        // Ignora chaves sem CST definido
        if chave.get_cst().is_none() {
            continue;
        }

        // Valida Ano e Mes
        if let (Some(ano), Some(mes)) = (chave.get_ano(), chave.get_mes()) {
            let ano_mes = (ano as u32) * 100 + (mes as u32);

            // 1. Obtém (ou cria) o HashSet para este CNPJ
            let periodos = map_cnpj_periodos.entry(chave.get_cnpj_base()).or_default();

            // 2. Insere o novo período (simplificação solicitada)
            periodos.insert(ano_mes);

            // 3. Otimização "Fail Fast":
            // Se detectarmos mais de 1 período para este CNPJ, paramos imediatamente.
            // Não é necessário continuar processando o resto do arquivo.
            if periodos.len() > 1 {
                return true;
            }
        }
    }

    // Se percorreu tudo e nenhum CNPJ teve > 1 período
    false
}

// --- AllValues --- //

trait AllValues {
    fn get_all_values(&mut self) -> Vec<&mut Decimal>;
}

impl AllValues for ConsolidacaoCST {
    fn get_all_values(&mut self) -> Vec<&mut Decimal> {
        vec![
            &mut self.valor_item,
            &mut self.valor_bc,
            &mut self.valor_pis,
            &mut self.valor_cofins,
        ]
    }
}

impl AllValues for AnaliseDosCreditos {
    fn get_all_values(&mut self) -> Vec<&mut Decimal> {
        vec![
            &mut self.valor_bc,
            &mut self.valor_rbnc_trib,
            &mut self.valor_rbnc_ntrib,
            &mut self.valor_rbnc_exp,
            &mut self.valor_rb_cum,
        ]
    }
}

/// Despise small values
pub trait Despise {
    fn despise_small_values(&mut self);
}

impl<T: AllValues> Despise for T {
    fn despise_small_values(&mut self) {
        for value in self.get_all_values() {
            if value.abs() < SMALL_VALUE {
                *value = Decimal::ZERO
            }
        }
    }
}

/// Extensão para facilitar comparações seguras com números de ponto flutuante (`f64`).
///
/// Em computação, `0.1 + 0.2 != 0.3` devido à precisão binária.
/// Portanto, nunca devemos usar `==` ou `>` diretamente para valores monetários em `f64`.
pub trait FloatExt {
    fn eh_zero(self) -> bool;
    fn eh_igual(self, other: f64) -> bool;
    fn eh_maior_que_zero(self) -> bool;
}

impl FloatExt for f64 {
    /// Verifica se o valor é virtualmente zero.
    ///
    /// Retorna `true` se o valor absoluto for menor que a tolerância de erro.
    #[inline]
    fn eh_zero(self) -> bool {
        self.abs() < PRECISAO_FLOAT
    }

    /// Verifica a igualdade entre dois floats considerando a margem de erro.
    #[inline]
    fn eh_igual(self, other: f64) -> bool {
        (self - other).abs() < PRECISAO_FLOAT
    }

    /// Verifica se o valor é positivo e significativo.
    ///
    /// Retorna `true` apenas se o número for maior que a tolerância (ex: 0.00000001).
    /// Valores extremamente pequenos (ruído numérico) são tratados como zero/falso.
    #[inline]
    fn eh_maior_que_zero(self) -> bool {
        self > PRECISAO_FLOAT
    }
}

// ============================================================================
// Decimal Extension
// ============================================================================

/// Trait de extensão: Adiciona métodos utilitários diretamente aos
/// tipos `Decimal` e `Option<Decimal>`.
pub trait DecimalExt {
    /// Verifica se o valor é estritamente positivo (maior que zero).
    fn eh_maior_que_zero(&self) -> bool;

    /// Verifica se o valor é nulo ou igual a zero.
    fn eh_zero(&self) -> bool;

    /// Verifica a igualdade exata com outro valor `Decimal`.
    fn eh_igual(&self, other: Decimal) -> bool;

    /// Formata o valor como String no padrão brasileiro (PtBr), com separadores de milhar.
    ///
    /// No arredondamento de `Decimal`, usa a estatégia `RoundingStrategy::MidpointNearestEven`.
    fn to_formatted_string(&self, decimals: usize) -> String;
}

impl DecimalExt for Decimal {
    fn eh_maior_que_zero(&self) -> bool {
        *self > Decimal::ZERO
    }

    fn eh_zero(&self) -> bool {
        self.is_zero()
    }

    fn eh_igual(&self, other: Decimal) -> bool {
        *self == other
    }

    fn to_formatted_string(&self, decimals: usize) -> String {
        thousands_separator(self, decimals, FormatStyle::PtBr)
    }
}

// Implementação para Option<Decimal> para facilitar chamadas diretas
impl DecimalExt for Option<Decimal> {
    fn eh_maior_que_zero(&self) -> bool {
        match self {
            Some(d) => d.eh_maior_que_zero(),
            None => false,
        }
    }

    fn eh_zero(&self) -> bool {
        match self {
            Some(d) => d.is_zero(),
            None => false,
        }
    }

    fn eh_igual(&self, other: Decimal) -> bool {
        match self {
            Some(d) => d.eh_igual(other),
            None => false,
        }
    }

    fn to_formatted_string(&self, decimals: usize) -> String {
        match self {
            Some(d) => d.to_formatted_string(decimals),
            None => String::new(),
        }
    }
}

// ============================================================================
// SEÇÃO 1: EXTENSIONS E UTILITÁRIOS
// Conversões seguras e funcionais para tipos primitivos e Options
// ============================================================================

/// Extension para facilitar o parsing e conversão de strings dentro de `Option`.
pub trait StringParser {
    /// Parse `Option<T>` para `Option<U>`.
    ///
    /// Tenta realizar o parse para um tipo `U` (ex: u32, f64, Decimal).
    /// Retorna `None` se a entrada for `None`, vazia ou se o parse falhar.
    fn parse_opt<U: FromStr>(&self) -> Option<U>;

    /// Converte `Option<T>` para `Option<Arc<str>>`.
    ///
    /// Útil para dados que serão compartilhados entre threads e que possuem vida longa,
    /// mas que excedem o tamanho do buffer inline do CompactString.
    ///
    /// Retorna `None` se a entrada for `None` ou string vazia (Economia de RAM).
    fn to_arc(&self) -> Option<Arc<str>>;

    /// Converte `Option<T>` para `Option<CompactString>`.
    ///
    /// TÉCNICA: Se a string tiver até 24 bytes, ela é armazenada na Stack (Zero Heap Allocation).
    ///
    /// Retorna `None` se a entrada for `None` ou string vazia (Economia de RAM).
    fn to_compact_string(&self) -> Option<CompactString>;

    /// Converte `Option<&str>` para Uppercase dentro de um `Option<Arc<str>>`.
    ///
    /// Só aloca nova string se houver alguma letra minúscula.
    /// - "NOTA 123" -> Retorna Arc(original) (Zero Copy)
    /// - "Nota 123" -> Retorna Arc("NOTA 123") (Alocação necessária)
    ///
    /// TÉCNICA: Caminho rápido para strings que já estão em maiúsculo.
    fn to_upper_arc(&self) -> Option<Arc<str>>;

    fn to_upper_arc_old(&self) -> Option<Arc<str>>;

    /// Converte `Option<&str>` para Uppercase dentro de um `Option<CompactString>`.
    ///
    /// Só aloca nova string se houver alguma letra minúscula.
    /// - "NOTA 123" -> Retorna CompactString(original) (Zero Copy)
    /// - "Nota 123" -> Retorna CompactString("NOTA 123") (Alocação necessária)
    ///
    /// TÉCNICA: Caminho rápido para strings que já estão em maiúsculo.
    fn to_upper_compact(&self) -> Option<CompactString>;

    fn to_upper_compact_old(&self) -> Option<CompactString>;
}

impl<T> StringParser for Option<T>
where
    T: AsRef<str>,
{
    #[inline]
    fn parse_opt<U: FromStr>(&self) -> Option<U> {
        self.as_ref()
            .map(|t| t.as_ref()) // 1. Obtém o &str (Zero Copy)
            .filter(|s| !s.is_empty()) // 2. "Fail fast": Se vazio, nem tenta parsear
            .and_then(|s| s.parse().ok())
    }

    fn to_arc(&self) -> Option<Arc<str>> {
        self.as_ref()
            .map(|t| t.as_ref())
            .filter(|s| !s.is_empty())
            .map(|s| {
                // Caminho Rápido usando std::str::contains (SIMD Accelerated)
                if !s.contains("  ") {
                    return Arc::from(s);
                }

                // Caminho Lento: Só entra aqui se realmente houver erro
                let mut res = String::with_capacity(s.len());
                let mut last_was_space = false;
                for c in s.chars() {
                    if c == ' ' {
                        if !last_was_space {
                            res.push(' ');
                            last_was_space = true;
                        }
                        // Se last_was_space for true, simplesmente ignoramos (remove espaço duplo)
                    } else {
                        res.push(c);
                        last_was_space = false;
                    }
                }
                Arc::from(res)
            })
    }

    fn to_compact_string(&self) -> Option<CompactString> {
        self.as_ref()
            .map(|t| t.as_ref())
            .filter(|s| !s.is_empty())
            .map(|s| {
                // Caminho Rápido: SIMD (Otimizado pela std)
                if !s.contains("  ") {
                    return CompactString::from(s);
                }

                // CAMINHO LENTO: Otimizado para evitar decodificação UTF-8
                let mut res = CompactString::with_capacity(s.len());
                let mut last_was_space = false;

                // TÉCNICA: Iterar sobre bytes é mais rápido que chars()
                // quando buscamos apenas caracteres ASCII (como o espaço).
                for &b in s.as_bytes() {
                    if b == b' ' {
                        if !last_was_space {
                            res.push(' ');
                            last_was_space = true;
                        }
                    } else {
                        // push byte diretamente (CompactString aceita char,
                        // converter b:u8 para char é apenas uma expansão de bits)
                        res.push(b as char);
                        last_was_space = false;
                    }
                }
                res
            })
    }

    /*
    Por que esta implementação é superior?

    1. Redução de Alocações Intermediárias: No código original, chamadas como s.replace_multiple_whitespaces().to_uppercase()
    criariam duas String completas na Heap. Agora, criamos apenas uma (o buffer res).

    2. Aproveitamento do Arc: Para o caso de strings que já estão corretas, Arc::from(s) é a operação mais barata possível,
    pois ela apenas aloca o espaço exato na Heap e copia os bytes do &str.

    3. Fusão de Operações: Limpeza de espaços duplos e conversão para maiúsculas ocorrem no mesmo loop.
    Isso reduz o número de vezes que a CPU precisa ler os bytes da memória (melhor uso de cache L1).

    4. Segurança e Consistência: O comportamento é idêntico ao to_upper_compact, garantindo que seu sistema trate os dados
    de forma previsível, independentemente do tipo de armazenamento escolhido.

    Resumo dos Ganhos

    - Strings Limpas: Passou de 1 alocação temporária + 1 alocação final para apenas 1 alocação final.
    - Strings Sujas: Passou de 2 alocações temporárias + 1 alocação final para 1 alocação temporária (buffer) que se torna a final.
    - Performance: Menos pressão no alocador de memória do sistema operacional e menos ciclos de CPU desperdiçados em loops repetitivos.

    */

    fn to_upper_arc(&self) -> Option<Arc<str>> {
        self.as_ref()
            .map(|t| t.as_ref())
            .filter(|s| !s.is_empty())
            .map(|s| {
                let mut needs_fix = false;
                let mut prev_is_space = false;

                // TÉCNICA: Inspeção O(n) para detectar se o "Caminho Rápido" é possível

                // --- PASSO 1: Inspeção Fail-Fast O(n) ---
                for c in s.chars() {
                    if c.is_lowercase() || (c == ' ' && prev_is_space) {
                        needs_fix = true;
                        break; // ENCONTROU ERRO, PARA TUDO!
                    }
                    prev_is_space = c == ' ';
                }

                // CAMINHO RÁPIDO: String já está limpa e em maiúsculas
                if !needs_fix {
                    // Aloca o Arc diretamente do slice original.
                    // Uma única alocação na Heap, zero strings temporárias.
                    return Arc::from(s);
                }

                // --- PASSO 2: Caminho Lento (Transformação Única) ---
                // Se chegou aqui, não sabemos qual erro foi, então corrigimos AMBOS.
                // Criamos uma String como buffer temporário.
                // TÉCNICA: Usamos with_capacity para evitar realocações durante o loop.
                let mut res = String::with_capacity(s.len());
                let mut last_was_space = false;

                for c in s.chars() {
                    if c == ' ' {
                        if !last_was_space {
                            res.push(' ');
                            last_was_space = true;
                        }
                        // Se last_was_space for true, simplesmente ignoramos (remove espaço duplo)
                    } else {
                        last_was_space = false;
                        // TÉCNICA: Otimização ASCII dentro da transformação
                        if c.is_ascii() {
                            res.push(c.to_ascii_uppercase());
                        } else {
                            res.extend(c.to_uppercase());
                        }
                    }
                }

                // Converte a String final em Arc.
                // Rust é inteligente o suficiente para tentar "reaproveitar" o buffer
                // da String para o Arc se possível, minimizando cópias.
                Arc::from(res)
            })
    }

    fn to_upper_arc_old(&self) -> Option<Arc<str>> {
        self.as_ref()
            .map(|t| t.as_ref()) // 1. Obtém o &str de T
            .filter(|s| !s.is_empty())
            .map(|s| {
                // Verificamos se há necessidade de transformação
                let has_lower = s.chars().any(|c| c.is_lowercase());
                let has_double_space = s.contains("  ");

                if has_lower || has_double_space {
                    // CAMINHO LENTO: Alocação necessária para limpeza e conversão
                    Arc::from(s.replace_multiple_whitespaces().to_uppercase())
                } else {
                    // CAMINHO RÁPIDO: Já está limpo e em Uppercase. Zero alocação de String temporária.
                    Arc::from(s)
                }
            })
    }

    /*
    Por que isso é mais eficiente?

    1. Eliminação de s.to_string() e s.to_uppercase(): No seu código anterior, s.to_string() criava uma
     na Heap mesmo se a string fosse pequena. Agora, vamos de &str direto para CompactString.

    2. Lógica Unificada: Em vez de chamar replace_multiple_whitespaces() (que criaria uma String) e
     to_uppercase() (que criaria outra String), fazemos ambas as operações em uma única passagem de caracteres.

    3. Memória Inline: Se o resultado final (após remover espaços e converter para maiúsculas) tiver até 24 bytes,
    nenhuma memória será alocada na Heap em todo o processo de transformação.

    4. Early Exit: O loop de inspeção inicial termina assim que descobre que a string precisa de "trabalho",
    economizando ciclos de CPU em strings longas que já sabemos estarem sujas.

    4 casos possiveis em has_lower com has_double_space:

    Análise dos 4 Casos:
    1. (L=false, S=false): CompactString::from(s). O caminho mais rápido.
    2. (L=true, S=false): O loop de transformação apenas faz o to_uppercase e ignora a lógica de espaços.
    3. (L=false, S=true): O loop apenas limpa os espaços e ignora a lógica de to_uppercase.
    4. (L=true, S=true): O loop realiza ambas as operações simultaneamente.

    Essa abordagem trata a CompactString como um buffer de destino, garantindo que os dados
    sejam movidos e transformados com o menor número possível de operações de escrita em memória.
    */
    fn to_upper_compact(&self) -> Option<CompactString> {
        self.as_ref()
            .map(|t| t.as_ref())
            .filter(|s| !s.is_empty())
            .map(|s| {
                let mut needs_fix = false;
                let mut prev_is_space = false;

                // --- PASSO 1: Inspeção Fail-Fast O(n) ---
                for c in s.chars() {
                    if c.is_lowercase() || (c == ' ' && prev_is_space) {
                        needs_fix = true;
                        break; // ENCONTROU ERRO, PARA TUDO!
                    }
                    prev_is_space = c == ' ';
                }

                // CAMINHO RÁPIDO: String perfeita.
                if !needs_fix {
                    return CompactString::from(s);
                }

                // --- PASSO 2: Caminho Lento (Transformação Única) ---
                // Se chegou aqui, não sabemos qual erro foi, então corrigimos AMBOS.
                let mut res = CompactString::with_capacity(s.len());
                let mut last_was_space = false;

                for c in s.chars() {
                    if c == ' ' {
                        if !last_was_space {
                            res.push(' ');
                            last_was_space = true;
                        }
                        // Se last_was_space for true, simplesmente ignoramos (remove espaço duplo)
                    } else {
                        last_was_space = false;
                        // TÉCNICA: Otimização ASCII dentro da transformação
                        if c.is_ascii() {
                            res.push(c.to_ascii_uppercase());
                        } else {
                            res.extend(c.to_uppercase());
                        }
                    }
                }
                res
            })
    }

    fn to_upper_compact_old(&self) -> Option<CompactString> {
        self.as_ref()
            .map(|t| t.as_ref()) // 1. Obtém o &str de T
            .filter(|s| !s.is_empty())
            .map(|s| {
                // Verificamos se há necessidade de transformação
                let has_lower = s.chars().any(|c| c.is_lowercase());
                let has_double_space = s.contains("  ");

                if has_lower || has_double_space {
                    // Se precisar de limpeza, a CompactString cuidará da alocação na Heap apenas se necessário
                    let cleaned = if has_double_space {
                        s.replace_multiple_whitespaces()
                    } else {
                        s.to_string()
                    };
                    CompactString::from(cleaned.to_uppercase())
                } else {
                    // Já está perfeito: Se < 24 bytes, Zero alocação na Heap.
                    CompactString::from(s)
                }
            })
    }
}

/// A trait for splitting a string into individual fields using a delimiter.
pub trait SplitLine {
    fn split_line(&self) -> Vec<String>;
}

// Alterado para aceitar referências via AsRef, evitando clonar se não necessário
// Porém, como o retorno é Vec<String>, a alocação é inevitável no final.
impl<T> SplitLine for T
where
    T: AsRef<str>,
{
    fn split_line(&self) -> Vec<String> {
        self.as_ref()
            .split(DELIMITER_CHAR)
            .skip_last() // Skip the last element (empty string)
            .skip(1) // Skip the first element (empty string)
            .map(|campo| campo.trim().to_string())
            .collect()
    }
}

pub trait SpedRecordTrait: Debug + Any + Send + Sync {
    fn nivel(&self) -> u16;
    fn bloco(&self) -> char;
    fn registro_name(&self) -> &str;
    fn line_number(&self) -> usize;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// A trait to convert an `Option<T>` into an `EFDResult<Option<Decimal>>`.
pub trait ToDecimal {
    fn to_decimal(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<Decimal>>;
}

// Otimizado: T: AsRef<str> evita .to_string() imediato
impl<T: AsRef<str>> ToDecimal for Option<T> {
    fn to_decimal(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<Decimal>> {
        self.as_ref() // Option<T> -> Option<&T>
            .map(|s| s.as_ref().trim()) // Option<&T> -> Option<&str>
            .filter(|s| !s.is_empty())
            .map(|s| {
                // Decimal requer troca de vírgula por ponto (custo de alocação)
                // Se RustDecimal suportasse locale pt-BR nativamente, evitaríamos essa string.
                let s_parsed = s.replace('.', "").replace(',', ".");

                Decimal::from_str_exact(&s_parsed).map_err(|source| EFDError::ParseDecimalError {
                    source,
                    valor_str: s.to_string(), // Aloca apenas no erro para contexto
                    campo_nome: field_name.to_string(),
                    arquivo: file_path.to_path_buf(),
                    linha_num: line_number,
                })
            })
            .transpose()
    }
}

/// A trait to convert an `Option<T>` to `EFDResult<Option<NaiveDate>>`.
pub trait ToNaiveDate {
    fn to_optional_date(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<NaiveDate>>;

    fn to_date(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<NaiveDate>;
}

// Otimizado: Zero allocation no caminho feliz para datas completas (8 chars)
impl<T: AsRef<str>> ToNaiveDate for Option<T> {
    fn to_optional_date(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<NaiveDate>> {
        self.as_ref()
            .map(|s| s.as_ref().trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                let date_format = "%-d%-m%Y";
                let result = if s.len() == 8 {
                    // Formato DDMMYYYY: Parse direto do slice (Zero Copy)
                    NaiveDate::parse_from_str(s, date_format)
                } else {
                    // Formato MMYYYY: Requer alocação para prefixar '01'
                    let day_month_year = format!("01{}", s);
                    NaiveDate::parse_from_str(&day_month_year, date_format)
                };

                result.map_err(|source| EFDError::ParseDateError {
                    source,
                    data_str: s.to_string(),
                    campo_nome: field_name.to_string(),
                    arquivo: file_path.to_path_buf(),
                    line_number,
                })
            })
            .transpose()
    }

    fn to_date(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<NaiveDate> {
        self.to_optional_date(file_path, line_number, field_name)?
            .ok_or_else(|| EFDError::KeyNotFound(field_name.to_string()))
    }
}

/// Trait para converter `Option<T>` em `Option<String>` (trimada e não vazia).
pub trait ToOptionalString {
    fn to_optional_string(&self) -> Option<String>;
}

impl<T: AsRef<str>> ToOptionalString for Option<T> {
    fn to_optional_string(&self) -> Option<String> {
        self.as_ref()
            .map(|s| s.as_ref().trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()) // Aloca apenas se não for vazio
    }
}

/// Extension trait for parsing optional string-like values into integers.
pub trait ToOptionalInteger {
    /// Parses the inner value into generic type `U`.
    fn to_optional_integer<U>(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> Result<Option<U>, EFDError>
    where
        U: FromStr + Debug,
        <U as FromStr>::Err: Into<std::num::ParseIntError>;
}

// Blanket implementation for any Option containing a string-like type.
// High performance: Works on &str, allocates only on error.
impl<T> ToOptionalInteger for Option<T>
where
    T: AsRef<str>,
{
    fn to_optional_integer<U>(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> Result<Option<U>, EFDError>
    where
        U: FromStr + Debug,
        <U as FromStr>::Err: Into<std::num::ParseIntError>,
    {
        self.as_ref()
            .map(|s| s.as_ref().trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                s.parse::<U>().map_err(|e| EFDError::ParseIntegerError {
                    source: e.into(),
                    data_str: s.to_string(), // Allocates only on error
                    campo_nome: field_name.to_string(),
                    arquivo: file_path.to_path_buf(),
                    line_number,
                })
            })
            .transpose()
    }
}

/// Trait para validação e conversão de CNPJ.
pub trait ToCNPJ {
    fn to_optional_cnpj(
        &self,
        file_path: &Path,
        line_number: usize,
        registro: &str,
        field_name: &str,
    ) -> EFDResult<Option<String>>;

    fn to_cnpj(
        &self,
        file_path: &Path,
        line_number: usize,
        registro: &str,
        field_name: &str,
    ) -> EFDResult<String>;
}

impl<T: AsRef<str>> ToCNPJ for Option<T> {
    fn to_optional_cnpj(
        &self,
        file_path: &Path,
        line_number: usize,
        registro: &str,
        field_name: &str,
    ) -> EFDResult<Option<String>> {
        self.as_ref()
            .map(|s| s.as_ref().trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                // Validação feita no slice, alocação apenas no sucesso ou erro.
                if s.len() == 14 {
                    Ok(s.to_string())
                } else {
                    Err(EFDError::InvalidCNPJ {
                        arquivo: file_path.to_path_buf(),
                        linha_num: line_number,
                        registro: registro.to_string(),
                        campo_nome: field_name.to_string(),
                        cnpj: s.to_string(),
                    })
                }
            })
            .transpose()
    }

    fn to_cnpj(
        &self,
        file_path: &Path,
        line_number: usize,
        registro: &str,
        field_name: &str,
    ) -> EFDResult<String> {
        self.to_optional_cnpj(file_path, line_number, registro, field_name)?
            .ok_or_else(|| EFDError::KeyNotFound(field_name.to_string()))
    }
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//
//
// cargo test -- --help
// cargo test -- --nocapture
// cargo test -- --show-output

/// Run tests with:
/// cargo test -- --show-output traits_tests
#[cfg(test)]
#[path = "tests/traits_tests.rs"]
mod traits_tests;
