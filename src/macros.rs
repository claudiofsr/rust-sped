// Helper macro to reduce boilerplate in the SpedRecordTrait implementation
#[macro_export]
macro_rules! impl_sped_record_trait {
    ($struct_name:ident) => {
        impl $crate::traits::SpedRecordTrait for $struct_name {
            fn nivel(&self) -> u16 {
                self.nivel
            }

            fn bloco(&self) -> char {
                self.bloco
            }

            fn registro_name(&self) -> &str {
                &self.registro
            }

            fn line_number(&self) -> usize {
                self.line_number
            }

            // Implementação mágica para permitir voltar ao tipo original
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };
}

// Helper macro to parse a specific record type
#[macro_export]
macro_rules! parse_registro {
    ($struct_name:ident, $file_path:expr, $line_number:expr, $fields:expr) => {{
        // Chama o método parse_reg do struct específico
        let parsed_record = $struct_name::parse_reg($file_path, $line_number, $fields)?;
        // Encapsula o resultado em SpedRecord::Generic
        Ok(Some($crate::SpedRecord::new_generic(parsed_record)))
    }};
}

/*
Este macro substitue o match seguinte.

    match registro.as_ref() {
        // Bloco 0
        "0000" => parse_registro!(Registro0000, file_path, line_number, &fields),
        "0001" => parse_registro!(Registro0001, file_path, line_number, &fields),

        "F800" => parse_registro!(RegistroF800, file_path, line_number, &fields),
        "F990" => parse_registro!(RegistroF990, file_path, line_number, &fields),
        // Bloco I
        "I001" => parse_registro!(RegistroI001, file_path, line_number, &fields),
        "I010" => parse_registro!(RegistroI010, file_path, line_number, &fields),
        "I100" => parse_registro!(RegistroI100, file_path, line_number, &fields),
        "I199" => parse_registro!(RegistroI199, file_path, line_number, &fields),
        "I200" => parse_registro!(RegistroI200, file_path, line_number, &fields),
        "I299" => parse_registro!(RegistroI299, file_path, line_number, &fields),
        "I300" => parse_registro!(RegistroI300, file_path, line_number, &fields),
        "I399" => parse_registro!(RegistroI399, file_path, line_number, &fields),
        "I990" => parse_registro!(RegistroI990, file_path, line_number, &fields),
        // Bloco M
        "M001" => parse_registro!(RegistroM001, file_path, line_number, &fields),
        "M100" => parse_registro!(RegistroM100, file_path, line_number, &fields),
        "M105" => parse_registro!(RegistroM105, file_path, line_number, &fields),
        "M110" => parse_registro!(RegistroM110, file_path, line_number, &fields),
        "M115" => parse_registro!(RegistroM115, file_path, line_number, &fields),
        "M200" => parse_registro!(RegistroM200, file_path, line_number, &fields),
        "M205" => parse_registro!(RegistroM205, file_path, line_number, &fields),
        "M210" => {
            if fields.len() == 13 {
                parse_registro!(RegistroM210Antigo, file_path, line_number, &fields)
            } else {
                parse_registro!(RegistroM210, file_path, line_number, &fields)
            }
        }
        "M211" => parse_registro!(RegistroM211, file_path, line_number, &fields),
        "M215" => parse_registro!(RegistroM215, file_path, line_number, &fields),
        "M220" => parse_registro!(RegistroM220, file_path, line_number, &fields),

        "1809" => parse_registro!(Registro1809, file_path, line_number, &fields),
        "1900" => parse_registro!(Registro1900, file_path, line_number, &fields),
        "1990" => parse_registro!(Registro1990, file_path, line_number, &fields),

        // Bloco 9
        "9001" => parse_registro!(Registro9001, file_path, line_number, &fields),
        "9900" => parse_registro!(Registro9900, file_path, line_number, &fields),
        "9990" => parse_registro!(Registro9990, file_path, line_number, &fields),
        "9999" => parse_registro!(Registro9999, file_path, line_number, &fields),

        _ => {
            warn!(
                "[{:?}:Linha nº {:2}] Registro '{}' não suportado. Linha ignorada.",
                file_path.display(),
                line_number,
                registro,
            );
            Ok(None) // For unsupported record types, also log a warning and skip.
        }
    }

1. Novas Regras de Delimitadores:
simple => [ $( ... ),* ],:
    Agora, todos os registros que são despachados diretamente para um único struct são listados dentro de um bloco simple => [ ... ].
conditional => [ $( ... ),* ]:
    E todos os registros que têm uma lógica condicional para decidir qual struct usar são listados dentro de um bloco conditional => [ ... ].

2. $(,)? (Trailing Comma): Adicionei o $(,)? para permitir vírgulas opcionais no final de cada lista de repetição.
Isso é uma boa prática e evita erros se você adicionar itens e esquecer uma vírgula.
*/

#[macro_export]
macro_rules! dispatch_sped_parsers {
    (
        $registro_str:expr,
        $file_path:expr,
        $line_number:expr,
        $fields:expr,
        // Envolve os registros simples em um bloco "simple"
        simple => [ $( ($reg_name:literal, $struct_name:ident) ),* $(,)? ],
        // Envolve os registros condicionais em um bloco "conditional"
        conditional => [ $( ($reg_name_conditional:literal, $condition:expr, $struct_cond_true:ident, $struct_cond_false:ident) ),* $(,)? ]
    ) => {
        match $registro_str {
            $(
                $reg_name => $crate::parse_registro!($struct_name, $file_path, $line_number, $fields),
            )*
            $(
                $reg_name_conditional => {
                    if $condition {
                        $crate::parse_registro!($struct_cond_true, $file_path, $line_number, $fields)
                    } else {
                        $crate::parse_registro!($struct_cond_false, $file_path, $line_number, $fields)
                    }
                }
            )*
            _ => {
                warn!(
                    "[{:?}:Linha nº {:2}] Registro '{}' não suportado. Linha ignorada.",
                    $file_path.display(),
                    $line_number,
                    $registro_str,
                );
                Ok(None) // For unsupported record types, also log a warning and skip.
            }
        }
    };
}

#[macro_export]
macro_rules! dispatch_records {
    (
        $target_name:expr, $inner:expr, $self:expr,
        // Lista de registros padrão: "NOME" => Struct => funcao
        simple => [
            $( ($reg_str:literal, $struct_name:ty, $fn_name:ident) ),* $(,)?
        ],
        // Casos especiais que exigem argumentos extras ou lógica diferente
        conditional => {
            $( $pat:pat => $body:expr ),* $(,)?
        }
    ) => {
        match $target_name {
            $(
                $reg_str => {
                    let reg_typed = $inner.as_any()
                        .downcast_ref::<$struct_name>()
                        .ok_or_else(|| EFDError::RecordCastError($reg_str.to_string()))?;
                    $self.$fn_name(reg_typed)
                }
            )*
            // Injeta os casos especiais
            $( $pat => $body, )*

            _ => Ok(()),
        }
    };
}
