use regex::Regex;
use std::sync::LazyLock;

/*
Regex, flags.
All flags are by default disabled unless stated otherwise. They are:

    i     case-insensitive: letters match both upper and lower case
    m     multi-line mode: ^ and $ match begin/end of line
    s     allow . to match \n
    R     enables CRLF mode: when multi-line mode is enabled, \r\n is used
    U     swap the meaning of x* and x*?
    u     Unicode support (enabled by default)
    x     verbose mode, ignores whitespace and allow line comments (starting with `#`)
*/

pub static REGEX_REMOVE_NON_DIGITS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?isx)
        \D
    ",
    )
    .unwrap()
});

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
