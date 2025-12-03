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
                    "[{:?}:Linha nº {:2}] Registro {:?} não suportado. Linha ignorada.",
                    $file_path.display(),
                    $line_number,
                    $registro_str,
                );
                Ok(None) // For unsupported record types, also log a warning and skip.
            }
        }
    };
}

// ============================================================================
// SEÇÃO 3: MACROS PARA IMPLEMENTAÇÃO DE TRAITS
// Redução drástica de boilerplate mapping campos das structs para as Traits.
// ============================================================================

// A macro impl_dopai! é uma "máquina de escrever código". O objetivo dela é automatizar a implementação da
// Trait RegistroPai para várias structs diferentes (como RegistroC100, RegistroD100), evitando que você
// tenha que escrever manualmente <impl RegistroPai for> ... repetidas vezes.

/// Implementa RegistroPai para mapear campos da Struct para a Trait
#[macro_export]
macro_rules! impl_dopai {
    ($struct:ident, { $($trait_fn:ident : $field:ident),* $(,)? }) => {
        impl $crate::info_new::RegistroPai for $struct {
            $( $crate::impl_dopai!(@map $trait_fn, $field); )*
        }
    };
    // Datas e Decimais (Copy types) retornam valor
    // Datas de Emissão e Entrada
    (@map get_dt_emissao, $v:ident) => { fn get_dt_emissao(&self) -> Option<NaiveDate> { self.$v } };
    (@map get_dt_entrada, $v:ident) => { fn get_dt_entrada(&self) -> Option<NaiveDate> { self.$v } };

    // Métodos que retornam referência (&str)
    (@map get_chave, $v:ident) => { fn get_chave(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cod_cta, $v:ident) => { fn get_cod_cta(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cod_item, $v:ident) => { fn get_cod_item(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cod_mod, $v:ident) => { fn get_cod_mod(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cod_ncm, $v:ident) => { fn get_cod_ncm(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cod_part, $v:ident) => { fn get_cod_part(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_num_doc, $v:ident) => { fn get_num_doc(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_valor_bc_icms, $v:ident) => { fn get_valor_bc_icms(&self) -> Option<Decimal> { self.$v } };
    (@map get_valor_icms, $v:ident) => { fn get_valor_icms(&self) -> Option<Decimal> { self.$v } };

    // Safety catch
    (@method $other:ident, $v:ident) => { compile_error!(concat!("Chave desconhecida '", stringify!($other), "' em impl_dopai!")); };
}

/// Implementa RegistroFilho para mapear campos da Struct para a Trait
#[macro_export]
macro_rules! impl_filho {
    ($struct:ident, { $($trait_fn:ident : $field:ident),* $(,)? }) => {
        impl $crate::info_new::RegistroFilho for $struct {
            $( $crate::impl_filho!(@map $trait_fn, $field); )*
        }
    };
    // Datas de Emissão e Entrada
    (@map get_dt_emissao, $v:ident) => { fn get_dt_emissao(&self) -> Option<NaiveDate> { self.$v } };
    (@map get_dt_entrada, $v:ident) => { fn get_dt_entrada(&self) -> Option<NaiveDate> { self.$v } };

    // Identificação por Código
    (@map get_cod_cta, $v:ident) => { fn get_cod_cta(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cod_cred, $v:ident) => { fn get_cod_cred(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cod_item, $v:ident) => { fn get_cod_item(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cod_nat, $v:ident) => { fn get_cod_nat(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cod_part, $v:ident) => { fn get_cod_part(&self) -> Option<&str> { self.$v.as_deref() } };

    // Identificação do Item
    (@map get_num_item, $v:ident) => { fn get_num_item(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_descr_item, $v:ident) => { fn get_descr_item(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_descr_compl, $v:ident) => { fn get_descr_compl(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_info_compl, $v:ident) => { fn get_info_compl(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_part_override, $v:ident) => { fn get_part_override(&self) -> Option<&str> { self.$v.as_deref() } };

    // Classificação Fiscal
    (@map get_cst_pis, $v:ident) => { fn get_cst_pis(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cst_cofins, $v:ident) => { fn get_cst_cofins(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cfop, $v:ident) => { fn get_cfop(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_nat_bc_cred, $v:ident) => { fn get_nat_bc_cred(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_ind_orig_cred, $v:ident) => { fn get_ind_orig_cred(&self) -> Option<&str> { self.$v.as_deref() } };

    // Decimal/Copy
    (@map get_valor_item, $v:ident) => { fn get_valor_item(&self) -> Option<Decimal> { self.$v } };
    (@map get_valor_desc, $v:ident) => { fn get_valor_desc(&self) -> Option<Decimal> { self.$v } };

    // PIS
    (@map get_valor_bc_pis, $v:ident) => { fn get_valor_bc_pis(&self) -> Option<Decimal> { self.$v } };
    (@map get_aliq_pis, $v:ident) => { fn get_aliq_pis(&self) -> Option<Decimal> { self.$v } };
    (@map get_valor_pis, $v:ident) => { fn get_valor_pis(&self) -> Option<Decimal> { self.$v } };

    // COFINS
    (@map get_valor_bc_cofins, $v:ident) => { fn get_valor_bc_cofins(&self) -> Option<Decimal> { self.$v } };
    (@map get_aliq_cofins, $v:ident) => { fn get_aliq_cofins(&self) -> Option<Decimal> { self.$v } };
    (@map get_valor_cofins, $v:ident) => { fn get_valor_cofins(&self) -> Option<Decimal> { self.$v } };

    // Outros Tributos (ICMS, ISS, IPI)
    (@map get_valor_iss, $v:ident) => { fn get_valor_iss(&self) -> Option<Decimal> { self.$v } };
    (@map get_valor_ipi, $v:ident) => { fn get_valor_ipi(&self) -> Option<Decimal> { self.$v } };
    (@map get_valor_icms, $v:ident) => { fn get_valor_icms(&self) -> Option<Decimal> { self.$v } };

    (@map get_aliq_icms, $v:ident) => { fn get_aliq_icms(&self) -> Option<Decimal> { self.$v } };
    (@map get_valor_bc_icms, $v:ident) => { fn get_valor_bc_icms(&self) -> Option<Decimal> { self.$v } };
    (@map get_valor_icms_st, $v:ident) => { fn get_valor_icms_st(&self) -> Option<Decimal> { self.$v } };
    (@map get_valor_bc_icms_st, $v:ident) => { fn get_valor_bc_icms_st(&self) -> Option<Decimal> { self.$v } };

    // Safety catch
    (@method $other:ident, $v:ident) => { compile_error!(concat!("Chave desconhecida '", stringify!($other), "' em impl_filho!")); };
}

// ============================================================================
// SEÇÃO 1: MACROS GLOBAIS DE PROCESSAMENTO
// Reduzem boilerplate nos processadores de bloco (A, C, D, F, etc.)
// ============================================================================

/// Captura o CNPJ do estabelecimento atual (Ex: A010, C010, D010).
/// Atualiza a variável de estado que rastreia o contexto do CNPJ.
#[macro_export]
macro_rules! capture_cnpj {
    ($target:expr, $rec:expr, $ty:ty) => {
        if let Ok(reg) = $rec.downcast_ref::<$ty>() {
            $target = reg.cnpj.as_deref();
        }
    };
}

/// Processa registros "Solo" (sem pai hierárquico no Builder). Ex: F100, C880.
#[macro_export]
macro_rules! process_only_child {
    ($docs:expr, $ctx:expr, $cnpj:expr, $rec:expr, $ty:ty) => {
        if let Ok(filho) = $rec.downcast_ref::<$ty>() {
            $docs.push(DocsBuilder::from_child($ctx, filho, $cnpj).build());
        }
    };
}

/// Processa registros Filhos (vinculados a um Pai). Ex: C170 -> C100.
#[macro_export]
macro_rules! process_child_and_parent {
    ($docs:expr, $ctx:expr, $cnpj:expr, $rec:expr, $ty:ty, $pai:expr) => {
        if let (Ok(filho), Some(p)) = ($rec.downcast_ref::<$ty>(), $pai) {
            $docs.push(DocsBuilder::from_child_and_parent($ctx, filho, Some(p), $cnpj).build());
        }
    };
}

/// Processa registros Filhos com resolução de correlação PIS/COFINS. Ex: C185, D105.
#[macro_export]
macro_rules! process_corr {
    ($docs:expr, $ctx:expr, $cnpj:expr, $mgr:expr, $rec:expr, $ty:ty, $pai:expr) => {
        if let (Ok(filho), Some(p)) = ($rec.downcast_ref::<$ty>(), $pai) {
            $docs.push(
                DocsBuilder::from_child_and_parent($ctx, filho, Some(p), $cnpj)
                    .resolve_pis_correlation($mgr, filho)
                    .build(),
            );
        }
    };
}

/// Armazena dados de PIS no CorrelationManager.
#[macro_export]
macro_rules! store_pis {
    ($mgr:expr, $rec:expr, $ty:ty) => {
        if let Ok(reg) = $rec.downcast_ref::<$ty>() {
            $mgr.store(
                reg.cst_pis.as_ref(),
                reg.vl_item,
                reg.aliq_pis,
                reg.vl_pis,
                reg.get_cfop(),          // Resolve via Trait (campo 'cfop' ou None)
                reg.get_part_override(), // Resolve via Trait (campo 'cnpj_cpf_part' ou None)
            );
        }
    };
}
