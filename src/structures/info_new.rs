use chrono::{Datelike, NaiveDate};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use std::{collections::HashMap, sync::Arc};

use crate::{
    ALIQ_BASICA_COF, ALIQ_BASICA_PIS, DecimalExt, DocsFiscaisNew, FloatExt, IndicadorOrigem,
    MesesDoAno, SpedContext, SpedFile, SpedRecord, SpedRecordTrait, StringParser, TipoDeCredito,
    TipoDeRateio, TipoOperacao, blocos::*, capture_cnpj, cred_presumido, impl_dopai, impl_filho,
    obter_cod_da_natureza_da_bc, obter_modelo_do_documento_fiscal, obter_tipo_do_item,
    process_child_and_parent, process_correlations, process_only_child, store_pis,
};

/*
O Padrão Adapter

Em info_new.rs foi aplicado o Pattern Adapter.

As classes incompatíveis (RegistroC170, RegistroD100, RegistroF100) têm campos com
nomes diferentes (vl_item, vl_oper, vl_doc).

As Traits (RegistroGeral, RegistroPai, RegistroFilho) atuam como uma interface comum.

As macros em info_new.rs são os adaptadores que conectam a interface comum às structs específicas.
*/

// ============================================================================
// SEÇÃO 2: DEFINIÇÃO DE TRAITS (ABSTRAÇÃO DE REGISTROS)
// Interfaces para normalizar o acesso aos dados dos registros de forma polimórfica.
// Cobrem TODOS os campos necessários para preencher DocsFiscaisNew.
// ============================================================================

/// Representa um registro "Pai" (Cabeçalho) que fornece contexto para os "Filhos".
///
/// Ex: C100, D100. Fornece dados contextuais para os filhos.
pub trait RegistroPai {
    /// Data da Emissão do Documento Fiscal
    fn get_dt_emissao(&self) -> Option<NaiveDate> {
        None
    }
    /// Data da Entrada da Mercadoria ou Início
    /// da Execução do Serviço
    fn get_dt_entrada(&self) -> Option<NaiveDate> {
        None
    }
    /// Chave da Nota Fiscal Eletrônica
    fn get_chave(&self) -> Option<&str> {
        None
    }
    /// Código da conta analítica contábil
    fn get_cod_cta(&self) -> Option<&str> {
        None
    }
    /// Código do item (campo 02 do Registro 0200)
    fn get_cod_item(&self) -> Option<&str> {
        None
    }
    /// Código do modelo do documento fiscal, conforme Tabela 4.1.1
    fn get_cod_mod(&self) -> Option<&str> {
        None
    }
    /// Código da Nomenclatura Comum do Mercosul
    fn get_cod_ncm(&self) -> Option<&str> {
        None
    }
    /// Código do Participante
    fn get_cod_part(&self) -> Option<&str> {
        None
    }
    /// Número do documento fiscal
    fn get_num_doc(&self) -> Option<&str> {
        None
    }
    /// Valor da base de cálculo do ICMS. Ver Registro D100
    fn get_valor_bc_icms(&self) -> Option<Decimal> {
        None
    }
    /// Valor acumulado do ICMS. Ver Registro C500
    fn get_valor_icms(&self) -> Option<Decimal> {
        None
    }
}

// Implementação default para Unit type (sem pai)
impl RegistroPai for () {} // Implementação para filhos órfãos

/// Registro "Filho" contendo detalhes do item e tributação (Ex: C170, C191).
pub trait RegistroFilho: SpedRecordTrait {
    fn get_dt_emissao(&self) -> Option<NaiveDate> {
        None
    }
    fn get_dt_entrada(&self) -> Option<NaiveDate> {
        None
    }

    // Identificação por Código
    fn get_cod_cta(&self) -> Option<&str> {
        None
    }
    fn get_cod_cred(&self) -> Option<u16> {
        None
    }
    fn get_cod_item(&self) -> Option<&str> {
        None
    }
    fn get_cod_mod(&self) -> Option<&str> {
        None
    }
    fn get_cod_nat(&self) -> Option<&str> {
        None
    }
    fn get_cod_part(&self) -> Option<&str> {
        None
    }

    // Identificação do Item
    fn get_num_item(&self) -> Option<u16> {
        None
    }
    fn get_descr_item(&self) -> Option<&str> {
        None
    }
    fn get_descr_compl(&self) -> Option<&str> {
        None
    }
    fn get_info_compl(&self) -> Option<&str> {
        None
    }
    // Caso com CNPJ ou CPF
    fn get_part_override(&self) -> Option<&str> {
        None
    }

    // Classificação Fiscal
    fn get_cst_pis(&self) -> Option<u16> {
        None
    }
    fn get_cst_cofins(&self) -> Option<u16> {
        None
    }
    fn get_cfop(&self) -> Option<u16> {
        None
    }
    fn get_nat_bc_cred(&self) -> Option<&str> {
        None
    }
    fn get_ind_orig_cred(&self) -> Option<&str> {
        None
    }

    // Valores Monetários do Item
    fn get_valor_item(&self) -> Option<Decimal> {
        None
    }
    fn get_valor_desc(&self) -> Option<Decimal> {
        None
    }

    // PIS
    fn get_valor_bc_pis(&self) -> Option<Decimal> {
        None
    }
    fn get_aliq_pis(&self) -> Option<Decimal> {
        None
    }
    fn get_valor_pis(&self) -> Option<Decimal> {
        None
    }

    // COFINS
    fn get_valor_bc_cofins(&self) -> Option<Decimal> {
        None
    }
    fn get_aliq_cofins(&self) -> Option<Decimal> {
        None
    }
    fn get_valor_cofins(&self) -> Option<Decimal> {
        None
    }

    // Outros Tributos (ICMS, ISS, IPI)
    fn get_valor_iss(&self) -> Option<Decimal> {
        None
    }
    fn get_valor_ipi(&self) -> Option<Decimal> {
        None
    }
    fn get_valor_icms(&self) -> Option<Decimal> {
        None
    }

    fn get_aliq_icms(&self) -> Option<Decimal> {
        None
    }
    fn get_valor_bc_icms(&self) -> Option<Decimal> {
        None
    }
    fn get_valor_icms_st(&self) -> Option<Decimal> {
        None
    }
    fn get_valor_bc_icms_st(&self) -> Option<Decimal> {
        None
    }
}

// ============================================================================
// SEÇÃO 4: BINDINGS (Mapeamento Struct -> Traits)
// ============================================================================

// Bloco A
impl_dopai!(RegistroA100, {
    get_dt_emissao: dt_doc, get_dt_entrada: dt_exe_serv, get_chave: chv_nfse,
    get_cod_part: cod_part, get_num_doc: num_doc
});
impl_filho!(RegistroA170, {
    get_num_item: num_item, get_cod_item: cod_item, get_descr_compl: descr_compl, get_valor_item: vl_item,
    get_valor_desc: vl_desc, get_nat_bc_cred: nat_bc_cred, get_ind_orig_cred: ind_orig_cred,
    get_cst_pis: cst_pis, get_valor_bc_pis: vl_bc_pis, get_aliq_pis: aliq_pis, get_valor_pis: vl_pis,
    get_cst_cofins: cst_cofins, get_valor_bc_cofins: vl_bc_cofins, get_aliq_cofins: aliq_cofins,
    get_valor_cofins: vl_cofins, get_cod_cta: cod_cta
});

// Bloco C
impl_dopai!(RegistroC100, {
    get_dt_emissao: dt_doc, get_dt_entrada: dt_e_s, get_chave: chv_nfe,
    get_cod_part: cod_part, get_cod_mod: cod_mod, get_num_doc: num_doc,
});
impl_filho!(RegistroC170, {
    get_num_item: num_item, get_cod_item: cod_item, get_descr_compl: descr_compl,
    get_cst_pis: cst_pis, get_cst_cofins: cst_cofins,
    get_cfop: cfop, get_cod_nat: cod_nat, get_cod_cta: cod_cta,
    get_valor_item: vl_item, get_valor_desc: vl_desc,
    get_valor_bc_pis: vl_bc_pis, get_aliq_pis: aliq_pis, get_valor_pis: vl_pis,
    get_valor_bc_cofins: vl_bc_cofins, get_aliq_cofins: aliq_cofins, get_valor_cofins: vl_cofins,
    get_valor_bc_icms: vl_bc_icms, get_aliq_icms: aliq_icms, get_valor_icms: vl_icms,
    get_valor_bc_icms_st: vl_bc_icms_st, get_valor_icms_st: vl_icms_st, get_valor_ipi: vl_ipi
});
impl_filho!(RegistroC175, {
    get_valor_item: vl_opr, get_cst_pis: cst_pis, get_cst_cofins: cst_cofins, get_cfop: cfop,
    get_valor_bc_pis: vl_bc_pis, get_aliq_pis: aliq_pis, get_valor_pis: vl_pis,
    get_valor_bc_cofins: vl_bc_cofins, get_aliq_cofins: aliq_cofins, get_valor_cofins: vl_cofins
});

// Blocos Especiais (filhos que agem como itens)
impl_dopai!(RegistroC180, {
   get_dt_emissao: dt_doc_ini, get_cod_mod: cod_mod,
   get_cod_item: cod_item, get_cod_ncm: cod_ncm
});
impl_filho!(RegistroC181, {
    get_valor_item: vl_item, get_cst_pis: cst_pis, get_aliq_pis: aliq_pis, get_valor_pis: vl_pis,
    get_cfop: cfop, get_cod_cta: cod_cta
});
impl_filho!(RegistroC185, {
    get_valor_item: vl_item, get_cst_cofins: cst_cofins, get_aliq_cofins: aliq_cofins,
    get_valor_cofins: vl_cofins, get_valor_bc_cofins: vl_bc_cofins, get_cfop: cfop,
    get_cod_cta: cod_cta
});

/*
Procedimento adotado para registros especiais.
O registro pai C190 possui os registros filhos: C191, C195, C198 e C199.
Os registros C198 e C199 são posteriores aos registros registros C191 e C195.
Objetivo: reter informações dos registros C198 e C199 e transmiti-las aos registros C191 e C195.

Mesmo procedimento para C499, pois os registros C499 são posteriores aos registros C491 e C495.
Mesmo procedimento para D609, pois os registros D609 são posteriores aos registros D601 e D605.
*/

impl_dopai!(RegistroC190, {
    get_dt_emissao: dt_ref_ini, get_cod_mod: cod_mod,
    get_cod_item: cod_item, get_cod_ncm: cod_ncm,
});
impl_filho!(RegistroC191, {
    get_part_override: cnpj_cpf_part, get_cfop: cfop, get_cod_cta: cod_cta,
    get_cst_pis: cst_pis, get_valor_item: vl_item, get_valor_bc_pis: vl_bc_pis,
    get_aliq_pis: aliq_pis, get_valor_pis: vl_pis,
});
impl_filho!(RegistroC195, {
    get_part_override: cnpj_cpf_part, get_cfop: cfop, get_cod_cta: cod_cta,
    get_cst_cofins: cst_cofins, get_valor_item: vl_item, get_valor_bc_cofins: vl_bc_cofins,
    get_aliq_cofins: aliq_cofins, get_valor_cofins: vl_cofins,
});

impl_dopai!(RegistroC380, { get_dt_emissao: dt_doc_ini });
impl_filho!(RegistroC381, {
    get_valor_item: vl_item, get_cst_pis: cst_pis, get_aliq_pis: aliq_pis, get_valor_pis: vl_pis
});
impl_filho!(RegistroC385, {
    get_valor_item: vl_item, get_cst_cofins: cst_cofins, get_aliq_cofins: aliq_cofins,
    get_valor_cofins: vl_cofins, get_valor_bc_cofins: vl_bc_cofins
});

impl_dopai!(RegistroC395, { get_dt_emissao: dt_doc, get_cod_part: cod_part });
impl_filho!(RegistroC396, {
    get_valor_item: vl_item, get_cst_pis: cst_pis, get_cst_cofins: cst_cofins, get_aliq_pis: aliq_pis,
    get_valor_pis: vl_pis, get_aliq_cofins: aliq_cofins, get_valor_cofins: vl_cofins,
    get_valor_bc_cofins: vl_bc_cofins
});

impl_dopai!(RegistroC400, { get_cod_mod: cod_mod }); // Reter info para posterior uso por reg C485
impl_dopai!(RegistroC405, { get_dt_emissao: dt_doc }); // Reter info para posterior uso por reg C485
impl_filho!(RegistroC481, {
    get_valor_item: vl_item, get_cst_pis: cst_pis, get_aliq_pis: aliq_pis,
    get_valor_pis: vl_pis, get_valor_bc_pis: vl_bc_pis,
});
impl_filho!(RegistroC485, {
    get_valor_item: vl_item, get_cst_cofins: cst_cofins, get_aliq_cofins: aliq_cofins,
    get_valor_cofins: vl_cofins, get_valor_bc_cofins: vl_bc_cofins,
    get_cod_item: cod_item, get_cod_cta: cod_cta
});

impl_dopai!(RegistroC490, { get_dt_emissao: dt_doc_ini });
impl_filho!(RegistroC491, {
    get_valor_item: vl_item, get_cst_pis: cst_pis, get_aliq_pis: aliq_pis, get_valor_pis: vl_pis
});
impl_filho!(RegistroC495, {
    get_valor_item: vl_item, get_cst_cofins: cst_cofins, get_aliq_cofins: aliq_cofins,
    get_valor_cofins: vl_cofins, get_valor_bc_cofins: vl_bc_cofins, get_cfop: cfop
});

impl_dopai!(RegistroC500, {
    get_dt_emissao: dt_doc, get_dt_entrada: dt_ent,
    get_cod_part: cod_part, get_cod_mod: cod_mod,
    get_num_doc: num_doc, get_chave: chv_doce,
    get_valor_icms: vl_icms
});
impl_filho!(RegistroC501, {
    get_cst_pis: cst_pis, get_valor_item: vl_item, get_nat_bc_cred: nat_bc_cred,
    get_valor_bc_pis: vl_bc_pis, get_aliq_pis: aliq_pis, get_valor_pis: vl_pis,
    get_cod_cta: cod_cta
});
impl_filho!(RegistroC505, {
    get_cst_cofins: cst_cofins, get_valor_item: vl_item, get_nat_bc_cred: nat_bc_cred,
    get_valor_bc_cofins: vl_bc_cofins, get_aliq_cofins: aliq_cofins, get_valor_cofins: vl_cofins,
    get_cod_cta: cod_cta
});

impl_dopai!(RegistroC600, { get_dt_emissao: dt_doc, get_cod_mod: cod_mod, get_cod_part: cod_mun });
impl_filho!(RegistroC601, {
    get_valor_item: vl_item, get_cst_pis: cst_pis, get_aliq_pis: aliq_pis, get_valor_pis: vl_pis
});
impl_filho!(RegistroC605, {
    get_valor_item: vl_item, get_cst_cofins: cst_cofins, get_aliq_cofins: aliq_cofins,
    get_valor_cofins: vl_cofins, get_valor_bc_cofins: vl_bc_cofins
});

impl_dopai!(RegistroC860, { get_dt_emissao: dt_doc });
impl_filho!(RegistroC870, {
    get_valor_item: vl_item, get_cst_pis: cst_pis, get_cst_cofins: cst_cofins, get_cfop: cfop,
    get_aliq_pis: aliq_pis, get_valor_pis: vl_pis, get_aliq_cofins: aliq_cofins,
    get_valor_cofins: vl_cofins, get_valor_bc_cofins: vl_bc_cofins, get_cod_item: cod_item
});
impl_filho!(RegistroC880, {
    get_valor_item: vl_item, get_cst_pis: cst_pis, get_cst_cofins: cst_cofins, get_cfop: cfop,
    get_valor_pis: vl_pis, get_valor_cofins: vl_cofins
});

// Bloco D
impl_dopai!(RegistroD100, {
    get_cod_part: cod_part, get_cod_mod: cod_mod,
    get_num_doc: num_doc, get_chave: chv_cte,
    get_dt_emissao: dt_doc, get_dt_entrada: dt_a_p,
    get_valor_bc_icms: vl_bc_icms, get_valor_icms: vl_icms, get_cod_cta: cod_cta,
});
impl_filho!(RegistroD101, {
    get_nat_bc_cred: nat_bc_cred, get_valor_item: vl_item, get_cst_pis: cst_pis,
    get_aliq_pis: aliq_pis, get_valor_pis: vl_pis, get_valor_bc_pis: vl_bc_pis,
    get_cod_cta: cod_cta
});
impl_filho!(RegistroD105, {
    get_nat_bc_cred: nat_bc_cred, get_valor_item: vl_item, get_cst_cofins: cst_cofins,
    get_aliq_cofins: aliq_cofins, get_valor_cofins: vl_cofins, get_valor_bc_cofins: vl_bc_cofins,
    get_cod_cta: cod_cta
});

impl_dopai!(RegistroD200, { get_dt_emissao: dt_ref, get_cod_mod: cod_mod, get_num_doc: num_doc_ini });
impl_filho!(RegistroD201, {
    get_valor_item: vl_item, get_cst_pis: cst_pis, get_aliq_pis: aliq_pis, get_valor_pis: vl_pis
});
impl_filho!(RegistroD205, {
    get_valor_item: vl_item, get_cst_cofins: cst_cofins, get_aliq_cofins: aliq_cofins,
    get_valor_cofins: vl_cofins, get_valor_bc_cofins: vl_bc_cofins
});

impl_filho!(RegistroD300, {
    get_valor_item: vl_doc, get_cst_pis: cst_pis, get_cst_cofins: cst_cofins,
    get_aliq_pis: aliq_pis, get_valor_pis: vl_pis, get_aliq_cofins: aliq_cofins,
    get_valor_cofins: vl_cofins, get_valor_bc_cofins: vl_bc_cofins
});
impl_filho!(RegistroD350, {
    get_valor_item: vl_brt, get_cst_pis: cst_pis, get_cst_cofins: cst_cofins, get_aliq_pis: aliq_pis,
    get_valor_pis: vl_pis, get_aliq_cofins: aliq_cofins, get_valor_cofins: vl_cofins,
    get_valor_bc_cofins: vl_bc_cofins
});

impl_dopai!(RegistroD500, { get_dt_emissao: dt_a_p, get_cod_part: cod_part, get_cod_mod: cod_mod });
impl_filho!(RegistroD501, {
    get_valor_item: vl_item, get_cst_pis: cst_pis, get_aliq_pis: aliq_pis, get_valor_pis: vl_pis
});
impl_filho!(RegistroD505, {
    get_valor_item: vl_item, get_cst_cofins: cst_cofins, get_aliq_cofins: aliq_cofins,
    get_valor_cofins: vl_cofins, get_valor_bc_cofins: vl_bc_cofins
});

impl_dopai!(RegistroD600, { get_dt_emissao: dt_doc_ini, get_cod_mod: cod_mod });
impl_filho!(RegistroD601, {
    get_valor_item: vl_item, get_cst_pis: cst_pis, get_aliq_pis: aliq_pis, get_valor_pis: vl_pis
});
impl_filho!(RegistroD605, {
    get_valor_item: vl_item, get_cst_cofins: cst_cofins, get_aliq_cofins: aliq_cofins,
    get_valor_cofins: vl_cofins, get_valor_bc_cofins: vl_bc_cofins
});

// Bloco F
// F100 é Híbrido: tem dados de cabeçalho e de item na mesma linha
impl_filho!(RegistroF100, {
    get_cod_part: cod_part, get_cod_item: cod_item, get_dt_emissao: dt_oper, get_valor_item: vl_oper,

    get_cst_pis: cst_pis, get_valor_bc_pis: vl_bc_pis,
    get_aliq_pis: aliq_pis, get_valor_pis: vl_pis,

    get_cst_cofins: cst_cofins, get_valor_bc_cofins: vl_bc_cofins, get_aliq_cofins: aliq_cofins,
    get_valor_cofins: vl_cofins,

    get_nat_bc_cred: nat_bc_cred, get_ind_orig_cred: ind_orig_cred,
    get_cod_cta: cod_cta, get_descr_item: desc_doc_oper,
});

impl_filho!(RegistroF120, {
    get_valor_item: vl_oper_dep, get_descr_item: desc_bem_imob,
    get_cst_pis: cst_pis, get_cst_cofins: cst_cofins, get_aliq_pis: aliq_pis,
    get_valor_pis: vl_pis, get_aliq_cofins: aliq_cofins, get_valor_cofins: vl_cofins,
    get_valor_bc_cofins: vl_bc_cofins, get_descr_compl: desc_bem_imob
});

impl_filho!(RegistroF130, {
    get_nat_bc_cred: nat_bc_cred, get_cod_cta: cod_cta,
    get_valor_item: vl_bc_cofins, get_descr_item: desc_bem_imob,
    get_cst_pis: cst_pis, get_cst_cofins: cst_cofins, get_aliq_pis: aliq_pis,
    get_valor_pis: vl_pis, get_aliq_cofins: aliq_cofins, get_valor_cofins: vl_cofins,
    get_valor_bc_cofins: vl_bc_cofins, get_descr_compl: desc_bem_imob
});

impl_filho!(RegistroF150, {
    get_nat_bc_cred: nat_bc_cred, get_cod_cta: cod_cta,
    get_valor_item: vl_tot_est, get_descr_item: desc_est, get_cst_pis: cst_pis,
    get_cst_cofins: cst_cofins, get_aliq_pis: aliq_pis, get_valor_pis: vl_cred_pis,
    get_aliq_cofins: aliq_cofins, get_valor_cofins: vl_cred_cofins,
    get_valor_bc_cofins: vl_bc_est, get_descr_compl: desc_est
});

impl_filho!(RegistroF200, {
    get_part_override: cpf_cnpj_adqu,
    get_dt_emissao: dt_oper, get_valor_item: vl_tot_rec, get_info_compl: inf_comp,
    get_cod_part: cpf_cnpj_adqu, get_cst_pis: cst_pis, get_cst_cofins: cst_cofins,
    get_aliq_pis: aliq_pis, get_valor_pis: vl_pis, get_aliq_cofins: aliq_cofins,
    get_valor_cofins: vl_cofins, get_valor_bc_cofins: vl_bc_cofins
});

impl_filho!(RegistroF205, {
    get_valor_item: vl_cus_inc_per_esc,
    get_cst_pis: cst_pis, get_cst_cofins: cst_cofins,
    get_aliq_pis: aliq_pis, get_aliq_cofins: aliq_cofins,
});
impl_filho!(RegistroF210, {
    get_valor_item: vl_cus_orc,
    get_cst_pis: cst_pis, get_cst_cofins: cst_cofins,
    get_aliq_pis: aliq_pis, get_aliq_cofins: aliq_cofins,
});

impl_filho!(RegistroF500, {
    get_valor_item: vl_rec_caixa, get_info_compl: info_compl,
    get_cst_pis: cst_pis, get_cst_cofins: cst_cofins,
    get_aliq_pis: aliq_pis, get_aliq_cofins: aliq_cofins,
    get_valor_pis: vl_pis, get_valor_cofins: vl_cofins,
    get_valor_bc_pis: vl_bc_pis, get_valor_bc_cofins: vl_bc_cofins,
});
impl_filho!(RegistroF510, {
    get_valor_item: vl_rec_caixa, get_info_compl: info_compl,
    get_cst_pis: cst_pis, get_cst_cofins: cst_cofins,
    get_valor_pis: vl_pis, get_valor_cofins: vl_cofins,
    get_cod_mod: cod_mod, get_cfop: cfop, get_cod_cta: cod_cta,
});
impl_filho!(RegistroF525, {
    get_valor_item: vl_rec, get_info_compl: info_compl,
    get_cst_pis: cst_pis, get_cst_cofins: cst_cofins,
    get_cod_cta: cod_cta,
});
impl_filho!(RegistroF550, {
    get_valor_item: vl_rec_comp, get_info_compl: info_compl,
    get_cst_pis: cst_pis, get_cst_cofins: cst_cofins,
    get_aliq_pis: aliq_pis, get_aliq_cofins: aliq_cofins,
    get_valor_pis: vl_pis, get_valor_cofins: vl_cofins,
    get_valor_bc_pis: vl_bc_pis, get_valor_bc_cofins: vl_bc_cofins,
    get_cod_mod: cod_mod, get_cfop: cfop, get_cod_cta: cod_cta,
});
impl_filho!(RegistroF560, {
    get_valor_item: vl_rec_comp, get_info_compl: info_compl,
    get_cst_pis: cst_pis, get_cst_cofins: cst_cofins,
    get_valor_pis: vl_pis, get_valor_cofins: vl_cofins,
    get_cod_mod: cod_mod, get_cfop: cfop, get_cod_cta: cod_cta,
});

// Bloco I
impl_filho!(RegistroI100, {
    get_valor_item: vl_rec, get_info_compl: info_compl, get_cst_pis: cst_pis_cofins,
    get_cst_cofins: cst_pis_cofins, get_aliq_pis: aliq_pis, get_valor_pis: vl_pis,
    get_aliq_cofins: aliq_cofins, get_valor_cofins: vl_cofins, get_valor_bc_cofins: vl_bc_cofins
});

// Bloco M (Campos Específicos em M505)

//impl_filho!(RegistroM500, { get_cod_cred: cod_cred, get_aliq_cofins: aliq_cofins, get_valor_bc_cofins: vl_bc_cofins });
impl_filho!(RegistroM105, {
    get_cst_pis: cst_pis, get_valor_bc_pis: vl_bc_pis,
    get_nat_bc_cred: nat_bc_cred, get_info_compl: desc_cred,
});
impl_filho!(RegistroM505, {
    get_cst_cofins: cst_cofins, get_valor_bc_cofins: vl_bc_cofins,
    get_nat_bc_cred: nat_bc_cred, get_info_compl: desc_cred,
});

// Bloco 1
impl_filho!(Registro1100, { get_cod_cred: cod_cred });
impl_filho!(Registro1500, { get_cod_cred: cod_cred });

// ============================================================================
// SEÇÃO 5: CORRELATION MANAGER
// Implementa lógica rigorosa de "dispatch_table.rs" (Weak vs Strong Key)
// Gerencia a lógica de correlação PIS <-> COFINS com cache de alta performance
// ============================================================================

// MEMORY OPTIMIZATION: Usar Arc<str> nas chaves reduz drasticamente a alocação
// pois CSTs, CFOPs e Participantes são altamente repetitivos.
type WeakKey = (u16, Decimal);
type StrongKey = (u16, Decimal, Option<u16>, Option<Arc<str>>);
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

    /// Helper privado.
    /// Retorna Option<StrongKey> apenas se houver contexto adicional (CFOP ou Part).
    fn make_strong_key(
        cst: u16,
        val: Decimal,
        cfop: Option<u16>,
        part: Option<&str>,
    ) -> Option<StrongKey> {
        // filter + map é a forma perfeita de transformar Option<&str> vazio em None
        let participant = part.filter(|s| !s.is_empty()).map(Arc::from);

        if cfop.is_some() || participant.is_some() {
            Some((cst, val, cfop, participant))
        } else {
            None
        }
    }

    /// Armazena dados de PIS.
    fn store(
        &mut self,
        cst: Option<u16>,
        vl_item: Option<Decimal>,
        aliq_pis: Option<Decimal>,
        vl_pis: Option<Decimal>,
        cfop: Option<u16>,
        part: Option<&str>,
    ) {
        // Pattern matching em tupla para garantir que todos os dados obrigatórios existem
        if let (Some(c), Some(v_item), Some(a_pis), Some(v_pis)) = (cst, vl_item, aliq_pis, vl_pis)
        {
            let data_pis = (
                a_pis.to_f64().unwrap_or_default(),
                v_pis.to_f64().unwrap_or_default(),
            );

            // 1. Armazena na Cache Fraca
            // Clone do Arc é barato (apenas incrementa contador)
            self.weak_cache.insert((c, v_item), data_pis);

            // 2. Armazena na Cache Forte (se houver contexto)
            // Move cst_arc (sem clone extra) se possível, ou usa o clone anterior
            if let Some(strong_key) = Self::make_strong_key(c, v_item, cfop, part) {
                self.strong_cache.insert(strong_key, data_pis);
            }
        }
    }

    /// Resolve PIS baseado em dados de COFINS.
    /// Prioridade: Forte (com contexto) -> Fraca (apenas valores).
    fn resolve(
        &self,
        cst: Option<u16>,
        val_item: Option<Decimal>,
        cfop: Option<u16>,
        part: Option<&str>,
    ) -> Option<PisData> {
        let (c, val) = cst.zip(val_item)?;

        // 1. Tenta Chave Forte
        if let Some(data_pis) =
            Self::make_strong_key(c, val, cfop, part).and_then(|key| self.strong_cache.get(&key))
        {
            Some(*data_pis)
        } else {
            // 2. Fallback: Chave Fraca
            self.weak_cache.get(&(c, val)).copied()
        }
    }
}

// ============================================================================
// SEÇÃO 6: BUILDER PARA DocsFiscaisNew
// Padrão Builder para construção de DocsFiscaisNew, aplicando regras de negócio
// e recuperando dados do Contexto Global.
// ============================================================================

// ============================================================================
// SEÇÃO 6: BUILDER PARA DocsFiscaisNew
// Padrão Builder Otimizado: Zero-Cost Abstractions & Functional Style
// ============================================================================

/// Estrutura auxiliar para transportar dados do Pai de forma legível.
/// Adicionado `Copy` e `Clone` para armazenamento eficiente (stack-only).
#[derive(Debug, Default, Clone, Copy)]
struct ParentHeader<'a> {
    dt_emissao: Option<NaiveDate>,
    dt_entrada: Option<NaiveDate>,
    chave: Option<&'a str>,
    cod_cta: Option<&'a str>,
    cod_item: Option<&'a str>,
    cod_mod: Option<&'a str>,
    cod_ncm: Option<&'a str>,
    cod_part: Option<&'a str>,
    num_doc: Option<&'a str>,
    vl_bc_icms: Option<Decimal>,
    vl_icms: Option<Decimal>,
}

impl<'a> ParentHeader<'a> {
    /// Extrair dados de RegistroPai
    fn from_record<P>(pai: Option<&'a P>) -> Self
    where
        P: RegistroPai + ?Sized,
    {
        // Se pai for None, retorna Default (Zero Cost)
        // Se pai existir, mapeia os campos
        pai.map(|p| Self {
            dt_emissao: p.get_dt_emissao(),
            dt_entrada: p.get_dt_entrada(),
            chave: p.get_chave(),
            cod_cta: p.get_cod_cta(),
            cod_item: p.get_cod_item(),
            cod_mod: p.get_cod_mod(),
            cod_ncm: p.get_cod_ncm(),
            cod_part: p.get_cod_part(),
            num_doc: p.get_num_doc(),
            vl_bc_icms: p.get_valor_bc_icms(),
            vl_icms: p.get_valor_icms(),
        })
        .unwrap_or_default()
    }
}

#[derive(Clone)]
struct DocsBuilder<'a> {
    header: ParentHeader<'a>, // [STACK] ~100 bytes (apenas ponteiros e números)
    ctx: &'a SpedContext,     // [STACK] 8 bytes (ponteiro)
    doc: DocsFiscaisNew,      // [STACK] Estrutura contendo Arcs (ponteiros inteligentes)
}

impl<'a> DocsBuilder<'a> {
    fn new(
        ctx: &'a SpedContext,
        registro: &str,
        line_num: usize,
        current_cnpj: Option<&str>,
    ) -> Self {
        // Zero-copy se CNPJ não mudar
        let estabelecimento_cnpj = current_cnpj
            .map(Arc::from)
            .unwrap_or_else(|| ctx.estabelecimento_cnpj.clone());

        let estabelecimento_nome = ctx
            .estabelecimentos
            // Transforma Arc<str> em &str sem alocar
            .get(estabelecimento_cnpj.as_ref())
            // Clona o Arc<str> (apenas incrementa contador, nanosegundos)
            .cloned()
            // Se não achar, usa o fallback (também Arc clone barato)
            .unwrap_or_else(|| ctx.estabelecimento_nome.clone());

        let doc = DocsFiscaisNew {
            linhas: 1,
            arquivo_efd: ctx.arquivo_efd.clone(),
            num_linha_efd: Some(line_num),
            registro: Arc::from(registro),
            estabelecimento_cnpj,
            estabelecimento_nome,
            periodo_de_apuracao: ctx.periodo_de_apuracao,
            ano: ctx.periodo_de_apuracao.map(|d| d.year()),
            mes: ctx
                .periodo_de_apuracao
                .and_then(|d| MesesDoAno::try_from(d.month()).ok()),
            trimestre: ctx.periodo_de_apuracao.map(|d| d.quarter()),
            ..Default::default()
        };

        Self {
            header: ParentHeader::default(), // Inicia com header vazio
            ctx,
            doc,
        }
    }

    /// Constrói o documento de forma fluente.
    /// Extrai os dados do Pai uma única vez e os propaga.
    /// O Header é calculado aqui e movido para dentro do Builder.
    fn from_child_and_parent<F, P>(
        ctx: &'a SpedContext,
        filho: &F,
        pai: Option<&'a P>,
        current_cnpj: Option<&str>,
    ) -> Self
    where
        F: RegistroFilho + ?Sized,
        P: RegistroPai + ?Sized,
    {
        // 1. Criação: Instancia o builder com dados básicos (Contexto Global)
        // O header inicia vazio (Default) para economizar alocação prévia.
        let mut builder = Self::new(
            ctx,
            filho.registro_name(),
            filho.line_number(),
            current_cnpj,
        );

        // 2. Contexto: Extrai dados do Pai e move para dentro do Builder (Zero-Copy)
        // Isso centraliza as referências do pai em um único struct otimizado (Stack).
        builder.header = ParentHeader::from_record(pai);

        // 3. Preenchimento: Pipeline funcional (Fluent API)
        // Cada etapa consome o builder, aplica regras de negócio e o retorna modificado.
        builder
            .with_header(filho) // Mescla dados de cabeçalho (Datas, Chaves)
            .with_itens_and_participant(filho) // Resolve Itens e Participante
            .with_values_and_classification(filho) // Aplica valores e CSTs
    }

    fn from_child<F>(ctx: &'a SpedContext, reg: &F, current_cnpj: Option<&str>) -> Self
    where
        F: RegistroFilho + ?Sized,
    {
        // Passa None como pai, gerando um ParentHeader default internamente
        Self::from_child_and_parent(ctx, reg, None::<&()>, current_cnpj)
    }

    // --- Fases de Construção ---

    fn with_header<F>(mut self, filho: &F) -> Self
    where
        F: RegistroFilho + ?Sized,
    {
        // Informações do Filho prevalesce sobre informações do Pai
        self.doc.data_emissao = filho.get_dt_emissao().or(self.header.dt_emissao);
        self.doc.data_entrada = filho.get_dt_entrada().or(self.header.dt_entrada);

        self.doc.chave_doc = self.header.chave.unwrap_or_default().into();
        self.doc.cod_ncm = self.header.cod_ncm.unwrap_or_default().into();
        self.doc.num_doc = self.header.num_doc.parse_opt();

        self.doc.valor_bc_icms = filho
            .get_valor_bc_icms()
            .or(self.header.vl_bc_icms)
            .to_f64_opt();

        self.doc.valor_icms = filho.get_valor_icms().or(self.header.vl_icms).to_f64_opt();

        if let Some(m) = self.header.cod_mod {
            self.doc.modelo_doc_fiscal = obter_modelo_do_documento_fiscal(m).into();
        }
        self
    }

    fn with_itens_and_participant<F>(mut self, filho: &F) -> Self
    where
        F: RegistroFilho + ?Sized,
    {
        // 1. Resolve Itens e Produto
        let cod_item = self.header.cod_item.or_else(|| filho.get_cod_item());
        self.apply_itens_info(cod_item);

        // 2. Resolve Participante (Lógica de precedência clara)
        let cod_part = filho
            .get_part_override() // 1. Override do Filho
            .or_else(|| filho.get_cod_part()) // 2. Normal do Filho
            .or(self.header.cod_part); // 3. Herança do Pai

        self.apply_participant_info(cod_part);

        self
    }

    /// Aplica dados do produto se o código for válido
    fn apply_itens_info(&mut self, cod_item: Option<&str>) {
        // Encadeamento funcional com if-let-chain (Rust moderno)
        if let Some(info) = cod_item.and_then(|c| self.ctx.produtos.get(c)) {
            // Só aplica NCM se estiver vazio (regra de negócio preservada)
            if self.doc.cod_ncm.is_empty() {
                self.doc.cod_ncm = info.get("COD_NCM").cloned().unwrap_or_default();
            }

            // map() é ótimo para transformações opcionais
            if let Some(tipo) = info.get("TIPO_ITEM") {
                self.doc.tipo_item = obter_tipo_do_item(tipo).into();
            }

            if let Some(desc) = info.get("DESCR_ITEM") {
                self.doc.descr_item = Self::to_upper_arc(desc);
            }
        }
    }

    /// Aplica dados do participante
    fn apply_participant_info(&mut self, cod_part: Option<&str>) {
        // filter: Ignora strings vazias
        let Some(cod) = cod_part.filter(|s| !s.is_empty()) else {
            return;
        };

        if let Some(hash) = self.ctx.participantes.get(cod) {
            // Zero-copy clones
            self.doc.participante_cnpj = hash.get("CNPJ").cloned().unwrap_or_default();
            self.doc.participante_cpf = hash.get("CPF").cloned().unwrap_or_default();
            self.doc.participante_nome = hash.get("NOME").cloned().unwrap_or_default();
        } else {
            // Fallback (CPF/CNPJ direto no código)
            match cod.len() {
                14 => {
                    self.doc.participante_cnpj = cod.into();
                    self.doc.participante_nome = self.ctx.obter_nome_participante(Some(cod), None);
                }
                11 => {
                    self.doc.participante_cpf = cod.into();
                    self.doc.participante_nome = self.ctx.obter_nome_participante(None, Some(cod));
                }
                _ => {}
            }
        }
    }

    fn with_values_and_classification<F>(mut self, filho: &F) -> Self
    where
        F: RegistroFilho + ?Sized,
    {
        // Se houver descrição complementar, ela tem precedência e deve ser uppercase
        if let Some(compl) = filho.get_descr_compl().map(Self::to_upper_arc) {
            self.doc.descr_item = compl;
        }

        // 1. Identificadores e Classificação
        self.doc.num_item = filho.get_num_item();
        self.doc.cst = filho.get_cst_cofins();
        self.doc.cfop = filho.get_cfop();
        self.doc.natureza_bc = filho.get_nat_bc_cred().parse_opt();
        self.doc.indicador_de_origem = filho.get_ind_orig_cred().parse_opt();

        // 2. Valores Base
        self.doc.valor_item = filho.get_valor_item().to_f64_opt();
        self.doc.valor_bc = filho.get_valor_bc_cofins().to_f64_opt();

        // 3. Tributos
        self.doc.aliq_pis = filho.get_aliq_pis().to_f64_opt();
        self.doc.valor_pis = filho.get_valor_pis().to_f64_opt();
        self.doc.aliq_cofins = filho.get_aliq_cofins().to_f64_opt();
        self.doc.valor_cofins = filho.get_valor_cofins().to_f64_opt();
        self.doc.valor_iss = filho.get_valor_iss().to_f64_opt();
        self.doc.aliq_icms = filho.get_aliq_icms().to_f64_opt();

        // Lookup de Natureza
        if let Some(desc) = filho
            .get_cod_nat()
            .and_then(|c| self.ctx.nat_operacao.get(c))
        {
            self.doc.nat_operacao = desc.clone();
        }

        // Lookup Contábil
        // Prioridade das informações: Filho -> Pai
        let codigo_da_conta = filho.get_cod_cta().or(self.header.cod_cta);
        self.apply_account_name(codigo_da_conta);

        self
    }

    /// Helper para buscar e aplicar nome da conta
    fn apply_account_name(&mut self, cod_cta: Option<&str>) {
        if let Some(nome) = cod_cta
            .and_then(|c| self.ctx.contabil.get(c))
            .and_then(|h| h.get("NOME_CTA"))
        {
            self.doc.nome_da_conta = nome.clone();
        }
    }

    // Helper para converter string para Arc<str> uppercase de forma eficiente
    // Só aloca nova string se houver alguma letra minúscula.
    // "NOTA 123" -> Retorna Arc(original) (Zero Copy)
    // "Nota 123" -> Retorna Arc("NOTA 123") (Alocação necessária)
    fn to_upper_arc(s: &str) -> Arc<str> {
        if s.chars().any(|c| c.is_lowercase()) {
            // Aloca nova string apenas se necessário
            Arc::from(s.to_uppercase().as_str())
        } else {
            // Zero-allocation (apenas envolve o ponteiro)
            Arc::from(s)
        }
    }

    fn resolve_pis_correlation<F>(mut self, manager: &CorrelationManager, filho: &F) -> Self
    where
        F: RegistroFilho + ?Sized,
    {
        // Se já tem valores, retorna cedo
        if self.doc.valor_pis.is_some_and(|p| p.eh_maior_que_zero())
            && self.doc.aliq_pis.is_some_and(|p| p.eh_maior_que_zero())
        {
            return self;
        }

        let cod_participante = filho.get_part_override().or(self.header.cod_part);

        if let Some((aliq, val)) = manager.resolve(
            filho.get_cst_cofins(),
            filho.get_valor_item(),
            filho.get_cfop(),
            cod_participante,
        ) {
            self.doc.aliq_pis = Some(aliq);
            self.doc.valor_pis = Some(val);
        } else if let Some(aliq_cof) = self.doc.aliq_cofins {
            if aliq_cof.eh_igual(ALIQ_BASICA_COF) {
                self.doc.aliq_pis = Some(ALIQ_BASICA_PIS);
            } else if aliq_cof.eh_igual(3.0) {
                self.doc.aliq_pis = Some(0.65);
            }
        }
        self
    }

    fn resolve_tipo_de_operacao(mut self) -> Self {
        if self.doc.tipo_de_operacao.is_none() {
            self.doc.tipo_de_operacao = obter_tipo_operacao(self.doc.cst);
        }
        self
    }

    fn resolve_natureza_bc(mut self) -> Self {
        if self.doc.natureza_bc.is_none() {
            self.doc.natureza_bc = obter_cod_da_natureza_da_bc(&self.doc.cfop, self.doc.cst);
        }
        self
    }

    fn resolve_indicador_de_origem(mut self) -> Self {
        // Uso do helper criado na Proposta 1
        self.doc.indicador_de_origem = self.doc.indicador_de_origem.or_else(|| {
            if is_importacao(self.doc.cfop) {
                Some(IndicadorOrigem::Importacao)
            } else {
                Some(IndicadorOrigem::MercadoInterno)
            }
        });
        self
    }

    fn resolve_tipo_de_credito(mut self) -> Self {
        if self.doc.tipo_de_credito.is_none() {
            let credito = determinar_tipo_de_credito(
                self.doc.cst,
                self.doc.aliq_pis,
                self.doc.aliq_cofins,
                self.doc.cod_credito,
                self.doc.cfop,
            );

            self.doc.tipo_de_credito = credito;

            if matches!(credito, Some(TipoDeCredito::Importacao)) {
                self.doc.indicador_de_origem = Some(IndicadorOrigem::Importacao);
            }
        }
        self
    }

    fn build(self) -> DocsFiscaisNew {
        let mut builder = self
            .resolve_tipo_de_operacao()
            .resolve_natureza_bc()
            .resolve_indicador_de_origem()
            .resolve_tipo_de_credito();

        // Atenção: O método format() em DocsFiscais deve estar preparado para lidar
        // com campos Arc<str>. Se o método original tentava mutar (push_str/insert)
        // nestes campos, ele precisará ser ajustado em docs_fiscais.rs.
        builder.doc.format();

        builder.doc
    }
}

// ============================================================================
// SEÇÃO 7: LÓGICA DE NEGÓCIO AUXILIAR
// ============================================================================

/// Verifica se um CFOP corresponde a uma operação de Importação.
/// Centraliza a regra "3000..=3999".
#[inline]
fn is_importacao(cfop: Option<u16>) -> bool {
    cfop.is_some_and(|c| (3000..=3999).contains(&c))
}

/// Deduz o Tipo de Operação baseado no CST.
/// Remove os "Magic Numbers" do método build.
fn obter_tipo_operacao(cst: Option<u16>) -> Option<TipoOperacao> {
    match cst {
        Some(1..=49) => Some(TipoOperacao::Saida),
        Some(50..=99) => Some(TipoOperacao::Entrada),
        _ => None,
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

    let existe_aliquota_positiva = aliq_pis.is_some_and(|p| p.eh_maior_que_zero())
        || aliq_cofins.is_some_and(|c| c.eh_maior_que_zero());

    // Pré-condição: Pelo menos uma alíquota deve ser positiva
    if !existe_aliquota_positiva {
        return None;
    }

    match (is_importacao(cfop), cst_cofins) {
        // Regra A: Importação
        (true, _) => Some(TipoDeCredito::Importacao),

        // Regra B: Mercado Interno + CST Básico (50-56)
        (false, Some(50..=56)) => {
            // Verifica se as alíquotas correspondem exatamente ao básico (1.65% e 7.6%)
            let pis_basico = aliq_pis.is_some_and(|p| p.eh_igual(ALIQ_BASICA_PIS));
            let cof_basico = aliq_cofins.is_some_and(|c| c.eh_igual(ALIQ_BASICA_COF));

            if pis_basico && cof_basico {
                Some(TipoDeCredito::AliquotaBasica)
            } else {
                Some(TipoDeCredito::AliquotasDiferenciadas)
            }
        }

        // Regra C: Mercado Interno + Crédito Presumido (CST 60-66)
        (false, Some(60..=66)) => {
            // A função `cred_presumido` deve conter a lógica específica da agroindústria/transportes
            if cred_presumido(aliq_pis, aliq_cofins) {
                Some(TipoDeCredito::PresumidoAgroindustria)
            } else {
                Some(TipoDeCredito::OutrosCreditosPresumidos)
            }
        }

        _ => None,
    }
}

// ============================================================================
// SEÇÃO 8: PROCESSADORES DE BLOCO
// Iteradores paralelos chamam esta função para processar blocos inteiros.
// ============================================================================

pub fn process_block_lines(bloco: char, file: &SpedFile, ctx: &SpedContext) -> Vec<DocsFiscaisNew> {
    let records = match file.obter_bloco_option(bloco) {
        Some(l) => l,
        None => return Vec::new(),
    };
    let mut docs = Vec::with_capacity(records.len());

    match bloco {
        'A' => BlocoA::default().process(records, ctx, &mut docs),
        'C' => BlocoC::default().process(records, ctx, &mut docs),
        'D' => BlocoD::default().process(records, ctx, &mut docs),
        'F' => BlocoF::default().process(records, ctx, &mut docs),
        'I' => BlocoI::default().process(records, ctx, &mut docs),
        'M' => BlocoM::default().process(records, ctx, &mut docs),
        '1' => Bloco1::default().process(records, ctx, &mut docs),
        _ => {}
    }
    docs
}

// --- Bloco A (Serviços) ---
#[derive(Default)]
struct BlocoA<'a> {
    a100: Option<&'a RegistroA100>,
    current_cnpj: Option<&'a str>,
}
impl<'a> BlocoA<'a> {
    fn process(
        &mut self,
        records: &'a [SpedRecord],
        ctx: &SpedContext,
        docs: &mut Vec<DocsFiscaisNew>,
    ) {
        for record in records {
            let SpedRecord::Generic(generic) = record else {
                continue;
            };

            match generic.registro_name() {
                "A010" => capture_cnpj!(self.current_cnpj, record, RegistroA010),
                "A100" => self.a100 = record.downcast_ref().ok(),
                "A170" => process_child_and_parent!(
                    docs,
                    ctx,
                    self.current_cnpj,
                    record,
                    RegistroA170,
                    self.a100
                ),
                _ => {}
            }
        }
    }
}

// --- Bloco C (Mercadorias - Complexo) ---
#[derive(Default)]
struct BlocoC<'a> {
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
    correlacao: CorrelationManager,
    c195_idxs: Vec<usize>,
    current_cnpj: Option<&'a str>,
}
impl<'a> BlocoC<'a> {
    fn process(
        &mut self,
        records: &'a [SpedRecord],
        ctx: &SpedContext,
        docs: &mut Vec<DocsFiscaisNew>,
    ) {
        for record in records {
            let SpedRecord::Generic(generic) = record else {
                continue;
            };

            match generic.registro_name() {
                "C010" => capture_cnpj!(self.current_cnpj, record, RegistroC010),
                "C100" => {
                    self.c100 = record.downcast_ref().ok();
                    self.correlacao.clear();
                }

                "C170" => process_child_and_parent!(
                    docs,
                    ctx,
                    self.current_cnpj,
                    record,
                    RegistroC170,
                    self.c100
                ),
                "C175" => process_child_and_parent!(
                    docs,
                    ctx,
                    self.current_cnpj,
                    record,
                    RegistroC175,
                    self.c100
                ),
                "C180" => {
                    self.c180 = record.downcast_ref().ok();
                    self.correlacao.clear();
                }
                "C181" => store_pis!(self.correlacao, record, RegistroC181),
                "C185" => process_correlations!(
                    docs,
                    ctx,
                    self.current_cnpj,
                    &self.correlacao,
                    record,
                    RegistroC185,
                    self.c180
                ),
                "C190" => {
                    self.c190 = record.downcast_ref().ok();
                    self.correlacao.clear();
                    self.c195_idxs.clear();
                }
                "C191" => store_pis!(self.correlacao, record, RegistroC191),
                "C195" => {
                    if let (Ok(filho), Some(pai)) =
                        (record.downcast_ref::<RegistroC195>(), self.c190)
                    {
                        docs.push(
                            DocsBuilder::from_child_and_parent(
                                ctx,
                                filho,
                                Some(pai),
                                self.current_cnpj,
                            )
                            .resolve_pis_correlation(&self.correlacao, filho)
                            .build(),
                        );
                        self.c195_idxs.push(docs.len() - 1);
                    }
                }
                "C199" => {
                    if let Ok(r) = record.downcast_ref::<RegistroC199>()
                        && let Some(n) = &r.num_doc_imp
                    {
                        let t = format!("Número do documento de Importação: {}", n);
                        for &i in &self.c195_idxs {
                            if let Some(d) = docs.get_mut(i) {
                                d.complementar = format!("{} {}", d.complementar, t).into();
                            }
                        }
                    }
                }
                "C380" => {
                    self.c380 = record.downcast_ref().ok();
                    self.correlacao.clear();
                }
                "C381" => store_pis!(self.correlacao, record, RegistroC381),
                "C385" => process_correlations!(
                    docs,
                    ctx,
                    self.current_cnpj,
                    &self.correlacao,
                    record,
                    RegistroC385,
                    self.c380
                ),
                "C395" => self.c395 = record.downcast_ref().ok(),
                "C396" => process_child_and_parent!(
                    docs,
                    ctx,
                    self.current_cnpj,
                    record,
                    RegistroC396,
                    self.c395
                ),
                "C400" => {
                    self.c400 = record.downcast_ref().ok();
                    self.correlacao.clear();
                }
                "C405" => self.c405 = record.downcast_ref().ok(),
                "C481" => store_pis!(self.correlacao, record, RegistroC481),
                "C485" => process_correlations!(
                    docs,
                    ctx,
                    self.current_cnpj,
                    &self.correlacao,
                    record,
                    RegistroC485,
                    self.c405
                ),
                "C490" => {
                    self.c490 = record.downcast_ref().ok();
                    self.correlacao.clear();
                }
                "C491" => store_pis!(self.correlacao, record, RegistroC491),
                "C495" => process_correlations!(
                    docs,
                    ctx,
                    self.current_cnpj,
                    &self.correlacao,
                    record,
                    RegistroC495,
                    self.c490
                ),
                "C500" => {
                    self.c500 = record.downcast_ref().ok();
                    self.correlacao.clear();
                }
                "C501" => store_pis!(self.correlacao, record, RegistroC501),
                "C505" => process_correlations!(
                    docs,
                    ctx,
                    self.current_cnpj,
                    &self.correlacao,
                    record,
                    RegistroC505,
                    self.c500
                ),
                "C600" => {
                    self.c600 = record.downcast_ref().ok();
                    self.correlacao.clear();
                }
                "C601" => store_pis!(self.correlacao, record, RegistroC601),
                "C605" => process_correlations!(
                    docs,
                    ctx,
                    self.current_cnpj,
                    &self.correlacao,
                    record,
                    RegistroC605,
                    self.c600
                ),

                "C860" => self.c860 = record.downcast_ref().ok(),
                "C870" => process_child_and_parent!(
                    docs,
                    ctx,
                    self.current_cnpj,
                    record,
                    RegistroC870,
                    self.c860
                ),
                "C880" => process_only_child!(docs, ctx, self.current_cnpj, record, RegistroC880),
                _ => {}
            }
        }
    }
}

// --- Bloco D (Transportes) ---
#[derive(Default)]
struct BlocoD<'a> {
    d100: Option<&'a RegistroD100>,
    d200: Option<&'a RegistroD200>,
    d500: Option<&'a RegistroD500>,
    d600: Option<&'a RegistroD600>,
    correlacao: CorrelationManager,
    current_cnpj: Option<&'a str>,
}
impl<'a> BlocoD<'a> {
    fn process(
        &mut self,
        records: &'a [SpedRecord],
        ctx: &SpedContext,
        docs: &mut Vec<DocsFiscaisNew>,
    ) {
        for record in records {
            let SpedRecord::Generic(g) = record else {
                continue;
            };

            match g.registro_name() {
                "D010" => capture_cnpj!(self.current_cnpj, record, RegistroD010),
                "D100" => {
                    self.d100 = record.downcast_ref().ok();
                    self.correlacao.clear();
                }
                "D101" => store_pis!(self.correlacao, record, RegistroD101),
                "D105" => process_correlations!(
                    docs,
                    ctx,
                    self.current_cnpj,
                    &self.correlacao,
                    record,
                    RegistroD105,
                    self.d100
                ),

                "D200" => {
                    self.d200 = record.downcast_ref().ok();
                    self.correlacao.clear();
                }
                "D201" => store_pis!(self.correlacao, record, RegistroD201),
                "D205" => process_correlations!(
                    docs,
                    ctx,
                    self.current_cnpj,
                    &self.correlacao,
                    record,
                    RegistroD205,
                    self.d200
                ),

                "D300" => process_only_child!(docs, ctx, self.current_cnpj, record, RegistroD300),
                "D350" => process_only_child!(docs, ctx, self.current_cnpj, record, RegistroD350),

                "D500" => {
                    self.d500 = record.downcast_ref().ok();
                    self.correlacao.clear();
                }
                "D501" => store_pis!(self.correlacao, record, RegistroD501),
                "D505" => process_correlations!(
                    docs,
                    ctx,
                    self.current_cnpj,
                    &self.correlacao,
                    record,
                    RegistroD505,
                    self.d500
                ),
                "D600" => {
                    self.d600 = record.downcast_ref().ok();
                    self.correlacao.clear();
                }
                "D601" => store_pis!(self.correlacao, record, RegistroD601),
                "D605" => process_correlations!(
                    docs,
                    ctx,
                    self.current_cnpj,
                    &self.correlacao,
                    record,
                    RegistroD605,
                    self.d600
                ),
                _ => {}
            }
        }
    }
}

// --- Bloco F (Financeiro / Diversos) ---
// Utiliza intensivamente RegistroGeral para registros autônomos
#[derive(Default)]
struct BlocoF<'a> {
    current_cnpj: Option<&'a str>,
}
impl<'a> BlocoF<'a> {
    fn process(
        &mut self,
        records: &'a [SpedRecord],
        ctx: &SpedContext,
        docs: &mut Vec<DocsFiscaisNew>,
    ) {
        for record in records {
            let SpedRecord::Generic(g) = record else {
                continue;
            };

            match g.registro_name() {
                "F010" => capture_cnpj!(self.current_cnpj, record, RegistroF010),
                "F100" => process_only_child!(docs, ctx, self.current_cnpj, record, RegistroF100),
                "F120" => process_only_child!(docs, ctx, self.current_cnpj, record, RegistroF120),
                "F130" => process_only_child!(docs, ctx, self.current_cnpj, record, RegistroF130),
                "F150" => process_only_child!(docs, ctx, self.current_cnpj, record, RegistroF150),
                "F200" => process_only_child!(docs, ctx, self.current_cnpj, record, RegistroF200),
                "F205" => process_only_child!(docs, ctx, self.current_cnpj, record, RegistroF205),
                "F210" => process_only_child!(docs, ctx, self.current_cnpj, record, RegistroF210),
                "F500" => process_only_child!(docs, ctx, self.current_cnpj, record, RegistroF500),
                "F510" => process_only_child!(docs, ctx, self.current_cnpj, record, RegistroF510),
                "F525" => process_only_child!(docs, ctx, self.current_cnpj, record, RegistroF525),
                "F550" => process_only_child!(docs, ctx, self.current_cnpj, record, RegistroF550),
                "F560" => process_only_child!(docs, ctx, self.current_cnpj, record, RegistroF560),
                _ => {}
            }
        }
    }
}

// --- Bloco I (Pessoa Jurídica) ---
#[derive(Default)]
struct BlocoI<'a> {
    current_cnpj: Option<&'a str>,
}

impl<'a> BlocoI<'a> {
    fn process(
        &mut self,
        records: &'a [SpedRecord],
        ctx: &SpedContext,
        docs: &mut Vec<DocsFiscaisNew>,
    ) {
        for record in records {
            let SpedRecord::Generic(g) = record else {
                continue;
            };

            match g.registro_name() {
                "I010" => capture_cnpj!(self.current_cnpj, record, RegistroI010),
                "I100" => process_only_child!(docs, ctx, self.current_cnpj, record, RegistroI100),
                _ => {}
            }
        }
    }
}

type KeyM = (Option<u16>, Option<u16>, Option<u8>, Option<Decimal>);

// --- Bloco M (Apuração e Ajustes) ---
#[derive(Default)]
struct BlocoM<'a> {
    m100: Option<&'a RegistroM100>,
    m500: Option<&'a RegistroM500>,
    correlacao: HashMap<KeyM, Option<Decimal>>,
}
impl<'a> BlocoM<'a> {
    fn process(
        &mut self,
        records: &'a [SpedRecord],
        ctx: &SpedContext,
        docs: &mut Vec<DocsFiscaisNew>,
    ) {
        for record in records {
            let SpedRecord::Generic(g) = record else {
                continue;
            };

            match g.registro_name() {
                "M100" => {
                    if let Ok(r) = record.downcast_ref::<RegistroM100>() {
                        //self.correlacao = HashMap::new();
                        self.m100 = Some(r);
                        docs.extend(mappers::build_m100(r, ctx));
                    }
                }
                "M105" => {
                    if let (Ok(r), Some(pai)) = (record.downcast_ref::<RegistroM105>(), self.m100) {
                        let cod_cred: Option<u16> = pai.cod_cred;
                        let cst_pis: Option<u16> = r.cst_pis;
                        let nat_bc_cred: Option<u8> = r.nat_bc_cred.parse_opt();
                        let key = (cod_cred, cst_pis, nat_bc_cred, r.vl_bc_pis);

                        self.correlacao.insert(key, pai.aliq_pis);
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
                        let cod_cred: Option<u16> = p.cod_cred;
                        let cst_cofins: Option<u16> = r.cst_cofins;
                        let nat_bc_cred: Option<u8> = r.nat_bc_cred.parse_opt();
                        let key = (cod_cred, cst_cofins, nat_bc_cred, r.vl_bc_cofins);

                        // println!("correlacao: {:?}", self.correlacao);
                        let aliq_pis = match self.correlacao.get(&key) {
                            Some(a) => *a,
                            None => {
                                let msg = "Ausência de correlação entre as alíquotas de PIS/PASEP e COFINS";
                                eprintln!("{msg}");
                                continue;
                            }
                        };

                        let mut b = DocsBuilder::from_child(ctx, r, None);
                        b.doc.data_emissao = ctx.periodo_de_apuracao;
                        b.doc.cod_credito = p.cod_cred;
                        b.doc.aliq_cofins = p.aliq_cofins.to_f64_opt();
                        b.doc.natureza_bc = r.nat_bc_cred.parse_opt();
                        b.doc.tipo_de_operacao = Some(TipoOperacao::Detalhamento);

                        if aliq_pis.is_some() {
                            b.doc.aliq_pis = aliq_pis.to_f64_opt();
                        }
                        docs.push(b.build());
                    }
                }
                _ => {}
            }
        }
    }
}

// --- Bloco 1 (Controle) ---
#[derive(Default)]
struct Bloco1<'a> {
    current_cnpj: Option<&'a str>,
}

impl<'a> Bloco1<'a> {
    fn process(
        &mut self,
        records: &'a [SpedRecord],
        ctx: &SpedContext,
        docs: &mut Vec<DocsFiscaisNew>,
    ) {
        self.current_cnpj = None;
        for record in records {
            let SpedRecord::Generic(g) = record else {
                continue;
            };

            match g.registro_name() {
                "1100" => {
                    if let Ok(r) = record.downcast_ref::<Registro1100>()
                        && let Some(d) = mappers::build_ctrl_credito(
                            r.line_number,
                            "1100",
                            r.vl_cred_desc_efd,
                            r.get_cod_cred(),
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
                            r.get_cod_cred(),
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
}

// ============================================================================
// SEÇÃO 9: MAPPERS (Pure Logic)
// Lógica encapsulada para geração de múltiplos documentos ou regras complexas
// ============================================================================

mod mappers {
    use super::*;

    pub fn build_m100(reg: &RegistroM100, ctx: &SpedContext) -> Vec<DocsFiscaisNew> {
        let b = DocsBuilder::new(ctx, "M100", reg.line_number, None); // Builder base imutável

        // Configuração base (com clone barato pois usa Arc e Option<Copy>)
        let make_base = || {
            let mut doc = b.clone();
            doc.doc.data_emissao = ctx.periodo_de_apuracao;
            doc.doc.aliq_pis = reg.aliq_pis.to_f64_opt();
            doc.doc.valor_bc = reg.vl_bc_pis.to_f64_opt();
            doc.doc.cod_credito = reg.cod_cred;
            doc
        };

        // Gera os ajustes funcionalmente
        generate_adjustments(
            make_base(),
            reg.vl_ajus_acres,
            reg.vl_ajus_reduc,
            reg.vl_cred_desc,
        )
    }

    pub fn build_m500(reg: &RegistroM500, ctx: &SpedContext) -> Vec<DocsFiscaisNew> {
        let b = DocsBuilder::new(ctx, "M500", reg.line_number, None);

        let make_base = || {
            let mut doc = b.clone();
            doc.doc.data_emissao = ctx.periodo_de_apuracao;
            doc.doc.aliq_cofins = reg.aliq_cofins.to_f64_opt();
            doc.doc.valor_bc = reg.vl_bc_cofins.to_f64_opt();
            doc.doc.cod_credito = reg.cod_cred;
            doc
        };

        generate_adjustments(
            make_base(),
            reg.vl_ajus_acres,
            reg.vl_ajus_reduc,
            reg.vl_cred_desc,
        )
    }

    /// Gera vetor de documentos de ajuste de forma funcional
    fn generate_adjustments(
        base_builder: DocsBuilder,
        acres: Option<Decimal>,
        reduc: Option<Decimal>,
        desc: Option<Decimal>,
    ) -> Vec<DocsFiscaisNew> {
        // Vetor de tuplas: (Valor, Operação, Sinal)
        let adjustments = [
            (acres, TipoOperacao::AjusteAcrescimo, 1.0),
            (reduc, TipoOperacao::AjusteReducao, -1.0),
            (desc, TipoOperacao::DescontoNoPeriodo, -1.0),
        ];

        adjustments
            .into_iter()
            .filter_map(|(val_opt, op, signal)| {
                val_opt.filter(|v| v.eh_maior_que_zero()).map(|v| {
                    let mut b = base_builder.clone();
                    // Aqui finalizamos o build
                    b.doc.valor_item = Some(v.abs().to_f64().unwrap_or_default() * signal);
                    b.doc.tipo_de_operacao = Some(op);
                    b.build() // Chama o build final aqui
                })
            })
            .collect()
    }

    pub fn build_ctrl_credito(
        line: usize,
        reg_name: &str,
        vl_desc: Option<Decimal>,
        cod_cred: Option<u16>,
        per_apu: Option<&str>,
        ctx: &SpedContext,
    ) -> Option<DocsFiscaisNew> {
        let val = vl_desc.abs_f64();
        if val.eh_zero() {
            return None;
        }

        let mut b = DocsBuilder::new(ctx, reg_name, line, None);
        b.doc.valor_item = Some(-val);
        b.doc.tipo_de_operacao = Some(TipoOperacao::DescontoPosterior);
        b.doc.cod_credito = cod_cred;

        if let (Some(orig_str), Some(curr)) = (per_apu, ctx.periodo_de_apuracao)
            && orig_str.len() == 6
        {
            let m = orig_str[0..2].parse::<u32>().unwrap_or_default();
            let y = orig_str[2..6].parse::<i32>().unwrap_or_default();
            if let Some(orig) = NaiveDate::from_ymd_opt(y, m, 1)
                && orig != curr
            {
                b.doc.complementar = format!("ORIGEM DIFERENTE: {}", orig_str).into();
            }
        }
        Some(b.build())
    }
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//
//
// cargo test -- --help
// cargo test -- --nocapture
// cargo test -- --show-output

/// Run tests with:
/// cargo test -- --show-output info_new_tests
#[cfg(test)]
#[path = "../tests/info_new_tests.rs"]
mod info_new_tests;
