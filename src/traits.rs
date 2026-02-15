use std::{
    any::Any,
    collections::{HashMap, HashSet},
    fmt::Debug,
    panic::Location,
    path::Path,
    str::FromStr,
    sync::Arc,
};

use chrono::NaiveDate;
use claudiofsr_lib::{FormatStyle, thousands_separator};
use compact_str::CompactString;
use rust_decimal::Decimal;

use crate::{
    AnaliseDosCreditos, CodigoDoCredito, CodigoSituacaoTributaria, ConsolidacaoCST, EFDError,
    EFDResult, GrupoDeContas, MesesDoAno, PRECISAO_FLOAT, SMALL_VALUE,
    structures::{analise_dos_creditos::Chaves, consolidacao_cst::Keys},
};

// --- Start: Definir traits para Ano, Mes, CST e CNPJBase ---
pub trait Ano {
    fn get_ano(&self) -> Option<i32>;
}

pub trait Mes {
    fn get_mes(&self) -> Option<MesesDoAno>;
    fn set_mes(&mut self, m: Option<MesesDoAno>);

    /// Define o mês como None para representar o total trimestral
    fn set_mes_para_total(&mut self) {
        self.set_mes(None);
    }

    /// Verifica se é uma linha de total (granularidade trimestral)
    fn is_total_trimestral(&self) -> bool {
        self.get_mes().is_none()
    }
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

// ============================================================================
// AUXILIARES INTERNOS (DRY & PERFORMANCE)
// ============================================================================

/// Trait interna para unificar a construção de `String` e `CompactString`.
pub trait StringBuilder: Sized + Extend<char> {
    fn with_capacity(cap: usize) -> Self;
    fn push(&mut self, c: char);
    fn push_str(&mut self, s: &str);
    fn from_str(s: &str) -> Self;
}

impl StringBuilder for String {
    #[inline(always)]
    fn with_capacity(cap: usize) -> Self {
        String::with_capacity(cap)
    }
    #[inline(always)]
    fn push(&mut self, c: char) {
        self.push(c);
    }
    #[inline(always)]
    fn push_str(&mut self, s: &str) {
        self.push_str(s);
    }
    #[inline(always)]
    fn from_str(s: &str) -> Self {
        s.to_string()
    }
}

impl StringBuilder for CompactString {
    #[inline(always)]
    fn with_capacity(cap: usize) -> Self {
        CompactString::with_capacity(cap)
    }
    #[inline(always)]
    fn push(&mut self, c: char) {
        self.push(c);
    }
    #[inline(always)]
    fn push_str(&mut self, s: &str) {
        self.push_str(s);
    }
    #[inline(always)]
    fn from_str(s: &str) -> Self {
        CompactString::from(s)
    }
}

/// Engine 1: Colapsa múltiplos espaços sequenciais em um único espaço.
///
/// Utiliza a técnica SIMD para o Fast Path e Slice-Pushing para o Slow Path.
/// Foco em Zero Alocações extras (além do builder de destino) e eficiência de cache.
///
/// # Performance
/// - **Fast Path**: Se a string não contém "  ", retorna o tipo de destino imediatamente.
/// - **O(n)**: Uma única passagem de inspeção + uma única passagem de cópia por blocos.
#[inline]
fn collapse_multiple_spaces<B: StringBuilder>(s: &str) -> B {
    // 1. FAST PATH (SIMD):
    // O método `contains` da biblioteca padrão do Rust é implementado com instruções SIMD.
    // Ele consegue verificar 16 ou 32 bytes por ciclo de clock.
    // Em arquivos SPED, a maioria dos campos é curta ou já está limpa.
    if !s.contains("  ") {
        return B::from_str(s);
    }

    let bytes = s.as_bytes();
    let len = bytes.len();

    // 2. ALOCAÇÃO ÚNICA:
    // Alocamos a capacidade total de uma vez para evitar re-alocações (realloc) durante o loop.
    let mut builder = B::with_capacity(len);

    // 'start' rastreia o início do próximo bloco de texto limpo.
    let mut start = 0;
    let mut last_was_space = false;

    for i in 0..bytes.len() {
        let byte = bytes[i];
        if byte == b' ' {
            if last_was_space {
                // 3. IDENTIFICAÇÃO DE "LIXO" (Espaço Duplicado):
                // Encontramos o segundo espaço consecutivo.
                // Antes de pulá-lo, copiamos todo o bloco limpo acumulado até aqui.

                // O bloco limpo termina exatamente antes deste byte extra.
                if i > start {
                    // PERFORMANCE: push_str em um slice (&s[start..i]) é traduzido para
                    // um `memcpy` de hardware. É ordens de grandeza mais rápido que
                    // empurrar caractere por caractere.
                    builder.push_str(&s[start..i]);
                }

                // Saltamos o byte atual (o espaço duplicado) movendo o cursor de início.
                start = i + 1;
            } else {
                last_was_space = true;
            }
        } else {
            last_was_space = false;
        }
    }

    // 4. FLUSH FINAL:
    // Copia o último segmento de texto após o último espaço duplo encontrado.
    if start < len {
        builder.push_str(&s[start..]);
    }

    builder
}

/// Engine 2: Converte para Uppercase e Colapsa espaços em uma única passagem.
///
/// # Unicode Safety
/// Utiliza `char_indices` para garantir que caracteres multi-byte (acentos/emojis)
/// sejam preservados e convertidos corretamente.
#[inline]
fn clean_and_uppercase_engine<B: StringBuilder>(s: &str) -> B {
    let mut res = B::with_capacity(s.len());
    let mut last_was_space = false;
    let mut start = 0;

    for (idx, c) in s.char_indices() {
        let is_double_space = c == ' ' && last_was_space;
        let is_lower = c.is_lowercase();

        if is_double_space || is_lower {
            // "Flush" de blocos limpos acumulados.
            if idx > start {
                res.push_str(&s[start..idx]);
            }

            if is_lower {
                // Fast-path para ASCII uppercase, fallback para Unicode expansivo.
                if c.is_ascii() {
                    res.push(c.to_ascii_uppercase());
                } else {
                    res.extend(c.to_uppercase());
                }
            }
            // Espaços duplos são ignorados no buffer de saída.
            start = idx + c.len_utf8();
        }
        last_was_space = c == ' ';
    }

    if start < s.len() {
        res.push_str(&s[start..]);
    }
    res
}

/// Verifica se a string precisa de qualquer alteração (Uppercase ou Espaços).
/// Usado como Fast Path para evitar alocações de buffer temporário.
#[inline]
fn needs_normalization(s: &str) -> bool {
    let mut prev_is_space = false;
    for c in s.chars() {
        if c.is_lowercase() || (c == ' ' && prev_is_space) {
            return true;
        }
        prev_is_space = c == ' ';
    }
    false
}

// ============================================================================
// TRAIT: FROMEFDFIELD
// ============================================================================

/// Define como um tipo de domínio (como `GrupoDeContas` ou `CodigoDoCredito`)
/// deve ser construído a partir de uma string bruta lida do arquivo EFD.
pub trait FromEFDField: Sized {
    /// Converte uma string em um tipo de destino, validando as regras de negócio do SPED.
    ///
    /// Recebe metadados (`arquivo`, `linha`, `campo`) para que, em caso de falha,
    /// o erro retornado seja rico em detalhes sobre a localização do problema.
    fn from_efd_field(s: &str, arquivo: &Path, linha: usize, campo: &str) -> EFDResult<Self>;
}

// ============================================================================
// PUBLIC TRAIT: STRINGPARSER
// ============================================================================

/// Trait para parsing e normalização de strings otimizada para SPED.
///
/// Oferece métodos para conversão de tipos primitivos, estruturados e
/// normalização de memória usando `Arc` e `CompactString`.
pub trait StringParser {
    /// Converte o campo para um tipo que implementa [FromStr] (como `u32`, `Decimal`).
    ///
    /// Ignora strings vazias e retorna `None` se o parsing falhar (silencioso).
    fn parse_opt<U: FromStr>(&self) -> Option<U>;

    /// Tenta converter o campo para um tipo estruturado do SPED que implementa [FromEFDField].
    ///
    /// Diferente do [parse_opt], este método é ruidoso: ele propaga erros detalhados
    /// contendo o nome do arquivo, a linha e o nome do campo se a validação falhar.
    fn to_efd_field<U: FromEFDField>(
        &self,
        arquivo: &Path,
        linha: usize,
        campo: &str,
    ) -> EFDResult<Option<U>>;

    /// Converte para `Arc<str>` com espaços colapsados. Útil para dados imutáveis globais.
    fn to_arc(&self) -> Option<Arc<str>>;

    /// Converte para `CompactString` (otimizado para stack até 24 bytes). Ideal para IDs e códigos.
    fn to_compact_string(&self) -> Option<CompactString>;

    /// Aplica Uppercase, colapsa espaços e converte para `Arc<str>`.
    fn to_upper_arc(&self) -> Option<Arc<str>>;

    /// Aplica Uppercase, colapsa espaços e converte para `CompactString`.
    fn to_upper_compact(&self) -> Option<CompactString>;
}

impl<T> StringParser for Option<T>
where
    T: AsRef<str>,
{
    #[inline]
    fn parse_opt<U: FromStr>(&self) -> Option<U> {
        self.as_ref()
            .map(|t| t.as_ref())
            .filter(|s| !s.is_empty())
            .and_then(|s| s.parse().ok())
    }

    #[inline]
    fn to_efd_field<U: FromEFDField>(
        &self,
        arquivo: &Path,
        linha: usize,
        campo: &str,
    ) -> EFDResult<Option<U>> {
        self.as_ref()
            .map(|t| t.as_ref().trim())
            .filter(|s| !s.is_empty())
            .map(|s| U::from_efd_field(s, arquivo, linha, campo))
            .transpose()
    }

    #[inline]
    fn to_arc(&self) -> Option<Arc<str>> {
        self.as_ref()
            .map(|t| t.as_ref())
            .filter(|s| !s.is_empty())
            .map(|s| Arc::from(collapse_multiple_spaces::<String>(s)))
    }

    #[inline]
    fn to_compact_string(&self) -> Option<CompactString> {
        self.as_ref()
            .map(|t| t.as_ref())
            .filter(|s| !s.is_empty())
            .map(collapse_multiple_spaces::<CompactString>)
    }

    fn to_upper_arc(&self) -> Option<Arc<str>> {
        self.as_ref()
            .map(|t| t.as_ref())
            .filter(|s| !s.is_empty())
            .map(|s| {
                if !needs_normalization(s) {
                    Arc::from(s)
                } else {
                    Arc::from(clean_and_uppercase_engine::<String>(s))
                }
            })
    }

    fn to_upper_compact(&self) -> Option<CompactString> {
        self.as_ref()
            .map(|t| t.as_ref())
            .filter(|s| !s.is_empty())
            .map(|s| {
                if !needs_normalization(s) {
                    CompactString::from(s)
                } else {
                    clean_and_uppercase_engine::<CompactString>(s)
                }
            })
    }
}

// --- Exemplo de implementação para os tipos específicos ---

impl FromEFDField for GrupoDeContas {
    fn from_efd_field(s: &str, arquivo: &Path, linha: usize, campo: &str) -> EFDResult<Self> {
        let val = s.parse::<u8>().map_loc(|e| EFDError::ParseIntegerError {
            source: e,
            data_str: s.to_string(),
            campo_nome: campo.to_string(),
            arquivo: arquivo.to_path_buf(),
            line_number: linha,
        })?;
        Self::new(val, arquivo, linha, campo)
    }
}

impl FromEFDField for CodigoDoCredito {
    fn from_efd_field(s: &str, arquivo: &Path, linha: usize, campo: &str) -> EFDResult<Self> {
        let val = s.parse::<u16>().map_loc(|e| EFDError::ParseIntegerError {
            source: e,
            data_str: s.to_string(),
            campo_nome: campo.to_string(),
            arquivo: arquivo.to_path_buf(),
            line_number: linha,
        })?;
        Self::new(val, arquivo, linha, campo)
    }
}

// ============================================================================
// SpedRecordTrait
// ============================================================================

/// Trait fundamental para todos os registros do SPED.
/// Define a interface comum para navegação, identificação e conversão de tipos (downcast).
pub trait SpedRecordTrait: Debug + Any + Send + Sync {
    /// Retorna o nível hierárquico do registro (ex: 0, 1, 2...).
    fn nivel(&self) -> u16;
    /// Retorna o caractere identificador do bloco (ex: 'C', '0', 'M').
    fn bloco(&self) -> char;
    /// Retorna o nome da tag do registro (ex: "C100", "0000").
    fn registro_name(&self) -> &str;
    /// Retorna o número da linha original no arquivo de texto.
    fn line_number(&self) -> usize;
    /// Facilita o downcast imutável para a struct concreta (via std::any::Any).
    fn as_any(&self) -> &dyn Any;
    /// Facilita o downcast mutável para a struct concreta (via std::any::Any).
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Implementação de Encaminhamento (Forwarding) para Option<T>.
///
/// Esta implementação permite que métodos de interface sejam chamados diretamente
/// em um `Option<Registro>`, facilitando o uso em macros de extração onde
/// um registro pai pode ser opcional.
impl<T: SpedRecordTrait> SpedRecordTrait for Option<T> {
    /// Retorna o nível do registro interno ou 0 se for None.
    #[inline(always)]
    fn nivel(&self) -> u16 {
        self.as_ref().map_or(0, |r| r.nivel())
    }

    /// Retorna o número da linha original ou 0 se for None.
    #[inline(always)]
    fn line_number(&self) -> usize {
        self.as_ref().map_or(0, |r| r.line_number())
    }

    /// Retorna o nome do registro (ex: "C100") ou uma string "?" se for None.
    #[inline(always)]
    fn registro_name(&self) -> &str {
        self.as_ref().map_or("?", |r| r.registro_name())
    }

    /// Retorna o caractere do bloco ou '?' se for None.
    #[inline(always)]
    fn bloco(&self) -> char {
        self.as_ref().map_or('?', |r| r.bloco())
    }

    /// Tenta retornar a referência do dado interno como Any.
    ///
    /// Se for `Some`, retorna o Any do registro interno (permitindo downcast para a struct).
    /// Se for `None`, retorna a referência do próprio Option como Any.
    #[inline(always)]
    fn as_any(&self) -> &dyn Any {
        match self {
            Some(r) => r.as_any(),
            None => self,
        }
    }

    /// Tenta retornar a referência mutável do dado interno como Any.
    #[inline(always)]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        match self {
            Some(r) => r.as_any_mut(),
            None => self,
        }
    }
}

// ============================================================================
// ToDecimal
// ============================================================================

/// A trait to convert an `Option<T>` into an `EFDResult<Option<Decimal>>`.
pub trait ToDecimal {
    fn to_decimal(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<Decimal>>;
}

// Trait ToDecimal implementada para Option de qualquer tipo que possa
// ser visto como String (&str, String, CompactString)
impl<T: AsRef<str>> ToDecimal for Option<T> {
    fn to_decimal(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<Decimal>> {
        self.as_ref()
            .map(|s| s.as_ref()) // Obtém a referência &str
            .filter(|s| !s.is_empty()) // Ignora campos vazios sem custo de alocação
            .map(|s| {
                let bytes = s.as_bytes();

                // 1. ANÁLISE PRÉVIA (FAST PATH)
                // Verifica se o campo contém caracteres que exigem tratamento (padrão brasileiro).
                // iter().any() é extremamente rápido e pode ser otimizado por SIMD pelo compilador.
                let needs_cleaning = bytes.iter().any(|&b| b == b',' || b == b'.');

                let result = if needs_cleaning {
                    const MAX: usize = 64; // 64 bytes cobrem qualquer número Decimal suportado (máx 28-29 dígitos)

                    if bytes.len() > MAX {
                        // FALLBACK (Caminho Lento): Se a string for bizarramente longa,
                        // aceitamos o custo de alocação na Heap para garantir a segurança.
                        let cleaned = s.replace('.', "").replace(',', ".");
                        Decimal::from_str_exact(&cleaned)
                    } else {
                        // 2. ZERO-HEAP ALLOCATION (Caminho Rápido/Slow Path Otimizado)
                        // Criamos um buffer fixo na Pilha (Stack). Não toca na memória RAM principal (Heap).
                        let mut buffer = [0u8; MAX];
                        let mut cursor = 0;
                        let mut caracter_invalido = false;

                        // Loop único (O(n)): Remove pontos e converte vírgula em uma única passagem.
                        for &b in bytes {
                            match b {
                                b'.' => continue, // Ignora separador de milhar brasileiro
                                b',' => {
                                    // Converte para ponto (padrão do parser do rust_decimal)
                                    buffer[cursor] = b'.';
                                    cursor += 1;
                                }
                                b'0'..=b'9' => {
                                    // Mantém apenas dígitos
                                    buffer[cursor] = b;
                                    cursor += 1;
                                }
                                b'-' | b'+' => {
                                    // EFICIÊNCIA E SEGURANÇA:
                                    // Sinais só são aceitos se forem o PRIMEIRO caractere do buffer.
                                    if cursor == 0 {
                                        buffer[cursor] = b;
                                        cursor += 1;
                                    } else {
                                        // Sinal no meio do número (ex: 12-34) é erro estrutural.
                                        // Interrompe o loop para economizar CPU.
                                        caracter_invalido = true;
                                        break;
                                    }
                                }
                                _ => {
                                    // Descarta ruídos (espaços, etc), mas sinaliza!
                                    // Espaços ou caracteres não numéricos.
                                    // No SPED, se o campo não estiver limpo, delegamos ao parser original.
                                    caracter_invalido = true;
                                    break;
                                }
                            }
                        }

                        // Se encontrou algo inválido ou o buffer está vazio, fallback para o parse padrão.
                        if cursor == 0 || caracter_invalido {
                            Decimal::from_str_exact(s)
                        } else {
                            // PERFORMANCE: unsafe validado.
                            // Garantimos manualmente que no buffer só existem:
                            // ASCII '0'-'9', '.', '+' ou '-'. Logo, é UTF-8 válido.
                            let cleaned_str =
                                unsafe { std::str::from_utf8_unchecked(&buffer[..cursor]) };
                            Decimal::from_str_exact(cleaned_str)
                        }
                    }
                } else {
                    // FASTEST PATH: Se a string já está limpa, faz o parse direto.
                    // Zero alocações, zero processamento extra.
                    Decimal::from_str_exact(s)
                };

                // 3. LAZY ERROR HANDLING (Tratamento de Erro Preguiçoso)
                // O custo de converter strings de erro e PathBuf só é pago se 'result' for Err.
                // No "caminho feliz" (sucesso), essas alocações nunca acontecem.
                result.map_loc(|source| EFDError::ParseDecimalError {
                    source,
                    valor_str: s.to_string(), // Alocação ocorre apenas aqui (no erro)
                    campo_nome: field_name.to_string(),
                    arquivo: file_path.to_path_buf(),
                    linha_num: line_number,
                })
            })
            .transpose() // Converte Option<EFDResult<D>> para EFDResult<Option<D>>
    }
}

/*
// Otimizado: T: AsRef<str> evita .to_string() imediato
impl<T: AsRef<str>> ToDecimal for Option<T> {
    fn to_decimal(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<Decimal>> {
        self.as_ref()
            .map(|s| s.as_ref())
            .filter(|s| !s.is_empty())
            .map(|s| {
                // 1. Decisão: Precisa de limpeza?
                // Usamos bytes para o check inicial (altíssima performance)
                let bytes = s.as_bytes();
                let needs_cleaning = bytes.iter().any(|&b| b == b',' || b == b'.');

                let result = if needs_cleaning {
                    // 2. Slow Path: Slice-Pushing (Otimizado para "1.234,56")
                    let mut cleaned = String::with_capacity(bytes.len());
                    let mut start = 0;

                    for (i, &b) in bytes.iter().enumerate() {
                        match b {
                            b'.' => {
                                // Pula o ponto de milhar: flush do acumulado até aqui
                                if i > start {
                                    cleaned.push_str(&s[start..i]);
                                }
                                start = i + 1;
                            }
                            b',' => {
                                // Troca decimal: flush e insere ponto padrão
                                if i > start {
                                    cleaned.push_str(&s[start..i]);
                                }
                                cleaned.push('.');
                                start = i + 1;
                            }
                            _ => {} // Digitos e sinais são ignorados aqui e pegos no flush
                        }
                    }

                    // Flush final do último segmento de dígitos
                    if start < bytes.len() {
                        cleaned.push_str(&s[start..]);
                    }

                    Decimal::from_str_exact(&cleaned)
                } else {
                    Decimal::from_str_exact(s)
                };

                // 3. Tratamento de Erro Unificado (DRY)
                result.map_loc(|source| EFDError::ParseDecimalError {
                    source,
                    valor_str: s.to_string(),
                    campo_nome: field_name.to_string(),
                    arquivo: file_path.to_path_buf(),
                    linha_num: line_number,
                })
            })
            .transpose()
    }
}
*/

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

// Otimizado: T: AsRef<str> evita alocações e permite trabalhar com &str, String ou CompactString.
impl<T: AsRef<str>> ToNaiveDate for Option<T> {
    #[track_caller]
    fn to_optional_date(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<NaiveDate>> {
        self.as_ref()
            .map(|s| s.as_ref()) // Obtém a referência &str sem custo de cópia.
            .filter(|s| !s.is_empty())
            .map(|s| {
                let bytes = s.as_bytes();
                let len = bytes.len();
                let date_format = "%-d%-m%Y";

                // 1. FAST PATH (Caminho Ultra-Rápido):
                // Se a string já está limpa e no tamanho certo, evitamos qualquer lógica de filtragem.
                let result = if (len == 8 || len == 6) && bytes.iter().all(|b| b.is_ascii_digit()) {
                    if len == 8 {
                        // DDMMYYYY limpo: Parse direto do slice original. Zero alocação.
                        NaiveDate::parse_from_str(s, date_format)
                    } else {
                        // MMYYYY limpo: Precisa prefixar "01" (dia).
                        // Usamos um buffer de 8 bytes na STACK. Zero alocação na HEAP.
                        let mut buf = [0u8; 8];
                        buf[0] = b'0';
                        buf[1] = b'1';
                        buf[2..8].copy_from_slice(bytes);

                        // SAFETY: O buffer contém apenas '0', '1' e os dígitos ASCII originais.
                        let cleaned = unsafe { std::str::from_utf8_unchecked(&buf) };
                        NaiveDate::parse_from_str(cleaned, date_format)
                    }
                } else {
                    // 2. BRANCHLESS FILTERING (Caminho para strings "sujas" ou com separadores):
                    // Usamos um buffer fixo na STACK para extrair apenas os dígitos.
                    let mut digit_buf = [0u8; 16];
                    let mut cursor = 0;

                    // Percorre no máximo 16 bytes (suficiente para "DD / MM / YYYY")
                    for &b in bytes.iter().take(16) {
                        // (b - '0') < 10: Truque aritmético para verificar se é dígito ASCII.
                        let is_digit = (b.wrapping_sub(b'0') < 10) as usize;

                        // Escrevemos o byte no buffer. Se não for dígito, o cursor não avança
                        // e o próximo byte irá sobrescrever esta posição.
                        // Isso é "branchless": sem 'if', sem quebra de pipeline na CPU.
                        digit_buf[cursor] = b;
                        cursor += is_digit;
                    }

                    match cursor {
                        8 => {
                            // Extraído DDMMYYYY (ex: de "10/02/2024").
                            let cleaned = unsafe { std::str::from_utf8_unchecked(&digit_buf[..8]) };
                            NaiveDate::parse_from_str(cleaned, date_format)
                        }
                        6 => {
                            // Extraído MMYYYY (ex: de "02/2024").
                            let mut full_buf = [0u8; 8];
                            full_buf[0] = b'0';
                            full_buf[1] = b'1';
                            full_buf[2..8].copy_from_slice(&digit_buf[..6]);
                            let cleaned = unsafe { std::str::from_utf8_unchecked(&full_buf) };
                            NaiveDate::parse_from_str(cleaned, date_format)
                        }
                        _ => {
                            // Se o número de dígitos for inválido, tenta o parse original
                            // para que o Chrono gere o erro de formato esperado.
                            NaiveDate::parse_from_str(s, date_format)
                        }
                    }
                };

                // 3. LAZY ERROR HANDLING (Tratamento de Erro Preguiçoso):
                // O s.to_string() e o PathBuf só são criados se o parse falhar.
                // No processamento normal (sucesso), estas alocações pesadas nunca ocorrem.
                result.map_loc(|source| EFDError::ParseDateError {
                    source,
                    data_str: s.to_string(),
                    campo_nome: field_name.to_string(),
                    arquivo: file_path.to_path_buf(),
                    line_number,
                })
            })
            .transpose()
    }

    #[track_caller]
    fn to_date(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<NaiveDate> {
        // 1. Tenta obter a data opcional (erros internos de parsing já vêm com loc)
        // 2. Se for Ok(None), o map_loc do Option transforma em Err + localização
        self.to_optional_date(file_path, line_number, field_name)?
            .map_loc(|_| EFDError::KeyNotFound(field_name.to_string()))
    }
}

/*
// Otimizado: Zero allocation no caminho feliz para datas completas (8 chars)
impl<T: AsRef<str>> ToNaiveDate for Option<T> {
    fn to_optional_date(
        &self,
        file_path: &Path,
        line_number: usize,
        field_name: &str,
    ) -> EFDResult<Option<NaiveDate>> {
        self.as_ref()
            .map(|s| s.as_ref())
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

                result.map_loc(|source| EFDError::ParseDateError {
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
*/

/// Trait para converter `Option<T>` em `Option<String>` (trimada e não vazia).
pub trait ToOptionalString {
    fn to_optional_string(&self) -> Option<String>;
}

impl<T: AsRef<str>> ToOptionalString for Option<T> {
    fn to_optional_string(&self) -> Option<String> {
        self.as_ref()
            .map(|s| s.as_ref())
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
            .map(|s| s.as_ref())
            .filter(|s| !s.is_empty())
            .map(|s| {
                s.parse::<U>().map_loc(|e| EFDError::ParseIntegerError {
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

/// Trait para validação e conversão de CNPJ (Alfanumérico 14 dígitos).
pub trait ToCNPJ {
    /// Limpa e valida o CNPJ. Retorna Some(Arc<str>) se válido, None se vazio, ou Erro se inválido.
    fn to_optional_cnpj(
        &self,
        file_path: &Path,
        line_number: usize,
        registro: &str,
        field_name: &str,
    ) -> EFDResult<Option<Arc<str>>>;

    /// Versão obrigatória: Retorna erro se o campo estiver vazio.
    fn to_cnpj(
        &self,
        file_path: &Path,
        line_number: usize,
        registro: &str,
        field_name: &str,
    ) -> EFDResult<Arc<str>>;
}

impl<T: AsRef<str>> ToCNPJ for Option<T> {
    fn to_optional_cnpj(
        &self,
        file_path: &Path,
        line_number: usize,
        registro: &str,
        field_name: &str,
    ) -> EFDResult<Option<Arc<str>>> {
        self.as_ref()
            .map(|s| s.as_ref())
            .filter(|s| !s.is_empty()) // Ignora se for apenas espaços
            .map(|s| {
                // 1. Filtragem funcional: mantém apenas A-Z e 0-9, converte para Uppercase
                let cleaned: String = s
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric())
                    .map(|c| c.to_ascii_uppercase())
                    .collect();

                // 2. Validação de tamanho (CNPJ deve ter exatamente 14 caracteres)
                if cleaned.len() == 14 {
                    Ok(Arc::from(cleaned))
                } else {
                    // 3. Emite erro específico caso o formato seja inválido
                    Err(EFDError::InvalidCNPJ {
                        arquivo: file_path.to_path_buf(),
                        linha_num: line_number,
                        registro: registro.to_string(),
                        campo_nome: field_name.to_string(),
                        cnpj: s.to_string(), // Mostra o valor original que falhou
                    })
                    .loc()
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
    ) -> EFDResult<Arc<str>> {
        self.to_optional_cnpj(file_path, line_number, registro, field_name)?
            .map_loc(|_| EFDError::KeyNotFound(field_name.to_string()))
    }
}

// --- Result Extension --- //

/// Extensão para Result e Option para suporte a rastreamento de erros.
pub trait ResultExt<T, E> {
    /// Converte o erro para EFDError e carimba a localização.
    ///
    /// A restrição de conversão (Into) fica apenas neste método.
    fn loc(self) -> Result<T, EFDError>
    where
        E: Into<EFDError>;

    /// Mapeia o erro original para um EFDError customizado e carimba a localização.
    fn map_loc<F>(self, op: F) -> Result<T, EFDError>
    where
        F: FnOnce(E) -> EFDError;
}

// --- Implementação para Result ---
impl<T, E> ResultExt<T, E> for Result<T, E> {
    #[track_caller]
    fn loc(self) -> Result<T, EFDError>
    where
        E: Into<EFDError>,
    {
        // Captura a localização do CHAMADOR antes de entrar na closure
        let caller = Location::caller();
        self.map_err(|e| e.into().tag(caller))
    }

    #[track_caller]
    fn map_loc<F>(self, op: F) -> Result<T, EFDError>
    where
        F: FnOnce(E) -> EFDError,
    {
        let caller = Location::caller();
        self.map_err(|e| op(e).tag(caller))
    }
}

// --- Implementação para Option ---
impl<T> ResultExt<T, ()> for Option<T> {
    #[track_caller]
    fn loc(self) -> Result<T, EFDError> {
        let caller = Location::caller();
        self.ok_or_else(|| EFDError::KeyNotFound("Valor obrigatório ausente".into()).tag(caller))
    }

    #[track_caller]
    fn map_loc<F>(self, op: F) -> Result<T, EFDError>
    where
        F: FnOnce(()) -> EFDError,
    {
        let caller = Location::caller();
        // IMPORTANTE: Não delegue para Result::map_loc,
        // senão o caller passará a ser este arquivo.
        match self {
            Some(v) => Ok(v),
            None => Err(op(()).tag(caller)),
        }
    }
}

/// Helper para disparar erros diretamente de forma fluente.
pub trait EfdRaise<T> {
    fn raise(self) -> Result<T, EFDError>;
}

impl<T> EfdRaise<T> for EFDError {
    #[track_caller]
    fn raise(self) -> Result<T, EFDError> {
        // Captura aqui para garantir que o erro aponte para onde o .raise() foi chamado
        let caller = Location::caller();
        Err(self.tag(caller))
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
