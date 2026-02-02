use super::*;
use rust_decimal_macros::dec;
use std::path::PathBuf;

// --- Novos testes para ToOptionalInteger ---

#[test]
fn to_optional_integer_success_u64() -> EFDResult<()> {
    let path = Path::new("test_file.txt");
    let line = 1;
    let field = "ID";

    let input: Option<&str> = Some("12345");
    let result: Option<u64> = input.to_optional_integer(path, line, field)?;
    assert_eq!(result, Some(12345u64));
    Ok(())
}

#[test]
fn to_optional_integer_success_i32() -> EFDResult<()> {
    let path = Path::new("test_file.txt");
    let line = 2;
    let field = "Valor";

    let input: Option<&&str> = Some(&"-500");
    let result: Option<i32> = input.to_optional_integer(path, line, field)?;
    assert_eq!(result, Some(-500i32));
    Ok(())
}

#[test]
fn to_optional_integer_empty_string_returns_none() -> EFDResult<()> {
    let path = Path::new("test_file.txt");
    let line = 3;
    let field = "Quantidade";

    let input: Option<&&str> = Some(&"");
    let result: Option<u16> = input.to_optional_integer(path, line, field)?;
    assert!(result.is_none());
    Ok(())
}

#[test]
fn to_optional_integer_none_input_returns_none() -> EFDResult<()> {
    let path = Path::new("test_file.txt");
    let line = 4;
    let field = "Código";

    let input: Option<&&str> = None;
    let result: Option<usize> = input.to_optional_integer(path, line, field)?;
    assert!(result.is_none());
    Ok(())
}

#[test]
fn to_optional_integer_invalid_string_returns_error() {
    let path = Path::new("test_file.txt");
    let line = 5;
    let field = "Preço";

    let input: Option<&&str> = Some(&"abc");
    let result: Result<Option<u32>, EFDError> = input.to_optional_integer(path, line, field);

    assert!(result.is_err());

    // 1. Extraímos o erro com unwrap_err()
    // 2. Limpamos o wrapper de localização com flatten()
    if let EFDError::ParseIntegerError {
        data_str,
        campo_nome,
        arquivo,
        line_number,
        source,
    } = result.unwrap_err().flatten()
    {
        assert_eq!(data_str, "abc");
        assert_eq!(campo_nome, field);
        assert_eq!(arquivo, path);
        assert_eq!(line_number, line);
        // Opcional: Verifica se o tipo de erro interno é de dígito inválido
        assert_eq!(source.kind(), &std::num::IntErrorKind::InvalidDigit);
    } else {
        panic!("Falha ao encontrar ParseIntegerError");
    }
}

#[test]
/// cargo test -- --show-output integer_overflow
fn to_optional_integer_overflow_returns_error() {
    let path = Path::new("test_file.txt");
    let line = 6;
    let field = "PequenoID";

    let input: Option<&&str> = Some(&"256"); // Max u8 is 255
    let result: Result<Option<u8>, EFDError> = input.to_optional_integer(path, line, field);

    assert!(result.is_err());

    // flatten() limpa o wrapper de localização para o teste focar no dado
    if let EFDError::ParseIntegerError {
        data_str,
        campo_nome,
        arquivo,
        line_number,
        source,
    } = result.unwrap_err().flatten()
    {
        assert_eq!(data_str, "256");
        assert_eq!(campo_nome, field);
        assert_eq!(arquivo, path);
        assert_eq!(line_number, line);
        // Verifica o tipo de erro de parse subjacente, se possível
        assert_eq!(source.kind(), &std::num::IntErrorKind::PosOverflow);
    } else {
        panic!("Falha ao encontrar ParseIntegerError");
    }
}

/// cargo test -- --show-output test_string_parser
#[test]
fn test_string_parser() {
    let campo_a: Option<String> = Some("07".to_string());
    println!("campo_a: '{campo_a:?}'");
    let resultado_a: Option<u16> = campo_a.parse_opt();
    println!("resultado_a: '{resultado_a:?}'\n");

    assert_eq!(resultado_a, Some(7));

    let campo_b: Option<&str> = Some("00123");
    println!("campo_b: '{campo_b:?}'");
    let resultado_b: Option<u16> = campo_b.parse_opt();
    println!("resultado_b: '{resultado_b:?}'\n");

    assert_eq!(resultado_b, Some(123));

    let campo_c: Option<String> = Some(" 54".to_string());
    println!("campo_c: '{campo_c:?}'");
    let resultado_c: Option<u16> = campo_c.parse_opt();
    println!("resultado_c: '{resultado_c:?}'\n");

    assert!(resultado_c.is_none());

    let campo_d: Option<String> = Some("".to_string());
    println!("campo_d: '{campo_d:?}'");
    let resultado_d: Option<u16> = campo_d.parse_opt();
    println!("resultado_d: '{resultado_d:?}'\n");

    assert!(resultado_d.is_none());

    let campo_e: Option<String> = None;
    println!("campo_e: '{campo_e:?}'");
    let resultado_e: Option<u16> = campo_e.parse_opt();
    println!("resultado_e: '{resultado_e:?}'\n");

    assert!(resultado_e.is_none());
}

// --- CompactString --- //

/// cargo test -- --show-output test_to_compact_strings
#[test]
fn test_to_compact_string_complex_utf8() {
    // Caso 1: Acentuação Brasileira e espaços triplos
    let s1 = Some("CRÉDITO   DE   ICMS   SÃO   PAULO");
    assert_eq!(s1.to_compact_string().unwrap(), "CRÉDITO DE ICMS SÃO PAULO");

    // Caso 2: Caracteres especiais e símbolos de moeda
    let s2 = Some("VALOR  €  1.250,50  §    123");
    assert_eq!(s2.to_compact_string().unwrap(), "VALOR € 1.250,50 § 123");

    // Caso 3: Emojis (O teste definitivo de UTF-8)
    // Um emoji como 🚀 ocupa 4 bytes. Se o parser for por byte, ele quebraria aqui.
    let s3 = Some("AVISO    🚀        NOTIFICAÇÃO    ✅");
    assert_eq!(s3.to_compact_string().unwrap(), "AVISO 🚀 NOTIFICAÇÃO ✅");

    // Caso 4: Mistura de Scripts (Cirílico, Grego e Chinês)
    let s4 = Some("SPED  Contribuições  Σ  П    你好");
    assert_eq!(
        s4.to_compact_string().unwrap(),
        "SPED Contribuições Σ П 你好"
    );

    // Caso 5: Zero alocação (String curta com caracteres especiais)
    // "Maçã" tem 4 caracteres mas 5 bytes (ç = 2 bytes).
    // Deve caber nos 24 bytes de stack da CompactString.
    let s5 = Some("MAÇÃ  BOA");
    let res = s5.to_compact_string().unwrap();
    assert_eq!(res, "MAÇÃ BOA");
    assert!(!res.is_heap_allocated()); // Verifica se não foi para a Heap
}

#[test]
fn test_to_compact_string_edge_cases() {
    // Espaços intercalados com caracteres
    let input = Some("A B  C   D    E");
    assert_eq!(input.to_compact_string().unwrap(), "A B C D E");

    // String que já está correta mas contém UTF-8 pesado
    let correct = Some("SÃO_PAULO_🚀");
    assert_eq!(correct.to_compact_string().unwrap(), "SÃO_PAULO_🚀");
}

/// cargo test -- --show-output test_to_compact_string_multibyte
#[test]
fn test_to_compact_string_multibyte_integrity() {
    // Ç: 2 bytes (Caractere multi-byte)
    // Ã: 2 bytes (Caractere multi-byte)
    let input = Some("AÇÃO     2024");
    let result_opt = input.to_compact_string();
    assert!(result_opt.is_some());
    let result = result_opt.unwrap();

    assert_eq!(result, "AÇÃO 2024");
    assert_eq!(
        result.chars().count(),
        9,
        "Should have exactly 9 characters (UTF-8 awareness)"
    );

    assert_eq!(result.len(), 11, "Should have exactly 10 bytes");

    // 2. The Emoji Test (4-byte character) - Critical for byte-alignment
    // 🚀 is represented by 4 bytes.
    let input = Some("System   🚀   Online");
    let result = input.to_compact_string().unwrap();
    assert_eq!(result, "System 🚀 Online");

    // Byte length: 6 (System) + 1 (space) + 4 (Emoji) + 1 (space) + 6 (Online) = 18
    assert_eq!(result.len(), 18, "Byte length should be exactly 18");

    // 3. The Euro Test (3-byte character)
    let input = Some("Price:   100€   only!");
    let result = input.to_compact_string().unwrap();
    assert_eq!(result, "Price: 100€ only!");
    assert_eq!(result.len(), 19);
    assert!(!result.is_heap_allocated());

    // 4. Combined Stress Test (Leading, trailing and multiple middle spaces)
    let input = Some("  SÃO   PAULO  🚀    €  ");
    let result = input.to_compact_string().unwrap();
    // collapse multiple spaces into single ones
    assert_eq!(result, " SÃO PAULO 🚀 € ");
    assert_eq!(result.len(), 21);
    assert!(result.is_heap_allocated());
}

/// cargo test -- --show-output test_to_compact_string_boundaries
#[test]
fn test_to_compact_string_specific_byte_boundaries() {
    // Ensures we don't slice in the middle of a multi-byte char
    // Á (U+00C1) is [195, 129] in UTF-8
    let input = Some("ABCÁ     DEF");
    let result = input.to_compact_string().unwrap();

    assert_eq!(result, "ABCÁ DEF");

    // Verify it's still a valid CompactString and fits in stack (inline)
    // "ABCÁ DEF" is 9 bytes total, should be inline (no heap allocation)
    assert_eq!(result.len(), 9);
    assert!(!result.is_heap_allocated());
}

/// cargo test -- --show-output test_to_compact_string_none_empty
#[test]
fn test_to_compact_string_none_and_empty() {
    // None input should return None
    let input: Option<&str> = None;
    assert!(input.to_compact_string().is_none());

    // Empty string input should return None (standardized behavior for SPED)
    let input = Some("");
    assert!(input.to_compact_string().is_none());

    // String with only spaces should return a single space or be handled by logic
    let input = Some("   ");
    assert_eq!(input.to_compact_string().unwrap(), " ");
}

// --- ToDeciaml --- //
// cargo test -- --show-output decimal

// Helper para evitar repetição nos testes
fn mock_context() -> (PathBuf, usize, &'static str) {
    (PathBuf::from("test_file.txt"), 123, "valor_item")
}

#[test]
fn test_to_decimal_sucesso_formatos_brasileiros() -> EFDResult<()> {
    let (path, line, field) = mock_context();

    // Formato comum: vírgula como decimal
    let input = Some("1234,56");
    let result = input.to_decimal(&path, line, field)?;
    assert_eq!(result, Some(dec!(1234.56)));

    // Formato com ponto em milhar e vírgula em decimal
    let input = Some("1.234.567,89");
    let result = input.to_decimal(&path, line, field)?;
    assert_eq!(result, Some(dec!(1234567.89)));

    // Número negativo com formatação
    let input = Some("-1.500,00");
    let result = input.to_decimal(&path, line, field)?;
    assert_eq!(result, Some(dec!(-1500.00)));

    // Apenas parte inteira
    let input = Some("1000");
    let result = input.to_decimal(&path, line, field)?;
    assert_eq!(result, Some(dec!(1000)));

    // Sinal de mais (explícito)
    let input = Some("+500,00");
    let result = input.to_decimal(&path, line, field)?;
    assert_eq!(result, Some(dec!(500.00)));

    // Zero
    let input = Some("0,00");
    let result = input.to_decimal(&path, line, field)?;
    assert_eq!(result, Some(dec!(0.0)));

    Ok(())
}

#[test]
fn test_to_decimal_none_e_vazio() -> EFDResult<()> {
    let (path, line, field) = mock_context();

    // Caso None
    let input: Option<&str> = None;
    let result = input.to_decimal(&path, line, field)?;
    assert!(result.is_none());

    // Caso String vazia
    let input = Some("");
    let result = input.to_decimal(&path, line, field)?;
    assert!(result.is_none());

    Ok(())
}

#[test]
fn test_to_decimal_erro_caracteres_invalidos() -> EFDResult<()> {
    let (path, line, field) = mock_context();

    // 1. Letras no meio do número
    let input = Some("123A,56");
    let result = input.to_decimal(&path, line, field);

    // Mapeamos o erro para remover a "casca" de localização antes do match
    match result.map_err(|e| e.flatten()) {
        Err(EFDError::ParseDecimalError { valor_str, .. }) => {
            assert_eq!(valor_str, "123A,56");
        }
        res => panic!(
            "Deveria ter retornado ParseDecimalError, mas retornou: {:?}",
            res
        ),
    }

    // 2. Sinal no meio do número
    let input = Some("123-4,56");
    let result = input.to_decimal(&path, line, field);

    match result.map_err(|e| e.flatten()) {
        Err(EFDError::ParseDecimalError { valor_str, .. }) => {
            assert_eq!(valor_str, "123-4,56");
        }
        res => panic!(
            "Deveria ter retornado ParseDecimalError, mas retornou: {:?}",
            res
        ),
    }

    // 3. Dois separadores decimais (inválido para o parser de Decimal)
    let input = Some("12,34,56");
    let result = input.to_decimal(&path, line, field);

    // Verificação simples de erro ignorando a variante interna
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e.flatten(), EFDError::ParseDecimalError { .. }));
    }

    Ok(())
}
