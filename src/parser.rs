use crate::{DELIMITER_CHAR, EFDResult, blocos::*, dispatch_sped_parsers, model::*};
use std::path::Path;

/// A trait for parsing different types of SPED records.
pub trait SpedParser {
    type Output; // Geralmente será a própria Struct (Self)
    fn parse_reg(file_path: &Path, line_number: usize, fields: &[&str]) -> EFDResult<Self::Output>;
}

/// Função central de processamento de linhas do SPED EFD Contribuições.
///
/// Converte uma linha bruta do arquivo texto em uma variante da hierarquia SpedRecord.
pub fn parse_sped_fields(
    file_path: &Path,
    line_number: usize,
    line: &str,
) -> EFDResult<Option<SpedRecord>> {
    // Convertemos a string para um slice de bytes para processamento em nível de CPU.
    // Trabalhar com bytes ([u8]) é ordens de grandeza mais rápido que trabalhar com caracteres Unicode (char).
    let bytes = line.as_bytes();

    // Buffer fixo na Stack (Pilha).
    // É usado apenas se precisarmos consertar um registro minúsculo (ex: |c100|).
    // Por estar na Stack, o custo de criação é virtualmente zero (não toca na Heap/RAM principal).
    let mut reg_buffer = [0u8; 4];

    let registro_uppercase: &str = match bytes.get(0..6) {
        // Capturamos os 4 bytes do ID (a, b, c, d) diretamente para registradores.
        // Registro segue o padrão: ADDD, tal que A é alfanumerico e D dígito.
        Some([b'|', a, b, c, d, b'|']) => {
            // Unificamos a lógica: se for minúsculo, bitwise UP, senão mantém.
            // O SPED permite 0-9 e A-Z. Apenas 'a'-'z' (0x61-0x7A) precisa de fix.
            if a.is_ascii_lowercase() {
                reg_buffer[0] = a & !0x20; // Trick bitwise: força maiúsculo
                reg_buffer[1] = *b; // Mantém o dígito original
                reg_buffer[2] = *c; // Mantém o dígito original
                reg_buffer[3] = *d; // Mantém o dígito original

                // Como validamos que os bytes vêm de uma string UTF-8 e apenas aplicamos
                // operações ASCII seguras, pulamos a validação de UTF-8 do Rust (que é cara).
                unsafe { std::str::from_utf8_unchecked(&reg_buffer) }
            } else {
                &line[1..5]
            }
        }
        // Se a linha não começar com "|XXXX|", é ignorada instantaneamente.
        _ => return Ok(None),
    };

    // Tokenização por referências (&str): Não aloca novas strings na Heap.
    // O map(str::trim) é importante para limpar espaços em branco ao redor dos campos.
    let fields: Vec<&str> = line.split(DELIMITER_CHAR).map(str::trim).collect();

    // Verificação de segurança pós-split
    if fields.len() < 4 {
        return Ok(None);
    }

    // Chamada da Macro com mapeamento simplificado
    // 4. Despachante principal (Match exaustivo por Bloco)
    dispatch_sped_parsers!(
        registro_uppercase, // Passamos o &str normalizado (ex: "C100")
        file_path,
        line_number,
        &fields, // Pass a slice of fields

        simple => [
            // Bloco 0
            ("0000", Bloco0, R0000, Registro0000), ("0001", Bloco0, R0001, Registro0001),
            ("0035", Bloco0, R0035, Registro0035), ("0100", Bloco0, R0100, Registro0100),
            ("0110", Bloco0, R0110, Registro0110), ("0111", Bloco0, R0111, Registro0111),
            ("0120", Bloco0, R0120, Registro0120), ("0140", Bloco0, R0140, Registro0140),
            ("0145", Bloco0, R0145, Registro0145), ("0150", Bloco0, R0150, Registro0150),
            ("0190", Bloco0, R0190, Registro0190), ("0200", Bloco0, R0200, Registro0200),
            ("0205", Bloco0, R0205, Registro0205), ("0206", Bloco0, R0206, Registro0206),
            ("0208", Bloco0, R0208, Registro0208), ("0400", Bloco0, R0400, Registro0400),
            ("0450", Bloco0, R0450, Registro0450), ("0500", Bloco0, R0500, Registro0500),
            ("0600", Bloco0, R0600, Registro0600), ("0900", Bloco0, R0900, Registro0900),
            ("0990", Bloco0, R0990, Registro0990),

            // Bloco 1
            ("1001", Bloco1, R1001, Registro1001), ("1010", Bloco1, R1010, Registro1010),
            ("1011", Bloco1, R1011, Registro1011), ("1020", Bloco1, R1020, Registro1020),
            ("1050", Bloco1, R1050, Registro1050), ("1100", Bloco1, R1100, Registro1100),
            ("1101", Bloco1, R1101, Registro1101), ("1102", Bloco1, R1102, Registro1102),
            ("1200", Bloco1, R1200, Registro1200), ("1210", Bloco1, R1210, Registro1210),
            ("1220", Bloco1, R1220, Registro1220), ("1300", Bloco1, R1300, Registro1300),
            ("1500", Bloco1, R1500, Registro1500), ("1501", Bloco1, R1501, Registro1501),
            ("1502", Bloco1, R1502, Registro1502), ("1600", Bloco1, R1600, Registro1600),
            ("1610", Bloco1, R1610, Registro1610), ("1620", Bloco1, R1620, Registro1620),
            ("1700", Bloco1, R1700, Registro1700), ("1800", Bloco1, R1800, Registro1800),
            ("1809", Bloco1, R1809, Registro1809), ("1900", Bloco1, R1900, Registro1900),
            ("1990", Bloco1, R1990, Registro1990),

            // Bloco A
            ("A001", BlocoA, RA001, RegistroA001), ("A010", BlocoA, RA010, RegistroA010),
            ("A100", BlocoA, RA100, RegistroA100), ("A110", BlocoA, RA110, RegistroA110),
            ("A111", BlocoA, RA111, RegistroA111), ("A120", BlocoA, RA120, RegistroA120),
            ("A170", BlocoA, RA170, RegistroA170), ("A990", BlocoA, RA990, RegistroA990),

            // Bloco C
            ("C001", BlocoC, RC001, RegistroC001), ("C010", BlocoC, RC010, RegistroC010),
            ("C100", BlocoC, RC100, RegistroC100), ("C110", BlocoC, RC110, RegistroC110),
            ("C111", BlocoC, RC111, RegistroC111), ("C120", BlocoC, RC120, RegistroC120),
            ("C170", BlocoC, RC170, RegistroC170), ("C175", BlocoC, RC175, RegistroC175),
            ("C180", BlocoC, RC180, RegistroC180), ("C181", BlocoC, RC181, RegistroC181),
            ("C185", BlocoC, RC185, RegistroC185), ("C188", BlocoC, RC188, RegistroC188),
            ("C190", BlocoC, RC190, RegistroC190), ("C191", BlocoC, RC191, RegistroC191),
            ("C195", BlocoC, RC195, RegistroC195), ("C198", BlocoC, RC198, RegistroC198),
            ("C199", BlocoC, RC199, RegistroC199), ("C380", BlocoC, RC380, RegistroC380),
            ("C381", BlocoC, RC381, RegistroC381), ("C385", BlocoC, RC385, RegistroC385),
            ("C395", BlocoC, RC395, RegistroC395), ("C396", BlocoC, RC396, RegistroC396),
            ("C400", BlocoC, RC400, RegistroC400), ("C405", BlocoC, RC405, RegistroC405),
            ("C481", BlocoC, RC481, RegistroC481), ("C485", BlocoC, RC485, RegistroC485),
            ("C489", BlocoC, RC489, RegistroC489), ("C490", BlocoC, RC490, RegistroC490),
            ("C491", BlocoC, RC491, RegistroC491), ("C495", BlocoC, RC495, RegistroC495),
            ("C499", BlocoC, RC499, RegistroC499), ("C500", BlocoC, RC500, RegistroC500),
            ("C501", BlocoC, RC501, RegistroC501), ("C505", BlocoC, RC505, RegistroC505),
            ("C509", BlocoC, RC509, RegistroC509), ("C600", BlocoC, RC600, RegistroC600),
            ("C601", BlocoC, RC601, RegistroC601), ("C605", BlocoC, RC605, RegistroC605),
            ("C609", BlocoC, RC609, RegistroC609), ("C800", BlocoC, RC800, RegistroC800),
            ("C810", BlocoC, RC810, RegistroC810), ("C820", BlocoC, RC820, RegistroC820),
            ("C830", BlocoC, RC830, RegistroC830), ("C860", BlocoC, RC860, RegistroC860),
            ("C870", BlocoC, RC870, RegistroC870), ("C880", BlocoC, RC880, RegistroC880),
            ("C890", BlocoC, RC890, RegistroC890), ("C990", BlocoC, RC990, RegistroC990),

            // Bloco D
            ("D001", BlocoD, RD001, RegistroD001), ("D010", BlocoD, RD010, RegistroD010),
            ("D100", BlocoD, RD100, RegistroD100), ("D101", BlocoD, RD101, RegistroD101),
            ("D105", BlocoD, RD105, RegistroD105), ("D111", BlocoD, RD111, RegistroD111),
            ("D200", BlocoD, RD200, RegistroD200), ("D201", BlocoD, RD201, RegistroD201),
            ("D205", BlocoD, RD205, RegistroD205), ("D209", BlocoD, RD209, RegistroD209),
            ("D300", BlocoD, RD300, RegistroD300), ("D309", BlocoD, RD309, RegistroD309),
            ("D350", BlocoD, RD350, RegistroD350), ("D359", BlocoD, RD359, RegistroD359),
            ("D500", BlocoD, RD500, RegistroD500), ("D501", BlocoD, RD501, RegistroD501),
            ("D505", BlocoD, RD505, RegistroD505), ("D509", BlocoD, RD509, RegistroD509),
            ("D600", BlocoD, RD600, RegistroD600), ("D601", BlocoD, RD601, RegistroD601),
            ("D605", BlocoD, RD605, RegistroD605), ("D609", BlocoD, RD609, RegistroD609),
            ("D990", BlocoD, RD990, RegistroD990),

            // Bloco F
            ("F001", BlocoF, RF001, RegistroF001), ("F010", BlocoF, RF010, RegistroF010),
            ("F100", BlocoF, RF100, RegistroF100), ("F111", BlocoF, RF111, RegistroF111),
            ("F120", BlocoF, RF120, RegistroF120), ("F129", BlocoF, RF129, RegistroF129),
            ("F130", BlocoF, RF130, RegistroF130), ("F139", BlocoF, RF139, RegistroF139),
            ("F150", BlocoF, RF150, RegistroF150), ("F200", BlocoF, RF200, RegistroF200),
            ("F205", BlocoF, RF205, RegistroF205), ("F210", BlocoF, RF210, RegistroF210),
            ("F211", BlocoF, RF211, RegistroF211), ("F500", BlocoF, RF500, RegistroF500),
            ("F509", BlocoF, RF509, RegistroF509), ("F510", BlocoF, RF510, RegistroF510),
            ("F519", BlocoF, RF519, RegistroF519), ("F525", BlocoF, RF525, RegistroF525),
            ("F550", BlocoF, RF550, RegistroF550), ("F559", BlocoF, RF559, RegistroF559),
            ("F560", BlocoF, RF560, RegistroF560), ("F569", BlocoF, RF569, RegistroF569),
            ("F600", BlocoF, RF600, RegistroF600), ("F700", BlocoF, RF700, RegistroF700),
            ("F800", BlocoF, RF800, RegistroF800), ("F990", BlocoF, RF990, RegistroF990),

            // Bloco I
            ("I001", BlocoI, RI001, RegistroI001), ("I010", BlocoI, RI010, RegistroI010),
            ("I100", BlocoI, RI100, RegistroI100), ("I199", BlocoI, RI199, RegistroI199),
            ("I200", BlocoI, RI200, RegistroI200), ("I299", BlocoI, RI299, RegistroI299),
            ("I300", BlocoI, RI300, RegistroI300), ("I399", BlocoI, RI399, RegistroI399),
            ("I990", BlocoI, RI990, RegistroI990),

            // Bloco M
            ("M001", BlocoM, RM001, RegistroM001), ("M100", BlocoM, RM100, RegistroM100),
            ("M105", BlocoM, RM105, RegistroM105), ("M110", BlocoM, RM110, RegistroM110),
            ("M115", BlocoM, RM115, RegistroM115), ("M200", BlocoM, RM200, RegistroM200),
            ("M205", BlocoM, RM205, RegistroM205), ("M211", BlocoM, RM211, RegistroM211),
            ("M215", BlocoM, RM215, RegistroM215), ("M220", BlocoM, RM220, RegistroM220),
            ("M225", BlocoM, RM225, RegistroM225), ("M230", BlocoM, RM230, RegistroM230),
            ("M300", BlocoM, RM300, RegistroM300), ("M350", BlocoM, RM350, RegistroM350),
            ("M400", BlocoM, RM400, RegistroM400), ("M410", BlocoM, RM410, RegistroM410),
            ("M500", BlocoM, RM500, RegistroM500), ("M505", BlocoM, RM505, RegistroM505),
            ("M510", BlocoM, RM510, RegistroM510), ("M515", BlocoM, RM515, RegistroM515),
            ("M600", BlocoM, RM600, RegistroM600), ("M605", BlocoM, RM605, RegistroM605),
            ("M611", BlocoM, RM611, RegistroM611), ("M615", BlocoM, RM615, RegistroM615),
            ("M620", BlocoM, RM620, RegistroM620), ("M625", BlocoM, RM625, RegistroM625),
            ("M630", BlocoM, RM630, RegistroM630), ("M700", BlocoM, RM700, RegistroM700),
            ("M800", BlocoM, RM800, RegistroM800), ("M810", BlocoM, RM810, RegistroM810),
            ("M990", BlocoM, RM990, RegistroM990),

            // Bloco P
            ("P001", BlocoP, RP001, RegistroP001), ("P010", BlocoP, RP010, RegistroP010),
            ("P100", BlocoP, RP100, RegistroP100), ("P110", BlocoP, RP110, RegistroP110),
            ("P199", BlocoP, RP199, RegistroP199), ("P200", BlocoP, RP200, RegistroP200),
            ("P210", BlocoP, RP210, RegistroP210), ("P990", BlocoP, RP990, RegistroP990),

            // Bloco 9
            ("9001", Bloco9, R9001, Registro9001), ("9900", Bloco9, R9900, Registro9900),
            ("9990", Bloco9, R9990, Registro9990), ("9999", Bloco9, R9999, Registro9999),
        ],
        conditional => [
            // Registros com despacho condicional baseado na quantidade de campos (`fields.len()`).
            ("M210", BlocoM, fields.len() == 15, RM210Antigo, RegistroM210Antigo, RM210, RegistroM210),
            ("M610", BlocoM, fields.len() == 15, RM610Antigo, RegistroM610Antigo, RM610, RegistroM610),
        ]
    )
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//
//
// cargo test -- --help
// cargo test -- --nocapture
// cargo test -- --show-output

/// Run tests with:
/// cargo test -- --show-output parser_tests
#[cfg(test)]
#[path = "tests/parser_tests.rs"]
mod parser_tests;
