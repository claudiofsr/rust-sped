use regex::{Regex, RegexSet};
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

pub static FORMAT_REGEX_SET: LazyLock<RegexSet> = LazyLock::new(|| {
    RegexSet::new([
        // INDEX_VALUE (0)
        r"(?i)^(?:Base de Cálculo|Crédito Vinculado)|Total|Valor",
        // INDEX_ALIQ (1)
        r"(?i)Alíquota",
        // INDEX_DATE (2)
        r"(?i)^(?:Data da Emissão|Data da Entrada|Período de Apuração)",
        // INDEX_CENTER (3)
        r"(?ix)^(?:
            CNPJ|CPF|CST|Chave|NCM|Registro|Identifica|
            Cancelado|Estado|Tomador|Ano|Trimestre|Mês|
            Tipo\sde\sOperação|Indicador|Nº\sdo
        )
        |Código|Versão|Linha",
    ])
    .unwrap()
});
