use regex::Regex;
use std::sync::LazyLock;

// Regex, flags:
// x: verbose mode, ignores whitespace and allow line comments (starting with `#`)
// i: case-insensitive: letters match both upper and lower case

pub static REGEX_CENTER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)
        # non-capturing group: (?:regex)
        ^(:?
            CNPJ|CPF|CST|
            Chave|NCM|
            Registro|Identifica|
            Cancelado|
            Estado|
            Tomador|
            Ano|Trimestre|Mês|
            Tipo|
            Indicador|
            Nº\sdo
        )|
        Código|
        Versão|
        Linha
    ",
    )
    .unwrap()
});

pub static REGEX_VALUE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)
        ^(:?
            Base\sde\sCálculo|
            Crédito\svinculado
        )|
        Total|Valor
    ",
    )
    .unwrap()
});

pub static REGEX_ALIQ: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)
        Alíquota
    ",
    )
    .unwrap()
});

pub static REGEX_DATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)
        ^(:?Data|Dia|Período\sde\sApuração)
    ",
    )
    .unwrap()
});
