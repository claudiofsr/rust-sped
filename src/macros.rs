// ============================================================================
// Macros de Auxílio
// ============================================================================

/// Macro para implementar a SpedRecordTrait em cada registro.
///
/// Esta macro vincula os campos internos da struct (nivel, bloco, registro, line_number)
/// aos métodos do trait e fornece a implementação de as_any, necessária para o
/// funcionamento do downcast no SpedFile.
#[macro_export]
macro_rules! impl_reg_methods {
    ($struct_name:ident) => {
        impl $crate::traits::SpedRecordTrait for $struct_name {
            #[inline]
            fn nivel(&self) -> u16 {
                self.nivel
            }

            #[inline]
            fn line_number(&self) -> usize {
                self.line_number
            }

            #[inline]
            fn registro_name(&self) -> &str {
                &self.registro
            }

            #[inline]
            fn bloco(&self) -> char {
                self.bloco
            }

            // Estas implementações permitem que o SpedRecord (Enum)
            // converta a referência de volta para a struct concreta.
            #[inline]
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            #[inline]
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };
}

// ============================================================================
// Macro de Despacho (Evita repetição de SpedRecord::BlocoX(BlocoX::R...))
// ============================================================================

/// Macro principal de despacho.
#[macro_export]
macro_rules! dispatch_sped_parsers {
    (
        $reg_id:expr, $path:expr, $line:expr, $fields:expr,
        simple => [ $( ($name:literal, $bloco:ident, $registro:ident, $struct:ident) ),* $(,)? ],
        conditional => [ $( ($reg_name:literal, $bloco_name:ident, $cond:expr, $reg_old:ident, $struct_old:ident, $reg_new:ident, $struct_new:ident) ),* $(,)? ]
    ) => {
        match $reg_id {
            $(
                $name => {
                    let parsed = <$struct as $crate::SpedParser>::parse_reg($path, $line, $fields)?;
                    Ok(Some($crate::SpedRecord::$bloco(Box::new($bloco::$registro(parsed.into())))))
                },
            )*
            $(
                $reg_name => {
                    let record = if $cond {
                        let parsed = <$struct_old as $crate::SpedParser>::parse_reg($path, $line, $fields)?;
                        $crate::SpedRecord::$bloco_name(Box::new($bloco_name::$reg_old(parsed.into())))
                    } else {
                        let parsed = <$struct_new as $crate::SpedParser>::parse_reg($path, $line, $fields)?;
                        $crate::SpedRecord::$bloco_name(Box::new($bloco_name::$reg_new(parsed.into())))
                    };
                    Ok(Some(record))
                }
            )*
            _ => {
                log::warn!("[{}:Linha {}] Registro '{}' não suportado.", $path.display(), $line, $reg_id);
                Ok(None)
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
        impl $crate::extractor::RegistroPai for $struct {
            $( $crate::impl_dopai!(@map $trait_fn, $field); )*
        }
    };
    // Datas e Decimais (Copy types) retornam valor
    // Datas de Emissão e Entrada
    (@map get_dt_emissao, $v:ident) => { fn get_dt_emissao(&self) -> Option<NaiveDate> { self.$v } };
    (@map get_dt_entrada, $v:ident) => { fn get_dt_entrada(&self) -> Option<NaiveDate> { self.$v } };

    // Métodos que retornam referência (&str)
    (@map get_cfop, $v:ident) => { fn get_cfop(&self) -> Option<u16> { self.$v } };
    (@map get_chave, $v:ident) => { fn get_chave(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cod_cta, $v:ident) => { fn get_cod_cta(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cod_inf, $v:ident) => { fn get_cod_inf(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cod_item, $v:ident) => { fn get_cod_item(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cod_mod, $v:ident) => { fn get_cod_mod(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cod_ncm, $v:ident) => { fn get_cod_ncm(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cod_part, $v:ident) => { fn get_cod_part(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_num_doc, $v:ident) => { fn get_num_doc(&self) -> Option<usize> { self.$v } };
    (@map get_valor_iss, $v:ident) => { fn get_valor_iss(&self) -> Option<Decimal> { self.$v } };
    (@map get_valor_bc_icms, $v:ident) => { fn get_valor_bc_icms(&self) -> Option<Decimal> { self.$v } };
    (@map get_valor_icms, $v:ident) => { fn get_valor_icms(&self) -> Option<Decimal> { self.$v } };

    // Safety catch
    (@method $other:ident, $v:ident) => { compile_error!(concat!("Chave desconhecida '", stringify!($other), "' em impl_dopai!")); };
}

/// Implementa RegistroFilho para mapear campos da Struct para a Trait
#[macro_export]
macro_rules! impl_filho {
    ($struct:ident, { $($trait_fn:ident : $field:ident),* $(,)? }) => {
        impl $crate::extractor::RegistroFilho for $struct {
            $( $crate::impl_filho!(@map $trait_fn, $field); )*
        }
    };
    // Datas de Emissão e Entrada
    (@map get_dt_emissao, $v:ident) => { fn get_dt_emissao(&self) -> Option<NaiveDate> { self.$v } };
    (@map get_dt_entrada, $v:ident) => { fn get_dt_entrada(&self) -> Option<NaiveDate> { self.$v } };
    (@map get_per_apu_cred, $v:ident) => { fn get_per_apu_cred(&self) -> Option<NaiveDate> { self.$v } };

    // Identificação por Código
    (@map get_cod_cred, $v:ident) => { fn get_cod_cred(&self) -> Option<$crate::CodigoDoCredito> { self.$v } };
    (@map get_cod_item, $v:ident) => { fn get_cod_item(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cod_mod, $v:ident) => { fn get_cod_mod(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cod_nat, $v:ident) => { fn get_cod_nat(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cod_part, $v:ident) => { fn get_cod_part(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_cod_cta, $v:ident) => { fn get_cod_cta(&self) -> Option<&str> { self.$v.as_deref() } };

    // Identificação do Item
    (@map get_num_item, $v:ident) => { fn get_num_item(&self) -> Option<u16> { self.$v } };
    (@map get_descr_item, $v:ident) => { fn get_descr_item(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_descr_compl, $v:ident) => { fn get_descr_compl(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_info_compl, $v:ident) => { fn get_info_compl(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_part_override, $v:ident) => { fn get_part_override(&self) -> Option<&str> { self.$v.as_deref() } };
    (@map get_num_doc, $v:ident) => { fn get_num_doc(&self) -> Option<usize> { self.$v } };

    // Classificação Fiscal
    (@map get_cst_pis, $v:ident) => { fn get_cst_pis(&self) -> Option<$crate::CodigoSituacaoTributaria> { self.$v } };
    (@map get_cst_cofins, $v:ident) => { fn get_cst_cofins(&self) -> Option<$crate::CodigoSituacaoTributaria> { self.$v } };
    (@map get_cfop, $v:ident) => { fn get_cfop(&self) -> Option<u16> { self.$v } };
    (@map get_nat_bc_cred, $v:ident) => { fn get_nat_bc_cred(&self) -> Option<u16> { self.$v } };
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
/// $rec aqui já é a referência para a struct (ex: &RegistroA010)
#[macro_export]
macro_rules! capture_cnpj {
    ($target:expr, $rec:expr) => {
        // r.cnpj é Option<Arc<str>>. Clonar o Arc é custo zero.
        $target = $rec.cnpj.to_arc()
    };
}

/// Processa registros "Solo" (sem pai hierárquico no Builder). Ex: F100, C880.
#[macro_export]
macro_rules! process_only_child {
    ($docs:expr, $ctx:expr, $cnpj:expr, $filho:expr) => {
        // $filho já é a referência tipada (ex: &RegistroF100)
        $docs.push(DocsBuilder::from_child($ctx, $filho, $cnpj.clone()).build())
    };
}

/// Processa registros Filhos (vinculados a um Pai). Ex: C170 -> C100.
#[macro_export]
macro_rules! process_child_and_parent {
    ($docs:expr, $ctx:expr, $cnpj:expr, $filho:expr, $pai:expr) => {
        // $pai é um Option<&RegistroPai> (ex: self.c100)
        // $filho é uma referência direta à struct (ex: &RegistroC170)
        if let Some(p) = $pai {
            $docs.push(
                DocsBuilder::from_child_and_parent($ctx, $filho, Some(p), $cnpj.clone()).build(),
            );
        }
    };
}

/// Processa registros Filhos com resolução de correlação PIS/COFINS. Ex: C185, D105.
#[macro_export]
macro_rules! process_correlations {
    ($docs:expr, $ctx:expr, $cnpj:expr, $mgr:expr, $filho:expr, $pai:expr) => {
        if let Some(p) = $pai {
            $docs.push(
                DocsBuilder::from_child_and_parent($ctx, $filho, Some(p), $cnpj.clone())
                    .resolve_pis_correlation($mgr, $filho)
                    .build(),
            );
        }
    };
}

/// Armazena dados de PIS no CorrelationManager para futura vinculação com COFINS.
#[macro_export]
macro_rules! store_pis {
    ($mgr:expr, $reg:expr) => {
        // $reg é a referência tipada (ex: &RegistroC181 ou &RegistroM210)
        // Tentamos criar a chave de correlação (CST + Valor)
        if let Some(key) = $crate::extractor::CorrelationKey::new($reg.cst_pis, $reg.vl_item) {
            let criteria = $crate::extractor::CorrelationCriteria {
                cfop: $reg.get_cfop(),
                nat_bc_cred: $reg.get_nat_bc_cred(),
                part: $reg.get_part_override(),
                cod_cta: $reg.get_cod_cta(),
                vl_bc: $reg.get_valor_bc_pis(),
            };
            $mgr.store(key, criteria, $reg.aliq_pis, $reg.vl_pis);
        }
    };
}
