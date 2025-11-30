use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::collections::HashMap;

use crate::{
    ALIQ_BASICA_COF, ALIQ_BASICA_PIS, DocsFiscais, FloatExt, IndicadorOrigem, MesesDoAno,
    SpedContext, SpedFile, SpedRecord, SpedRecordTrait, TipoDeCredito, TipoDeRateio, TipoOperacao,
    blocos::*, cred_presumido, obter_cod_da_natureza_da_bc,
};

// ============================================================================
// SEÇÃO 1: EXTENSIONS E TRAITS
// Abstrações para normalizar o acesso aos dados dos registros de forma polimórfica.
// Isso permite que o `DocsBuilder` não precise saber o tipo exato da struct.
// ============================================================================

// ----------------------------------------------------------------------------
// TRAITS
// ----------------------------------------------------------------------------

/// Extension para facilitar a conversão de `Option<Decimal>` para `f64`.
trait DecimalExt {
    fn to_f64_opt(&self) -> Option<f64>;
    fn abs_f64(&self) -> f64;
}

impl DecimalExt for Option<Decimal> {
    fn to_f64_opt(&self) -> Option<f64> {
        self.as_ref().and_then(|d| d.to_f64())
    }
    fn abs_f64(&self) -> f64 {
        self.map(|d| d.abs().to_f64().unwrap_or_default())
            .unwrap_or_default()
    }
}

/// Extension para facilitar o parsing de `Option<String>`.
trait StringParser {
    fn parse_opt<T: std::str::FromStr>(&self) -> Option<T>;
}

impl<U> StringParser for Option<U>
where
    U: AsRef<str>,
{
    fn parse_opt<T: std::str::FromStr>(&self) -> Option<T> {
        self.as_ref().and_then(|u| u.as_ref().parse().ok())
    }
}

// ----------------------------------------------------------------------------
// CATEGORIZAÇÃO DE REGISTROS
// ----------------------------------------------------------------------------

/// Representa registros autônomos que contêm informações fiscais diretas.
/// Mapeia campos variados (VL_OPER, VL_REC, etc.) para getters padronizados.
pub trait RegistroGeral: SpedRecordTrait {
    /// Data da emissão do documento fiscal
    fn get_data_emissao(&self) -> Option<NaiveDate> {
        None
    }
    /// Data de Execução / Conclusão do Serviço
    /// Data da Entrada / Aquisição / Execução ou da Saída / Prestação / Conclusão
    fn get_data_entrada(&self) -> Option<NaiveDate> {
        None
    }
    fn get_cnpj(&self) -> Option<String> {
        None
    }
    fn get_valor_geral(&self) -> Option<Decimal> {
        None
    }
    fn get_descr_geral(&self) -> Option<String> {
        None
    }
    fn get_part_geral(&self) -> Option<String> {
        None
    }
    fn get_cod_cred(&self) -> Option<String> {
        None
    }
    fn get_info_compl(&self) -> Option<String> {
        None
    }
}

/// Representa um registro "Pai" (Cabeçalho) que fornece contexto para os "Filhos".
///
/// Ex: C100, D100. Fornece dados contextuais para os filhos.
pub trait RegistroPai {
    /// Data da emissão do documento fiscal
    fn get_data_emissao(&self) -> Option<NaiveDate> {
        None
    }
    /// Data de Execução / Conclusão do Serviço
    /// Data da Entrada / Aquisição / Execução ou da Saída / Prestação / Conclusão
    fn get_data_entrada(&self) -> Option<NaiveDate> {
        None
    }
    fn get_chave(&self) -> Option<String> {
        None
    }
    fn get_participante_cod(&self) -> Option<String> {
        None
    }
    fn get_modelo_doc_fiscal(&self) -> Option<String> {
        None
    }
    fn get_num_doc(&self) -> Option<String> {
        None
    }
}

// Implementação default para Unit type (sem pai)
impl RegistroPai for () {}

/// Representa um registro "Filho" (Item/Detalhe) contendo valores tributários.
///
/// Ex: C170, D105.
pub trait RegistroFilho: SpedRecordTrait {
    fn get_valor_item(&self) -> Option<Decimal> {
        None
    }
    // Classificações
    fn get_cst_pis(&self) -> Option<String> {
        None
    }
    fn get_cst_cofins(&self) -> Option<String> {
        None
    }
    fn get_cfop(&self) -> Option<String> {
        None
    }
    fn get_cod_item(&self) -> Option<String> {
        None
    }
    fn get_cod_nat(&self) -> Option<String> {
        None
    }
    fn get_cod_cta(&self) -> Option<String> {
        None
    }
    // Valores PIS
    fn get_aliq_pis(&self) -> Option<Decimal> {
        None
    }
    fn get_val_pis(&self) -> Option<Decimal> {
        None
    }
    fn get_bc_pis(&self) -> Option<Decimal> {
        None
    }
    // Valores COFINS
    fn get_aliq_cofins(&self) -> Option<Decimal> {
        None
    }
    fn get_val_cofins(&self) -> Option<Decimal> {
        None
    }
    fn get_bc_cofins(&self) -> Option<Decimal> {
        None
    }
    // Valores ICMS
    fn get_bc_icms(&self) -> Option<Decimal> {
        None
    }
    fn get_aliq_icms(&self) -> Option<Decimal> {
        None
    }
    fn get_val_icms(&self) -> Option<Decimal> {
        None
    }
    // Outros
    fn get_descr_compl(&self) -> Option<String> {
        None
    }
    fn get_participante_override(&self) -> Option<String> {
        None
    }
}

// ============================================================================
// SEÇÃO 2: MACROS (BOILERPLATE REDUCTION)
// Redução de boilerplate para mapear campos das structs para as Traits.
// ============================================================================

// A macro impl_dopai! é uma "máquina de escrever código". O objetivo dela é automatizar a implementação da
// Trait RegistroPai para várias structs diferentes (como RegistroC100, RegistroD100), evitando que você
// tenha que escrever manualmente impl RegistroPai for ... repetidas vezes.

macro_rules! impl_geral {
    ($struct_name:ident, { $($key:ident : $value:ident),* $(,)? }) => {
        impl RegistroGeral for $struct_name {
            $( impl_geral!(@method $key, $value); )*
        }
    };
    (@method cnpj, $v:ident) => { fn get_cnpj(&self) -> Option<String> { self.$v.clone() } };
    (@method dt_emissao, $v:ident) => { fn get_data_emissao(&self) -> Option<NaiveDate> { self.$v } };
    (@method dt_entrada, $v:ident) => { fn get_data_entrada(&self) -> Option<NaiveDate> { self.$v } };
    (@method valor, $v:ident) => { fn get_valor_geral(&self) -> Option<Decimal> { self.$v } };
    (@method descr, $v:ident) => { fn get_descr_geral(&self) -> Option<String> { self.$v.clone() } };
    (@method part, $v:ident) => { fn get_part_geral(&self) -> Option<String> { self.$v.clone() } };
    (@method cred, $v:ident) => { fn get_cod_cred(&self) -> Option<String> { self.$v.clone() } };
    (@method info, $v:ident) => { fn get_info_compl(&self) -> Option<String> { self.$v.clone() } };
    (@method $other:ident, $v:ident) => { compile_error!(concat!("Chave desconhecida '", stringify!($other), "' em impl_geral!")); };
}

macro_rules! impl_dopai {
    ($struct_name:ident, { $($key:ident : $value:ident),* $(,)? }) => {
        impl RegistroPai for $struct_name {
            $( impl_dopai!(@method $key, $value); )*
        }
    };
    (@method dt_emissao, $v:ident) => { fn get_data_emissao(&self) -> Option<NaiveDate> { self.$v } };
    (@method dt_entrada, $v:ident) => { fn get_data_entrada(&self) -> Option<NaiveDate> { self.$v } };
    (@method chave, $v:ident) => { fn get_chave(&self) -> Option<String> { self.$v.clone() } };
    (@method part, $v:ident) => { fn get_participante_cod(&self) -> Option<String> { self.$v.clone() } };
    (@method modelo, $v:ident) => { fn get_modelo_doc_fiscal(&self) -> Option<String> { self.$v.clone() } };
    (@method num, $v:ident) => { fn get_num_doc(&self) -> Option<String> { self.$v.clone() } };
    (@method $other:ident, $v:ident) => { compile_error!(concat!("Chave desconhecida '", stringify!($other), "' em impl_dopai!")); };
}

macro_rules! impl_filho {
    ($struct_name:ident, { $($key:ident : $value:ident),* $(,)? }) => {
        impl RegistroFilho for $struct_name {
            $( impl_filho!(@method $key, $value); )*
        }
    };
    // Decimals (Copy)
    (@method val_item, $v:ident) => { fn get_valor_item(&self) -> Option<Decimal> { self.$v } };
    (@method aliq_pis, $v:ident) => { fn get_aliq_pis(&self) -> Option<Decimal> { self.$v } };
    (@method val_pis, $v:ident) => { fn get_val_pis(&self) -> Option<Decimal> { self.$v } };
    (@method vl_bc_pis, $v:ident) => { fn get_bc_pis(&self) -> Option<Decimal> { self.$v } };
    (@method aliq_cof, $v:ident) => { fn get_aliq_cofins(&self) -> Option<Decimal> { self.$v } };
    (@method val_cof, $v:ident) => { fn get_val_cofins(&self) -> Option<Decimal> { self.$v } };
    (@method bc_cof, $v:ident) => { fn get_bc_cofins(&self) -> Option<Decimal> { self.$v } };
    (@method bc_icms, $v:ident) => { fn get_bc_icms(&self) -> Option<Decimal> { self.$v } };
    (@method aliq_icms, $v:ident) => { fn get_aliq_icms(&self) -> Option<Decimal> { self.$v } };
    (@method val_icms, $v:ident) => { fn get_val_icms(&self) -> Option<Decimal> { self.$v } };
    // Strings (Clone)
    (@method cst_pis, $v:ident) => { fn get_cst_pis(&self) -> Option<String> { self.$v.clone() } };
    (@method cst_cof, $v:ident) => { fn get_cst_cofins(&self) -> Option<String> { self.$v.clone() } };
    (@method cfop, $v:ident) => { fn get_cfop(&self) -> Option<String> { self.$v.clone() } };
    (@method cod_item, $v:ident) => { fn get_cod_item(&self) -> Option<String> { self.$v.clone() } };
    (@method cod_nat, $v:ident) => { fn get_cod_nat(&self) -> Option<String> { self.$v.clone() } };
    (@method cod_cta, $v:ident) => { fn get_cod_cta(&self) -> Option<String> { self.$v.clone() } };
    (@method descr, $v:ident) => { fn get_descr_compl(&self) -> Option<String> { self.$v.clone() } };
    (@method part_over, $v:ident) => { fn get_participante_override(&self) -> Option<String> { self.$v.clone() } };
    (@method $other:ident, $v:ident) => { compile_error!(concat!("Chave desconhecida '", stringify!($other), "' em impl_filho!")); };
}

// ============================================================================
// SEÇÃO 3: BINDINGS (Mapeamento Struct -> Traits)
// ============================================================================

// Bloco A
// impl_geral!(RegistroA010, { cnpj: cnpj });
impl_dopai!(RegistroA100, { dt_emissao: dt_doc, dt_entrada: dt_exe_serv, chave: chv_nfse, part: cod_part });
impl_filho!(RegistroA170, { val_item: vl_item, cst_pis: cst_pis, cst_cof: cst_cofins, aliq_pis: aliq_pis, val_pis: vl_pis, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins, descr: descr_compl });

// Bloco C
// impl_geral!(RegistroC010, { cnpj: cnpj });
impl_dopai!(RegistroC100, { dt_entrada: dt_e_s, dt_emissao: dt_doc, chave: chv_nfe, part: cod_part, modelo: cod_mod, num: num_doc });
impl_filho!(RegistroC170, { val_item: vl_item, cst_pis: cst_pis, cst_cof: cst_cofins, cfop: cfop, aliq_pis: aliq_pis, val_pis: vl_pis, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins, cod_item: cod_item, cod_nat: cod_nat, cod_cta: cod_cta, bc_icms: vl_bc_icms, aliq_icms: aliq_icms, val_icms: vl_icms });
impl_filho!(RegistroC175, { val_item: vl_opr, cst_pis: cst_pis, cst_cof: cst_cofins, cfop: cfop, aliq_pis: aliq_pis, val_pis: vl_pis, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins });
impl_dopai!(RegistroC180, { dt_emissao: dt_doc_ini, modelo: cod_mod });
impl_filho!(RegistroC181, { val_item: vl_item, cst_pis: cst_pis, aliq_pis: aliq_pis, val_pis: vl_pis, cfop: cfop, cod_item: cod_cta });
impl_filho!(RegistroC185, { val_item: vl_item, cst_cof: cst_cofins, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins, cfop: cfop, cod_item: cod_cta });
impl_dopai!(RegistroC190, { dt_emissao: dt_ref_ini });
impl_filho!(RegistroC191, { val_item: vl_item, cst_pis: cst_pis, aliq_pis: aliq_pis, val_pis: vl_pis, cfop: cfop, part_over: cnpj_cpf_part });
impl_filho!(RegistroC195, { val_item: vl_item, cst_cof: cst_cofins, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins, cfop: cfop, part_over: cnpj_cpf_part });
impl_dopai!(RegistroC380, { dt_emissao: dt_doc_ini });
impl_filho!(RegistroC381, { val_item: vl_item, cst_pis: cst_pis, aliq_pis: aliq_pis, val_pis: vl_pis });
impl_filho!(RegistroC385, { val_item: vl_item, cst_cof: cst_cofins, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins });
impl_dopai!(RegistroC395, { dt_emissao: dt_doc, part: cod_part });
impl_filho!(RegistroC396, { val_item: vl_item, cst_pis: cst_pis, cst_cof: cst_cofins, aliq_pis: aliq_pis, val_pis: vl_pis, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins });
impl_dopai!(RegistroC400, {});
impl_dopai!(RegistroC405, { dt_emissao: dt_doc });
impl_filho!(RegistroC481, { val_item: vl_item, cst_pis: cst_pis, aliq_pis: aliq_pis, val_pis: vl_pis });
impl_filho!(RegistroC485, { val_item: vl_item, cst_cof: cst_cofins, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins });
impl_dopai!(RegistroC490, { dt_emissao: dt_doc_ini });
impl_filho!(RegistroC491, { val_item: vl_item, cst_pis: cst_pis, aliq_pis: aliq_pis, val_pis: vl_pis });
impl_filho!(RegistroC495, { val_item: vl_item, cst_cof: cst_cofins, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins, cfop: cfop });
impl_dopai!(RegistroC500, { dt_emissao: dt_doc, chave: chv_doce, num: num_doc, part: cod_part, modelo: cod_mod });
impl_filho!(RegistroC501, { val_item: vl_item, cst_pis: cst_pis, aliq_pis: aliq_pis, val_pis: vl_pis });
impl_filho!(RegistroC505, { val_item: vl_item, cst_cof: cst_cofins, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins });
impl_dopai!(RegistroC600, { dt_emissao: dt_doc, part: cod_mun, modelo: cod_mod });
impl_filho!(RegistroC601, { val_item: vl_item, cst_pis: cst_pis, aliq_pis: aliq_pis, val_pis: vl_pis });
impl_filho!(RegistroC605, { val_item: vl_item, cst_cof: cst_cofins, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins });
impl_dopai!(RegistroC860, { dt_emissao: dt_doc });
impl_filho!(RegistroC870, { val_item: vl_item, cst_pis: cst_pis, cst_cof: cst_cofins, cfop: cfop, aliq_pis: aliq_pis, val_pis: vl_pis, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins, cod_item: cod_item });
impl_filho!(RegistroC880, { val_item: vl_item, cst_pis: cst_pis, cst_cof: cst_cofins, cfop: cfop, val_pis: vl_pis, val_cof: vl_cofins });

// Bloco D
impl_geral!(RegistroD010, { cnpj: cnpj });
impl_dopai!(RegistroD100, { dt_emissao: dt_a_p, chave: chv_cte, part: cod_part, modelo: cod_mod });
impl_filho!(RegistroD101, { val_item: vl_item, cst_pis: cst_pis, aliq_pis: aliq_pis, val_pis: vl_pis });
impl_filho!(RegistroD105, { val_item: vl_item, cst_cof: cst_cofins, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins });
impl_dopai!(RegistroD200, { dt_emissao: dt_ref, modelo: cod_mod });
impl_filho!(RegistroD201, { val_item: vl_item, cst_pis: cst_pis, aliq_pis: aliq_pis, val_pis: vl_pis });
impl_filho!(RegistroD205, { val_item: vl_item, cst_cof: cst_cofins, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins });
impl_filho!(RegistroD300, { val_item: vl_doc, cst_pis: cst_pis, cst_cof: cst_cofins, aliq_pis: aliq_pis, val_pis: vl_pis, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins });
impl_filho!(RegistroD350, { val_item: vl_brt, cst_pis: cst_pis, cst_cof: cst_cofins, aliq_pis: aliq_pis, val_pis: vl_pis, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins });
impl_dopai!(RegistroD500, { dt_emissao: dt_a_p, part: cod_part, modelo: cod_mod });
impl_filho!(RegistroD501, { val_item: vl_item, cst_pis: cst_pis, aliq_pis: aliq_pis, val_pis: vl_pis });
impl_filho!(RegistroD505, { val_item: vl_item, cst_cof: cst_cofins, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins });
impl_dopai!(RegistroD600, { dt_emissao: dt_doc_ini, modelo: cod_mod });
impl_filho!(RegistroD601, { val_item: vl_item, cst_pis: cst_pis, aliq_pis: aliq_pis, val_pis: vl_pis });
impl_filho!(RegistroD605, { val_item: vl_item, cst_cof: cst_cofins, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins });

// Bloco F (Registros Autônomos e Híbridos)
impl_geral!(RegistroF010, { cnpj: cnpj });
// F100: DT_OPER->data, VL_OPER->valor, DESC_DOC_OPER->descr
impl_geral!(RegistroF100, { dt_emissao: dt_oper, valor: vl_oper, descr: desc_doc_oper, part: cod_part });
impl_filho!(RegistroF100, { val_item: vl_oper, cst_pis: cst_pis, cst_cof: cst_cofins, aliq_pis: aliq_pis, val_pis: vl_pis, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins, cod_item: cod_item, cod_nat: nat_bc_cred, descr: desc_doc_oper, part_over: cod_part });
// F120: VL_OPER_DEP->valor, DESC_BEM_IMOB->descr
impl_geral!(RegistroF120, { valor: vl_oper_dep, descr: desc_bem_imob });
impl_filho!(RegistroF120, { val_item: vl_oper_dep, cst_pis: cst_pis, cst_cof: cst_cofins, aliq_pis: aliq_pis, val_pis: vl_pis, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins, descr: desc_bem_imob });
// F130: VL_BC_COFINS->valor (Conforme dispatch_table: Custo Aquisicao), DESC_BEM_IMOB->descr
impl_geral!(RegistroF130, { valor: vl_bc_cofins, descr: desc_bem_imob });
impl_filho!(RegistroF130, { val_item: vl_bc_cofins, cst_pis: cst_pis, cst_cof: cst_cofins, aliq_pis: aliq_pis, val_pis: vl_pis, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins, descr: desc_bem_imob });
// F150: VL_TOT_EST->valor, DESC_EST->descr
impl_geral!(RegistroF150, { valor: vl_tot_est, descr: desc_est });
impl_filho!(RegistroF150, { val_item: vl_tot_est, cst_pis: cst_pis, cst_cof: cst_cofins, aliq_pis: aliq_pis, val_pis: vl_cred_pis, aliq_cof: aliq_cofins, val_cof: vl_cred_cofins, bc_cof: vl_bc_est, descr: desc_est });
// F200: VL_TOT_REC->valor, INF_COMP->descr, CPF_CNPJ_ADQU->part
impl_geral!(RegistroF200, { dt_emissao: dt_oper, valor: vl_tot_rec, descr: inf_comp, part: cpf_cnpj_adqu });
impl_filho!(RegistroF200, { val_item: vl_tot_rec, cst_pis: cst_pis, cst_cof: cst_cofins, aliq_pis: aliq_pis, val_pis: vl_pis, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins });
// F205/F210/F500/F510/F550/F560
impl_geral!(RegistroF205, { valor: vl_cus_inc_per_esc });
impl_geral!(RegistroF210, { valor: vl_cus_orc });
impl_geral!(RegistroF500, { valor: vl_rec_caixa, descr: info_compl });
impl_geral!(RegistroF510, { valor: vl_rec_caixa, descr: info_compl });
impl_geral!(RegistroF550, { valor: vl_rec_comp, descr: info_compl });
impl_geral!(RegistroF560, { valor: vl_rec_comp, descr: info_compl });

// Bloco I
impl_geral!(RegistroI010, { cnpj: cnpj });
impl_geral!(RegistroI100, { valor: vl_rec, descr: info_compl });
impl_filho!(RegistroI100, { val_item: vl_rec, cst_pis: cst_pis_cofins, cst_cof: cst_pis_cofins, aliq_pis: aliq_pis, val_pis: vl_pis, aliq_cof: aliq_cofins, val_cof: vl_cofins, bc_cof: vl_bc_cofins });

// Bloco M
impl_filho!(RegistroM505, { cst_cof: cst_cofins, bc_cof: vl_bc_cofins });

// Bloco 1 (Controle)
impl_geral!(Registro1100, { cred: cod_cred });
impl_geral!(Registro1500, { cred: cod_cred });

// ============================================================================
// SEÇÃO 4: CORRELATION MANAGER
// Implementa lógica rigorosa de "dispatch_table.rs" (Weak vs Strong Key)
// Gerencia a lógica de correlação PIS <-> COFINS com cache de alta performance
// ============================================================================

// Type aliases para satisfazer Clippy (type_complexity) e melhorar legibilidade
type WeakKey = (String, Decimal);
type StrongKey = (String, Decimal, Option<String>, Option<String>);
type PisData = (f64, f64);

#[derive(Default)]
struct CorrelationManager {
    // Chave Fraca: (CST, Valor)
    // Armazena sempre que houver dados mínimos.
    // Usada como fallback se a chave forte não for encontrada.
    weak_cache: HashMap<WeakKey, PisData>,

    // Chave Forte: (CST, Valor, Option<CFOP>, Option<Part>)
    // Armazena quando houver contexto extra. Option permite chaves parciais.
    // Preferida pois inclui contexto de CFOP e Participante.
    strong_cache: HashMap<StrongKey, PisData>,
}

impl CorrelationManager {
    fn clear(&mut self) {
        self.weak_cache.clear();
        self.strong_cache.clear();
    }

    /// Armazena dados de PIS.
    /// Usa `match` para extração segura e limpa dos 4 campos obrigatórios.
    fn store(
        &mut self,
        cst: Option<&String>,
        val_item: Option<Decimal>,
        aliq: Option<Decimal>,
        val: Option<Decimal>,
        cfop: Option<&str>,
        part: Option<&str>,
    ) {
        // 1. Verifica se os dados mandatórios existem
        // Nada a fazer se faltar dados básicos
        if let (Some(c), Some(v_item), Some(a), Some(v)) = (cst, val_item, aliq, val) {
            let data: PisData = (
                a.to_f64().unwrap_or_default(),
                v.to_f64().unwrap_or_default(),
            );

            // 2. Armazena Cache Fraco (CST + Valor)
            self.weak_cache.insert((c.clone(), v_item), data);

            // 3. Armazena Cache Forte (Se houver contexto de CFOP ou Participante)
            let cf = cfop.filter(|s| !s.is_empty()).map(ToString::to_string);
            let pt = part.filter(|s| !s.is_empty()).map(ToString::to_string);

            if cf.is_some() || pt.is_some() {
                self.strong_cache.insert((c.clone(), v_item, cf, pt), data);
            }
        }
    }

    /// Resolve PIS baseado em dados de COFINS.
    /// Prioridade: Forte (com contexto) -> Fraca (apenas valores).
    fn resolve(
        &self,
        cst: Option<&str>,
        val_item: Option<Decimal>,
        cfop: Option<&str>,
        part: Option<&str>,
    ) -> Option<PisData> {
        let (c, v) = cst.zip(val_item)?;

        // Normaliza contexto de busca
        let cf = cfop.filter(|s| !s.is_empty()).map(ToString::to_string);
        let pt = part.filter(|s| !s.is_empty()).map(ToString::to_string);

        // 1. Tenta Chave Forte (Se houver contexto de busca)
        if cf.is_some() || pt.is_some() {
            let strong_key = (c.to_string(), v, cf, pt);
            if let Some(res) = self.strong_cache.get(&strong_key) {
                return Some(*res);
            }
        }

        // 2. Fallback: Chave Fraca
        let weak_key = (c.to_string(), v);
        self.weak_cache.get(&weak_key).copied()
    }
}

// ============================================================================
// SEÇÃO 5: BUILDER
// Padrão Builder para construção de DocsFiscais, aplicando regras de negócio
// e recuperando dados do Contexto Global.
// ============================================================================

struct DocsBuilder<'a> {
    doc: DocsFiscais,
    ctx: &'a SpedContext,
}

impl<'a> DocsBuilder<'a> {
    /// Inicializa Builder básico
    fn new(
        ctx: &'a SpedContext,
        registro: &str,
        line_num: usize,
        current_cnpj: Option<&String>,
    ) -> Self {
        let mut doc = DocsFiscais {
            linhas: 1,
            arquivo_efd: ctx.path.display().to_string(),
            num_linha_efd: Some(line_num),
            // Usa o CNPJ do contexto local (Blc C010/D010) ou o global do arquivo
            estabelecimento_cnpj: current_cnpj
                .cloned()
                .unwrap_or(ctx.estabelecimento_cnpj.clone()),
            estabelecimento_nome: ctx.estabelecimento_nome.clone(),
            periodo_de_apuracao: ctx.periodo_de_apuracao,
            registro: registro.to_string(),
            ..Default::default()
        };

        if let Some(d) = ctx.periodo_de_apuracao {
            doc.ano = Some(d.year());
            doc.mes = MesesDoAno::try_from(d.month()).ok();
            doc.trimestre = Some((d.month() - 1) / 3 + 1);
        }
        Self { doc, ctx }
    }

    /// Constrói a partir de RegistroGeral (Registros autônomos ou sem pai explícito),
    /// extraindo dados comuns.
    ///
    /// Usa ?Sized para permitir Trait Objects (dyn RegistroGeral).
    fn from_geral<G>(ctx: &'a SpedContext, reg: &G, current_cnpj: Option<&String>) -> Self
    where
        G: RegistroGeral + ?Sized,
    {
        // Se o registro tiver um CNPJ próprio (Ex: F010), ele sobrescreve o atual
        let reg_cnpj = reg.get_cnpj();
        let effective_cnpj = reg_cnpj.as_ref().or(current_cnpj);

        let mut builder = Self::new(ctx, reg.registro_name(), reg.line_number(), effective_cnpj);

        // Mapeamento usando os Getters da Trait RegistroGeral
        builder.doc.data_emissao = reg.get_data_emissao();
        builder.doc.data_entrada = reg.get_data_entrada();
        builder.doc.valor_item = reg.get_valor_geral().to_f64_opt();
        builder.doc.descr_item = reg.get_descr_geral().unwrap_or_default();
        builder = builder.with_participant(&reg.get_part_geral());

        if let Some(info) = reg.get_info_compl() {
            builder.doc.complementar = info;
        }

        builder
    }

    /// Cria doc fiscal combinando Pai e Filho (Hierarquia SPED)
    fn from_child_and_parent<F, P>(
        ctx: &'a SpedContext,
        filho: &F,
        pai: Option<&P>,
        current_cnpj: Option<&String>,
    ) -> Self
    where
        F: RegistroFilho + ?Sized,
        P: RegistroPai + ?Sized,
    {
        let mut builder = Self::new(
            ctx,
            filho.registro_name(),
            filho.line_number(),
            current_cnpj,
        );

        // 1. Processar dados do Pai (Informacoes comuns a todos os filhos)
        if let Some(p) = pai {
            builder.doc.data_emissao = p.get_data_emissao();
            builder.doc.data_entrada = p.get_data_entrada();
            if let Some(k) = p.get_chave() {
                builder.doc.chave_doc = k;
            }
            if let Some(m) = p.get_modelo_doc_fiscal() {
                builder.doc.modelo_doc_fiscal = m;
            }
            builder.doc.num_doc = p.get_num_doc().parse_opt();
            builder = builder.with_participant(&p.get_participante_cod());
        }

        // 2. Processar dados do Filho (Item)
        // Participante do filho tem precedência sobre o pai (Ex: C191, F100)
        if let Some(part_over) = filho.get_participante_override() {
            builder = builder.with_participant(&Some(part_over));
        }

        builder.doc.valor_item = filho.get_valor_item().to_f64_opt();
        builder.doc.cst = filho.get_cst_cofins().parse_opt();
        builder.doc.cfop = filho.get_cfop().parse_opt();

        // PIS explícitos
        // builder.doc.valor_bc_pis = filho.get_bc_pis().to_f64_opt();
        builder.doc.aliq_pis = filho.get_aliq_pis().to_f64_opt();
        builder.doc.valor_pis = filho.get_val_pis().to_f64_opt();

        // COFINS explícitos
        builder.doc.valor_bc = filho.get_bc_cofins().to_f64_opt();
        builder.doc.aliq_cofins = filho.get_aliq_cofins().to_f64_opt();
        builder.doc.valor_cofins = filho.get_val_cofins().to_f64_opt();

        // ICMS & Detalhes
        builder.doc.valor_bc_icms = filho.get_bc_icms().to_f64_opt();
        builder.doc.aliq_icms = filho.get_aliq_icms().to_f64_opt();
        builder.doc.valor_icms = filho.get_val_icms().to_f64_opt();

        builder = builder.with_item_details(&filho.get_cod_item());

        if let Some(desc) = filho.get_descr_compl() {
            builder.doc.descr_item = desc;
        }

        // Lookups de Contexto
        if let Some(cod) = filho.get_cod_nat() {
            builder.doc.nat_operacao = ctx.nat_operacao.get(&cod).cloned().unwrap_or_default();
        }
        if let Some(cod) = filho.get_cod_cta() {
            builder.doc.nome_da_conta = ctx
                .contabil
                .get(&cod)
                .and_then(|h| h.get("NOME_CTA"))
                .cloned()
                .unwrap_or_default();
        }

        builder
    }

    /// Atalho para registros que são Filhos mas tratados como únicos (F200, C880)
    fn from_child<F>(ctx: &'a SpedContext, reg: &F, current_cnpj: Option<&String>) -> Self
    where
        F: RegistroFilho + ?Sized,
    {
        Self::from_child_and_parent(ctx, reg, None::<&()>, current_cnpj)
    }

    // --- Setters e Lógica ---

    fn with_participant(mut self, cod_part: &Option<String>) -> Self {
        if let Some(cod) = cod_part.as_deref().filter(|s| !s.is_empty()) {
            if let Some(hash) = self.ctx.participantes.get(cod) {
                self.doc.participante_cnpj = hash.get("CNPJ").cloned().unwrap_or_default();
                self.doc.participante_cpf = hash.get("CPF").cloned().unwrap_or_default();
                self.doc.participante_nome = hash.get("NOME").cloned().unwrap_or_default();
            } else if cod.len() == 14 {
                self.doc.participante_cnpj = cod.to_string();
                self.doc.participante_nome = self.ctx.obter_nome_participante(Some(cod), None);
            } else if cod.len() == 11 {
                self.doc.participante_cpf = cod.to_string();
                self.doc.participante_nome = self.ctx.obter_nome_participante(None, Some(cod));
            }
        }
        self
    }

    fn with_item_details(mut self, cod_item: &Option<String>) -> Self {
        if let Some(info) = cod_item.as_ref().and_then(|c| self.ctx.produtos.get(c)) {
            self.doc.cod_ncm = info.get("COD_NCM").cloned().unwrap_or_default();
            self.doc.tipo_item = info.get("TIPO_ITEM").cloned().unwrap_or_default();
            self.doc.descr_item = info.get("DESCR_ITEM").cloned().unwrap_or_default();
        }
        self
    }

    /// Tenta preencher dados de PIS usando o CorrelationManager (Cofins -> Pis).
    /// Replica a lógica de tentar Strong Key primeiro, depois Weak Key.    
    fn resolve_pis_correlation<F, P>(
        mut self,
        manager: &CorrelationManager,
        filho: &F,
        pai: Option<&P>,
    ) -> Self
    where
        F: RegistroFilho + ?Sized,
        P: RegistroPai + ?Sized,
    {
        // Se já tem PIS, não precisa correlacionar
        let valor_pis_eh_positivo = self.doc.valor_pis.is_some_and(|p| p.eh_maior_que_zero());
        let aliq_pis_eh_positivo = self.doc.aliq_pis.is_some_and(|p| p.eh_maior_que_zero());
        if valor_pis_eh_positivo && aliq_pis_eh_positivo {
            return self;
        }

        let part = filho
            .get_participante_override()
            .or_else(|| pai.and_then(|p| p.get_participante_cod()));

        if let Some((aliq, val)) = manager.resolve(
            filho.get_cst_cofins().as_deref(),
            filho.get_valor_item(),
            filho.get_cfop().as_deref(),
            part.as_deref(),
        ) {
            self.doc.aliq_pis = Some(aliq);
            self.doc.valor_pis = Some(val);
        } else if let Some(aliq_cofins) = self.doc.aliq_cofins {
            // Fallback heurístico simples se não achar no cache
            if aliq_cofins.eh_igual(ALIQ_BASICA_COF) {
                self.doc.aliq_pis = Some(1.65);
            } else if aliq_cofins.eh_igual(3.0) {
                self.doc.aliq_pis = Some(0.65);
            }
        }
        self
    }

    /// Build DocsFiscais
    fn build(mut self) -> DocsFiscais {
        // --- Regras de Negócio Finais (Derived Fields) ---

        // 1. Tipo de Operação baseado em CST
        if self.doc.tipo_de_operacao.is_none() {
            self.doc.tipo_de_operacao = match self.doc.cst {
                Some(1..=49) => Some(TipoOperacao::Saida),
                Some(50..=99) => Some(TipoOperacao::Entrada),
                _ => None,
            };
        }

        // 2. Natureza BC
        if self.doc.natureza_bc.is_none() {
            self.doc.natureza_bc = obter_cod_da_natureza_da_bc(&self.doc.cfop, self.doc.cst);
        }

        // 3. Indicador de Origem (Importação vs Mercado Interno)
        if self.doc.indicador_de_origem.is_none()
            && let Some(cfop) = self.doc.cfop
        {
            self.doc.indicador_de_origem = Some(if (3000..=3999).contains(&cfop) {
                IndicadorOrigem::Importacao
            } else {
                IndicadorOrigem::MercadoInterno
            });
        }

        // 4. Tipo de Crédito
        if self.doc.tipo_de_credito.is_none() {
            self.doc.tipo_de_credito = determinar_tipo_de_credito(
                self.doc.cst,
                self.doc.aliq_pis,
                self.doc.aliq_cofins,
                self.doc.cod_credito,
                self.doc.cfop,
            );

            // Sincronização de Contexto:
            // Se o Tipo de Crédito for Importação (ex: via código 108), forçamos a origem.
            if self.doc.tipo_de_credito == Some(TipoDeCredito::Importacao) {
                self.doc.indicador_de_origem = Some(IndicadorOrigem::Importacao);
            }
        }

        self.doc.format();
        self.doc
    }
}

/// Determina o `TipoDeCredito` com base nas regras do Guia Prático da EFD Contribuições.
fn determinar_tipo_de_credito(
    cst_cofins: Option<u16>,
    aliq_pis: Option<f64>,
    aliq_cofins: Option<f64>,
    cod_credito: Option<u16>,
    cfop: Option<u16>, // Alterado: Recebe o dado bruto (CFOP) ao invés do derivado
) -> Option<TipoDeCredito> {
    // ------------------------------------------------------------------------
    // 1. Prioridade Absoluta: Código do Crédito Informado (Blocos M e 1)
    // ------------------------------------------------------------------------
    // O código SPED é composto por 3 dígitos: XYY.
    // X (centena) = Tipo de Rateio (1 a 4).
    // YY (resto)  = Tipo de Crédito (1 a 99).
    if let Some(credito) = cod_credito
        .filter(|&cod| TipoDeRateio::from_u16(cod / 100).is_some()) // Valida o digito 'X'
        .and_then(|cod| TipoDeCredito::from_u16(cod % 100))
    // Extrai e converte 'YY'
    {
        return Some(credito);
    }

    // ------------------------------------------------------------------------
    // 2. Heurística (Fallback): Baseada em Alíquotas, Origem e CST
    // ------------------------------------------------------------------------

    // Normaliza valores para f64 (None vira 0.0) para simplificar comparações
    let pis = aliq_pis.unwrap_or_default();
    let cof = aliq_cofins.unwrap_or_default();

    // Pré-condição: Para haver crédito, deve haver alíquota positiva em pelo menos um tributo.
    // Utiliza o trait FloatExt para evitar falsos positivos com ruído numérico.
    if !pis.eh_maior_que_zero() && !cof.eh_maior_que_zero() {
        return None;
    }

    // Define origem baseada no CFOP (Faixa 3000-3999 é Importação)
    let is_importacao = cfop.is_some_and(|c| (3000..=3999).contains(&c));

    match (is_importacao, cst_cofins) {
        // Regra A: Importação
        (true, _) => Some(TipoDeCredito::Importacao),

        // Regra B: Mercado Interno + CST Básico (50-56)
        (false, Some(50..=56)) => {
            // Verifica se as alíquotas correspondem exatamente ao básico (1.65% e 7.6%)
            let aliquotas_basicas = pis.eh_igual(ALIQ_BASICA_PIS) && cof.eh_igual(ALIQ_BASICA_COF);

            if aliquotas_basicas {
                Some(TipoDeCredito::AliquotaBasica)
            } else {
                Some(TipoDeCredito::AliquotasDiferenciadas)
            }
        }

        // Regra C: Mercado Interno + Crédito Presumido (CST 60-66)
        (false, Some(60..=66)) => {
            // A função `cred_presumido` deve conter a lógica específica da agroindústria/transportes
            if cred_presumido(pis, cof) {
                Some(TipoDeCredito::PresumidoAgroindustria)
            } else {
                Some(TipoDeCredito::OutrosCreditosPresumidos)
            }
        }

        _ => None,
    }
}

// ============================================================================
// SEÇÃO 6: PROCESSAMENTO DE BLOCOS (Despacho Estático)
// Logica de travessia e correlação (Strict Dispatch)
// ============================================================================

pub fn process_block_lines(bloco: char, file: &SpedFile, ctx: &SpedContext) -> Vec<DocsFiscais> {
    let records = match file.obter_bloco_option(bloco) {
        Some(l) => l,
        None => return Vec::new(),
    };

    let mut docs = Vec::with_capacity(records.len());

    match bloco {
        'A' => BlockAProcessor::default().process(records, ctx, &mut docs),
        'C' => BlockCProcessor::default().process(records, ctx, &mut docs),
        'D' => BlockDProcessor::default().process(records, ctx, &mut docs),
        'F' => BlockFProcessor::default().process(records, ctx, &mut docs),
        'I' => BlockIProcessor::default().process(records, ctx, &mut docs),
        'M' => BlockMProcessor::default().process(records, ctx, &mut docs),
        '1' => process_block_1(records, ctx, &mut docs),
        _ => {}
    }

    docs
}

// --- Block A (Serviços) ---
#[derive(Default)]
struct BlockAProcessor<'a> {
    a100: Option<&'a RegistroA100>,
    current_cnpj: Option<String>,
}
impl<'a> BlockAProcessor<'a> {
    fn process(
        &mut self,
        records: &'a [SpedRecord],
        ctx: &SpedContext,
        docs: &mut Vec<DocsFiscais>,
    ) {
        self.current_cnpj = Some(ctx.estabelecimento_cnpj.clone());
        for record in records {
            let generic = match record {
                SpedRecord::Generic(g) => g,
                _ => continue,
            };
            match generic.registro_name() {
                "A010" => {
                    if let Ok(r) = record.downcast_ref::<RegistroA010>() {
                        self.current_cnpj = r.cnpj.clone();
                    }
                }
                "A100" => self.a100 = record.downcast_ref().ok(),
                "A170" => {
                    if let (Ok(filho), Some(pai)) =
                        (record.downcast_ref::<RegistroA170>(), self.a100)
                    {
                        let b = DocsBuilder::from_child_and_parent(
                            ctx,
                            filho,
                            Some(pai),
                            self.current_cnpj.as_ref(),
                        );
                        docs.push(b.build());
                    }
                }
                _ => {}
            }
        }
    }
}

// --- Block C (Mercadorias - Complexo) ---
#[derive(Default)]
struct BlockCProcessor<'a> {
    c100: Option<&'a RegistroC100>,
    c180: Option<&'a RegistroC180>,
    c190: Option<&'a RegistroC190>,
    c380: Option<&'a RegistroC380>,
    c395: Option<&'a RegistroC395>,
    c400: Option<&'a RegistroC400>,
    c405: Option<&'a RegistroC405>,
    c490: Option<&'a RegistroC490>,
    c500: Option<&'a RegistroC500>,
    c600: Option<&'a RegistroC600>,
    c860: Option<&'a RegistroC860>,
    correlation: CorrelationManager,
    c195_indices: Vec<usize>,
    current_cnpj: Option<String>,
}

impl<'a> BlockCProcessor<'a> {
    fn process(
        &mut self,
        records: &'a [SpedRecord],
        ctx: &SpedContext,
        docs: &mut Vec<DocsFiscais>,
    ) {
        self.current_cnpj = Some(ctx.estabelecimento_cnpj.clone());
        for record in records {
            let generic = match record {
                SpedRecord::Generic(g) => g,
                _ => continue,
            };
            match generic.registro_name() {
                "C010" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC010>() {
                        self.current_cnpj = r.cnpj.clone();
                    }
                }
                "C100" => {
                    self.c100 = record.downcast_ref().ok();
                    self.correlation.clear();
                }
                "C170" => {
                    if let (Ok(filho), Some(pai)) =
                        (record.downcast_ref::<RegistroC170>(), self.c100)
                    {
                        let mut b = DocsBuilder::from_child_and_parent(
                            ctx,
                            filho,
                            Some(pai),
                            self.current_cnpj.as_ref(),
                        );
                        b.doc.data_entrada = pai.dt_e_s;
                        b.doc.num_item = filho.num_item.parse_opt();
                        docs.push(b.build());
                    }
                }
                "C175" => {
                    if let (Ok(r), Some(p)) = (record.downcast_ref::<RegistroC175>(), self.c100) {
                        docs.push(
                            DocsBuilder::from_child_and_parent(
                                ctx,
                                r,
                                Some(p),
                                self.current_cnpj.as_ref(),
                            )
                            .build(),
                        );
                    }
                }
                "C180" => {
                    self.c180 = record.downcast_ref().ok();
                    self.correlation.clear();
                }
                "C181" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC181>() {
                        self.correlation.store(
                            r.cst_pis.as_ref(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            r.cfop.as_deref(),
                            None,
                        );
                    }
                }
                "C185" => {
                    if let (Ok(r), Some(p)) = (record.downcast_ref::<RegistroC185>(), self.c180) {
                        docs.push(
                            DocsBuilder::from_child_and_parent(
                                ctx,
                                r,
                                Some(p),
                                self.current_cnpj.as_ref(),
                            )
                            .resolve_pis_correlation(&self.correlation, r, Some(p))
                            .build(),
                        );
                    }
                }
                "C190" => {
                    self.c190 = record.downcast_ref().ok();
                    self.correlation.clear();
                    self.c195_indices.clear();
                }
                "C191" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC191>() {
                        self.correlation.store(
                            r.cst_pis.as_ref(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            r.cfop.as_deref(),
                            r.cnpj_cpf_part.as_deref(),
                        );
                    }
                }
                "C195" => {
                    if let (Ok(r), Some(p)) = (record.downcast_ref::<RegistroC195>(), self.c190) {
                        let b = DocsBuilder::from_child_and_parent(
                            ctx,
                            r,
                            Some(p),
                            self.current_cnpj.as_ref(),
                        )
                        .resolve_pis_correlation(
                            &self.correlation,
                            r,
                            Some(p),
                        );
                        docs.push(b.build());
                        self.c195_indices.push(docs.len() - 1);
                    }
                }
                "C199" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC199>()
                        && let Some(num_doc) = &r.num_doc_imp
                    {
                        let txt = format!("Número do documento de Importação: {}", num_doc);
                        for &idx in &self.c195_indices {
                            if let Some(doc) = docs.get_mut(idx) {
                                doc.complementar = if doc.complementar.is_empty() {
                                    txt.clone()
                                } else {
                                    format!("{} {}", doc.complementar, txt)
                                };
                            }
                        }
                    }
                }
                "C380" => {
                    self.c380 = record.downcast_ref().ok();
                    self.correlation.clear();
                }
                "C381" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC381>() {
                        self.correlation.store(
                            r.cst_pis.as_ref(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            None,
                            None,
                        );
                    }
                }
                "C385" => {
                    if let (Ok(r), Some(p)) = (record.downcast_ref::<RegistroC385>(), self.c380) {
                        docs.push(
                            DocsBuilder::from_child_and_parent(
                                ctx,
                                r,
                                Some(p),
                                self.current_cnpj.as_ref(),
                            )
                            .resolve_pis_correlation(&self.correlation, r, Some(p))
                            .build(),
                        );
                    }
                }
                "C395" => {
                    self.c395 = record.downcast_ref().ok();
                }
                "C396" => {
                    if let (Ok(r), Some(p)) = (record.downcast_ref::<RegistroC396>(), self.c395) {
                        docs.push(
                            DocsBuilder::from_child_and_parent(
                                ctx,
                                r,
                                Some(p),
                                self.current_cnpj.as_ref(),
                            )
                            .build(),
                        );
                    }
                }
                "C400" => {
                    self.c400 = record.downcast_ref().ok();
                    self.correlation.clear();
                }
                "C405" => {
                    self.c405 = record.downcast_ref().ok();
                }
                "C481" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC481>() {
                        self.correlation.store(
                            r.cst_pis.as_ref(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            None,
                            None,
                        );
                    }
                }
                "C485" => {
                    if let (Ok(r), Some(p)) = (record.downcast_ref::<RegistroC485>(), self.c405) {
                        docs.push(
                            DocsBuilder::from_child_and_parent(
                                ctx,
                                r,
                                Some(p),
                                self.current_cnpj.as_ref(),
                            )
                            .resolve_pis_correlation(&self.correlation, r, Some(p))
                            .build(),
                        );
                    }
                }
                "C490" => {
                    self.c490 = record.downcast_ref().ok();
                    self.correlation.clear();
                }
                "C491" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC491>() {
                        self.correlation.store(
                            r.cst_pis.as_ref(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            None,
                            None,
                        );
                    }
                }
                "C495" => {
                    if let (Ok(r), Some(p)) = (record.downcast_ref::<RegistroC495>(), self.c490) {
                        docs.push(
                            DocsBuilder::from_child_and_parent(
                                ctx,
                                r,
                                Some(p),
                                self.current_cnpj.as_ref(),
                            )
                            .resolve_pis_correlation(&self.correlation, r, Some(p))
                            .build(),
                        );
                    }
                }
                "C500" => {
                    self.c500 = record.downcast_ref().ok();
                    self.correlation.clear();
                }
                "C501" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC501>() {
                        self.correlation.store(
                            r.cst_pis.as_ref(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            None,
                            None,
                        );
                    }
                }
                "C505" => {
                    if let (Ok(r), Some(p)) = (record.downcast_ref::<RegistroC505>(), self.c500) {
                        let mut b = DocsBuilder::from_child_and_parent(
                            ctx,
                            r,
                            Some(p),
                            self.current_cnpj.as_ref(),
                        )
                        .resolve_pis_correlation(
                            &self.correlation,
                            r,
                            Some(p),
                        );
                        b.doc.data_entrada = p.dt_ent;
                        docs.push(b.build());
                    }
                }
                "C600" => {
                    self.c600 = record.downcast_ref().ok();
                    self.correlation.clear();
                }
                "C601" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC601>() {
                        self.correlation.store(
                            r.cst_pis.as_ref(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            None,
                            None,
                        );
                    }
                }
                "C605" => {
                    if let (Ok(r), Some(p)) = (record.downcast_ref::<RegistroC605>(), self.c600) {
                        docs.push(
                            DocsBuilder::from_child_and_parent(
                                ctx,
                                r,
                                Some(p),
                                self.current_cnpj.as_ref(),
                            )
                            .resolve_pis_correlation(&self.correlation, r, Some(p))
                            .build(),
                        );
                    }
                }
                "C860" => {
                    self.c860 = record.downcast_ref().ok();
                }
                "C870" => {
                    if let (Ok(r), Some(p)) = (record.downcast_ref::<RegistroC870>(), self.c860) {
                        docs.push(
                            DocsBuilder::from_child_and_parent(
                                ctx,
                                r,
                                Some(p),
                                self.current_cnpj.as_ref(),
                            )
                            .build(),
                        );
                    }
                }
                "C880" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC880>() {
                        docs.push(
                            DocsBuilder::from_child(ctx, r, self.current_cnpj.as_ref()).build(),
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

// --- Block D (Transportes) ---
#[derive(Default)]
struct BlockDProcessor<'a> {
    d100: Option<&'a RegistroD100>,
    d200: Option<&'a RegistroD200>,
    d500: Option<&'a RegistroD500>,
    d600: Option<&'a RegistroD600>,
    correlation: CorrelationManager,
    current_cnpj: Option<String>,
}
impl<'a> BlockDProcessor<'a> {
    fn process(
        &mut self,
        records: &'a [SpedRecord],
        ctx: &SpedContext,
        docs: &mut Vec<DocsFiscais>,
    ) {
        self.current_cnpj = Some(ctx.estabelecimento_cnpj.clone());
        for record in records {
            let generic = match record {
                SpedRecord::Generic(g) => g,
                _ => continue,
            };
            match generic.registro_name() {
                "D010" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD010>() {
                        self.current_cnpj = r.cnpj.clone();
                    }
                }
                "D100" => {
                    self.d100 = record.downcast_ref().ok();
                    self.correlation.clear();
                }
                "D101" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD101>() {
                        self.correlation.store(
                            r.cst_pis.as_ref(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            None,
                            None,
                        );
                    }
                }
                "D105" => {
                    if let (Ok(r), Some(p)) = (record.downcast_ref::<RegistroD105>(), self.d100) {
                        docs.push(
                            DocsBuilder::from_child_and_parent(
                                ctx,
                                r,
                                Some(p),
                                self.current_cnpj.as_ref(),
                            )
                            .resolve_pis_correlation(&self.correlation, r, Some(p))
                            .build(),
                        );
                    }
                }
                "D200" => {
                    self.d200 = record.downcast_ref().ok();
                    self.correlation.clear();
                }
                "D201" => {
                    if let (Ok(r), Some(p)) = (record.downcast_ref::<RegistroD201>(), self.d200) {
                        docs.push(
                            DocsBuilder::from_child_and_parent(
                                ctx,
                                r,
                                Some(p),
                                self.current_cnpj.as_ref(),
                            )
                            .resolve_pis_correlation(&self.correlation, r, Some(p))
                            .build(),
                        );
                    }
                }
                "D205" => {
                    if let (Ok(r), Some(p)) = (record.downcast_ref::<RegistroD205>(), self.d200) {
                        let mut b = DocsBuilder::from_child_and_parent(
                            ctx,
                            r,
                            Some(p),
                            self.current_cnpj.as_ref(),
                        )
                        .resolve_pis_correlation(
                            &self.correlation,
                            r,
                            Some(p),
                        );
                        b.doc.num_doc = p.num_doc_ini.parse_opt();
                        docs.push(b.build());
                    }
                }
                "D300" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD300>() {
                        let mut b = DocsBuilder::from_child(ctx, r, self.current_cnpj.as_ref());
                        b.doc.data_emissao = r.dt_ref;
                        docs.push(b.build());
                    }
                }
                "D350" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD350>() {
                        let mut b = DocsBuilder::from_child(ctx, r, self.current_cnpj.as_ref());
                        b.doc.data_emissao = r.dt_doc;
                        docs.push(b.build());
                    }
                }
                "D500" => {
                    self.d500 = record.downcast_ref().ok();
                    self.correlation.clear();
                }
                "D501" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD501>() {
                        self.correlation.store(
                            r.cst_pis.as_ref(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            None,
                            None,
                        );
                    }
                }
                "D505" => {
                    if let (Ok(r), Some(p)) = (record.downcast_ref::<RegistroD505>(), self.d500) {
                        docs.push(
                            DocsBuilder::from_child_and_parent(
                                ctx,
                                r,
                                Some(p),
                                self.current_cnpj.as_ref(),
                            )
                            .resolve_pis_correlation(&self.correlation, r, Some(p))
                            .build(),
                        );
                    }
                }
                "D600" => {
                    self.d600 = record.downcast_ref().ok();
                    self.correlation.clear();
                }
                "D601" => {
                    if let Ok(r) = record.downcast_ref::<RegistroD601>() {
                        self.correlation.store(
                            r.cst_pis.as_ref(),
                            r.vl_item,
                            r.aliq_pis,
                            r.vl_pis,
                            None,
                            None,
                        );
                    }
                }
                "D605" => {
                    if let (Ok(r), Some(p)) = (record.downcast_ref::<RegistroD605>(), self.d600) {
                        docs.push(
                            DocsBuilder::from_child_and_parent(
                                ctx,
                                r,
                                Some(p),
                                self.current_cnpj.as_ref(),
                            )
                            .resolve_pis_correlation(&self.correlation, r, Some(p))
                            .build(),
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

// --- Block F (Financeiro / Diversos) ---
// Utiliza intensivamente RegistroGeral para registros autônomos
#[derive(Default)]
struct BlockFProcessor {
    current_cnpj: Option<String>,
}
impl BlockFProcessor {
    fn process(&mut self, records: &[SpedRecord], ctx: &SpedContext, docs: &mut Vec<DocsFiscais>) {
        self.current_cnpj = Some(ctx.estabelecimento_cnpj.clone());
        for record in records {
            let generic = match record {
                SpedRecord::Generic(g) => g,
                _ => continue,
            };
            match generic.registro_name() {
                "F010" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF010>() {
                        self.current_cnpj = r.cnpj.clone();
                    }
                }
                "F100" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF100>() {
                        // F100 implementa tanto Geral (para cabeçalho) quanto Filho (para itens/tributos)
                        // Aqui usamos a lógica de Filho sem Pai
                        docs.push(
                            DocsBuilder::from_child(ctx, r, self.current_cnpj.as_ref()).build(),
                        );
                    }
                }
                "F120" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF120>() {
                        docs.push(
                            DocsBuilder::from_child(ctx, r, self.current_cnpj.as_ref()).build(),
                        );
                    }
                }
                "F130" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF130>() {
                        // F130 usa VL_BC_COFINS como VL_ITEM (Custo Aquisição) conforme dispatch_table
                        docs.push(
                            DocsBuilder::from_child(ctx, r, self.current_cnpj.as_ref()).build(),
                        );
                    }
                }
                "F150" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF150>() {
                        docs.push(
                            DocsBuilder::from_child(ctx, r, self.current_cnpj.as_ref()).build(),
                        );
                    }
                }
                "F200" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF200>() {
                        docs.push(
                            DocsBuilder::from_geral(ctx, r, self.current_cnpj.as_ref()).build(),
                        );
                    }
                }
                "F205" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF205>() {
                        docs.push(
                            DocsBuilder::from_geral(ctx, r, self.current_cnpj.as_ref()).build(),
                        );
                    }
                }
                "F210" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF210>() {
                        docs.push(
                            DocsBuilder::from_geral(ctx, r, self.current_cnpj.as_ref()).build(),
                        );
                    }
                }
                "F500" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF500>() {
                        docs.push(
                            DocsBuilder::from_geral(ctx, r, self.current_cnpj.as_ref()).build(),
                        );
                    }
                }
                "F510" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF510>() {
                        docs.push(
                            DocsBuilder::from_geral(ctx, r, self.current_cnpj.as_ref()).build(),
                        );
                    }
                }
                "F550" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF550>() {
                        docs.push(
                            DocsBuilder::from_geral(ctx, r, self.current_cnpj.as_ref()).build(),
                        );
                    }
                }
                "F560" => {
                    if let Ok(r) = record.downcast_ref::<RegistroF560>() {
                        docs.push(
                            DocsBuilder::from_geral(ctx, r, self.current_cnpj.as_ref()).build(),
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

// --- Block I (Pessoa Jurídica) ---
#[derive(Default)]
struct BlockIProcessor {
    current_cnpj: Option<String>,
}
impl BlockIProcessor {
    fn process(&mut self, records: &[SpedRecord], ctx: &SpedContext, docs: &mut Vec<DocsFiscais>) {
        self.current_cnpj = Some(ctx.estabelecimento_cnpj.clone());
        for record in records {
            let generic = match record {
                SpedRecord::Generic(g) => g,
                _ => continue,
            };
            match generic.registro_name() {
                "I010" => {
                    if let Ok(r) = record.downcast_ref::<RegistroI010>() {
                        self.current_cnpj = r.cnpj.clone();
                    }
                }
                "I100" => {
                    if let Ok(r) = record.downcast_ref::<RegistroI100>() {
                        docs.push(
                            DocsBuilder::from_child(ctx, r, self.current_cnpj.as_ref()).build(),
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

// --- Block M (Apuração e Ajustes) ---
#[derive(Default)]
struct BlockMProcessor<'a> {
    m100: Option<&'a RegistroM100>,
    m500: Option<&'a RegistroM500>,
}
impl<'a> BlockMProcessor<'a> {
    fn process(
        &mut self,
        records: &'a [SpedRecord],
        ctx: &SpedContext,
        docs: &mut Vec<DocsFiscais>,
    ) {
        for record in records {
            let generic = match record {
                SpedRecord::Generic(g) => g,
                _ => continue,
            };
            match generic.registro_name() {
                "M100" => {
                    if let Ok(r) = record.downcast_ref::<RegistroM100>() {
                        self.m100 = Some(r);
                        docs.extend(mappers::build_m100(r, ctx));
                    }
                }
                "M500" => {
                    if let Ok(r) = record.downcast_ref::<RegistroM500>() {
                        self.m500 = Some(r);
                        docs.extend(mappers::build_m500(r, ctx));
                    }
                }
                "M505" => {
                    if let (Ok(r), Some(p)) = (record.downcast_ref::<RegistroM505>(), self.m500) {
                        let mut b = DocsBuilder::from_child(ctx, r, None); // M505 herda do M500 apenas dados logicos, nao hierarquia fiscal padrao
                        b.doc.data_emissao = ctx.periodo_de_apuracao;
                        b.doc.cod_credito = p.cod_cred.parse_opt();
                        // Herda alíquota do Pai M500 para tentar correlação de PIS
                        b.doc.aliq_cofins = p.aliq_cofins.to_f64_opt();
                        b.doc.tipo_de_operacao = Some(TipoOperacao::Detalhamento);
                        b.doc.participante_nome = "MATRIZ".to_string(); // Block M é sempre matriz
                        b.doc.participante_cnpj = "12345678901234".to_string();

                        // Correlação por Alíquota (Heurística Padrão Dispatch Table)
                        if let Some(ac) = b.doc.aliq_cofins {
                            if (ac - 7.6).abs() < 0.001 {
                                b.doc.aliq_pis = Some(1.65);
                            } else if (ac - 3.0).abs() < 0.001 {
                                b.doc.aliq_pis = Some(0.65);
                            }
                        }
                        docs.push(b.build());
                    }
                }
                _ => {}
            }
        }
    }
}

// --- Block 1 (Controle) ---
fn process_block_1(records: &[SpedRecord], ctx: &SpedContext, docs: &mut Vec<DocsFiscais>) {
    for record in records {
        let generic = match record {
            SpedRecord::Generic(g) => g,
            _ => continue,
        };
        match generic.registro_name() {
            "1100" => {
                if let Ok(r) = record.downcast_ref::<Registro1100>()
                    && let Some(d) = mappers::build_ctrl_credito(
                        r.line_number,
                        "1100",
                        r.vl_cred_desc_efd,
                        r.get_cod_cred().as_deref(),
                        r.per_apu_cred.as_deref(),
                        ctx,
                    )
                {
                    docs.push(d);
                }
            }
            "1500" => {
                if let Ok(r) = record.downcast_ref::<Registro1500>()
                    && let Some(d) = mappers::build_ctrl_credito(
                        r.line_number,
                        "1500",
                        r.vl_cred_desc_efd,
                        r.get_cod_cred().as_deref(),
                        r.per_apu_cred.as_deref(),
                        ctx,
                    )
                {
                    docs.push(d);
                }
            }
            _ => {}
        }
    }
}

// ============================================================================
// SEÇÃO 6: MAPPERS (Pure Logic)
// Lógica encapsulada para geração de múltiplos documentos ou regras complexas
// ============================================================================

mod mappers {
    use super::*;

    pub fn build_m100(reg: &RegistroM100, ctx: &SpedContext) -> Vec<DocsFiscais> {
        let mut list = Vec::with_capacity(4);

        let mut b = DocsBuilder::new(ctx, "M100", reg.line_number, None::<&String>);
        b.doc.data_emissao = ctx.periodo_de_apuracao;
        b.doc.aliq_pis = reg.aliq_pis.to_f64_opt();
        b.doc.valor_pis = reg.vl_cred.to_f64_opt();
        // Visualmente coloca BC PIS na coluna de COFINS para alinhamento em relatórios (Legacy behavior)
        b.doc.valor_bc = reg.vl_bc_pis.to_f64_opt();
        b.doc.cod_credito = reg.cod_cred.parse_opt();
        b.doc.participante_nome = "MATRIZ".to_string();
        b.doc.participante_cnpj = "12345678901234".to_string();

        list.push(b.build());

        // Gera registros virtuais de Ajuste
        append_adjustments(
            &mut list,
            ctx,
            reg.line_number,
            "M100_AJ",
            reg.vl_ajus_acres,
            reg.vl_ajus_reduc,
            reg.vl_cred_desc,
        );
        list
    }

    pub fn build_m500(reg: &RegistroM500, ctx: &SpedContext) -> Vec<DocsFiscais> {
        let mut list = Vec::with_capacity(4);

        let mut b = DocsBuilder::new(ctx, "M500", reg.line_number, None::<&String>);
        b.doc.data_emissao = ctx.periodo_de_apuracao;
        b.doc.aliq_cofins = reg.aliq_cofins.to_f64_opt();
        b.doc.valor_cofins = reg.vl_cred.to_f64_opt();
        b.doc.valor_bc = reg.vl_bc_cofins.to_f64_opt();
        b.doc.cod_credito = reg.cod_cred.parse_opt();
        b.doc.participante_nome = "MATRIZ".to_string();
        b.doc.participante_cnpj = "12345678901234".to_string();

        list.push(b.build());

        append_adjustments(
            &mut list,
            ctx,
            reg.line_number,
            "M500_AJ",
            reg.vl_ajus_acres,
            reg.vl_ajus_reduc,
            reg.vl_cred_desc,
        );
        list
    }

    fn append_adjustments(
        list: &mut Vec<DocsFiscais>,
        ctx: &SpedContext,
        line: usize,
        reg_name: &str,
        acres: Option<Decimal>,
        reduc: Option<Decimal>,
        desc: Option<Decimal>,
    ) {
        let v_acres = acres.abs_f64();
        let v_reduc = reduc.abs_f64();
        let v_desc = desc.abs_f64();

        // Helper: Cria doc de ajuste
        // O legado insere valores negativos para Redução e Desconto no campo VL_ITEM
        let make_adj = |val: f64, op: TipoOperacao, negative: bool| {
            let mut b = DocsBuilder::new(ctx, reg_name, line, None::<&String>);
            b.doc.valor_item = Some(if negative { -val.abs() } else { val.abs() });
            b.doc.tipo_de_operacao = Some(op);
            b.doc.participante_cnpj = "12345678901234".to_string(); // Placeholder Matriz
            b.doc.participante_nome = "MATRIZ".to_string();
            b.build()
        };

        if v_acres > 0.0 {
            list.push(make_adj(v_acres, TipoOperacao::AjusteAcrescimo, false));
        }
        if v_reduc > 0.0 {
            list.push(make_adj(v_reduc, TipoOperacao::AjusteReducao, true));
        }
        if v_desc > 0.0 {
            list.push(make_adj(v_desc, TipoOperacao::DescontoNoPeriodo, true));
        }
    }

    pub fn build_ctrl_credito(
        line: usize,
        reg_name: &str,
        vl_desc: Option<Decimal>,
        cod_cred: Option<&str>,
        per_apu_orig: Option<&str>,
        ctx: &SpedContext,
    ) -> Option<DocsFiscais> {
        let val = vl_desc.abs_f64();
        if val == 0.0 {
            return None;
        }

        let mut b = DocsBuilder::new(ctx, reg_name, line, None);
        b.doc.valor_item = Some(-val); // Negativo (Desconto)
        b.doc.tipo_de_operacao = Some(TipoOperacao::DescontoPosterior);
        b.doc.participante_cnpj = "12345678901234".to_string();
        b.doc.participante_nome = "MATRIZ".to_string();
        b.doc.cod_credito = cod_cred.parse_opt();

        // Lógica de validação de período de origem (Logging ported)
        if let (Some(orig_str), Some(curr_date)) = (per_apu_orig, ctx.periodo_de_apuracao)
            && orig_str.len() == 6
        {
            let m = orig_str[0..2].parse::<u32>().unwrap_or(0);
            let y = orig_str[2..6].parse::<i32>().unwrap_or(0);
            if let Some(orig_date) = NaiveDate::from_ymd_opt(y, m, 1)
                && orig_date != curr_date
            {
                b.doc.complementar = format!("ORIGEM DIFERENTE: {}", orig_str);
                // Em produção, isso seria logado.
            }
        }

        Some(b.build())
    }
}
