use super::*;

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
    if let Err(EFDError::ParseIntegerError {
        data_str,
        campo_nome,
        arquivo,
        line_number,
        ..
    }) = result
    {
        assert_eq!(data_str, "abc");
        assert_eq!(campo_nome, field);
        assert_eq!(arquivo, path);
        assert_eq!(line_number, line);
    } else {
        panic!("Expected ParseIntError but got {:?}", result);
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
    if let Err(EFDError::ParseIntegerError {
        data_str,
        campo_nome,
        arquivo,
        line_number,
        source,
    }) = result
    {
        assert_eq!(data_str, "256");
        assert_eq!(campo_nome, field);
        assert_eq!(arquivo, path);
        assert_eq!(line_number, line);
        // Verifica o tipo de erro de parse subjacente, se possível
        assert_eq!(source.kind(), &std::num::IntErrorKind::PosOverflow);
    } else {
        panic!("Expected ParseIntError but got {:?}", result);
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
