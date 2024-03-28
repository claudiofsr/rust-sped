use once_cell::sync::Lazy;
use regex::Regex;

// Regex, flags:
// x: verbose mode, ignores whitespace and allow line comments (starting with `#`)
// i: case-insensitive: letters match both upper and lower case

/// Example:
///
/// <https://docs.rs/once_cell/latest/once_cell/sync/struct.Lazy.html>
pub static REGEX_CENTER: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?ix)
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
    ").unwrap()
);

pub static REGEX_VALUE: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?ix)
        ^(:?
            Base\sde\sCálculo|
            Crédito\svinculado
        )|
        Total|Valor
    ").unwrap()
);

pub static REGEX_ALIQ: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?ix)
        Alíquota
    ").unwrap()
);

pub static REGEX_DATE: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?ix)
        ^(:?Data|Dia|Período\sde\sApuração)
    ").unwrap()
);
