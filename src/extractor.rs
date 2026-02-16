use chrono::{Datelike, NaiveDate};
use log::{Level, log_enabled};
use rayon::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::{collections::HashMap, fmt::Write, str::FromStr, sync::Arc};

use crate::{
    ALIQ_BASICA_COF, ALIQ_BASICA_PIS, Bloco1, BlocoA, BlocoC, BlocoD, BlocoF, BlocoI, BlocoM,
    CSTOption, CodigoDoCredito, CodigoSituacaoTributaria, DECIMAL_ALIQ, DECIMAL_VALOR, DecimalExt,
    DocsFiscais, IndicadorDeOrigem, MesesDoAno, ModeloDocFiscal, NaturezaBaseCalculo, SpedContext,
    SpedFile, SpedRecordTrait, StringParser, TipoDeCredito, TipoDeOperacao, blocos::*,
    capture_cnpj, cred_presumido, impl_dopai, impl_filho, is_importacao, obter_natureza_da_bc,
    obter_pis_da_tabela_estatica, process_child_and_parent, process_correlations,
    process_only_child, store_pis,
};

const NUM_DE_ESTABELECIMENTOS: usize = 1;
const LIMITE_LINHAS: usize = 100_000; // Só ativa 2ª camada se houver > N documentos

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
// Cobrem TODOS os campos necessários para preencher DocsFiscais.
// ============================================================================

/// Representa um registro "Pai" (Cabeçalho) que fornece contexto para os "Filhos".
///
/// Ex: C100 (Nota) fornece dados para C170 (Itens).
pub trait RegistroPai {
    // --- Datas ---

    /// Data da Emissão do Documento Fiscal
    fn get_dt_emissao(&self) -> Option<NaiveDate> {
        None
    }
    /// Data da Entrada da Mercadoria ou Início
    /// da Execução do Serviço
    fn get_dt_entrada(&self) -> Option<NaiveDate> {
        None
    }

    // --- Identificadores e Códigos ---

    /// Código Fiscal de Operação e Prestação conforme
    /// tabela indicada no item 4.2.2
    fn get_cfop(&self) -> Option<u16> {
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
    /// Código da informação complementar do documento fiscal (campo 02 do Registro 0450)
    fn get_cod_inf(&self) -> Option<&str> {
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
    fn get_num_doc(&self) -> Option<usize> {
        None
    }

    // --- Valores de Impostos Retidos/Totais ---

    /// Valor do ISS.  Ver Registro A100
    fn get_valor_iss(&self) -> Option<Decimal> {
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

/// Implementação padrão para tipos nulos (registros sem pai/órfãos).
impl RegistroPai for () {} // Implementação para filhos órfãos

/// Registro "Filho" contendo detalhes do item e tributação (Ex: C170, C191).
pub trait RegistroFilho: SpedRecordTrait {
    // --- Temporal ---

    fn get_dt_emissao(&self) -> Option<NaiveDate> {
        None
    }
    fn get_dt_entrada(&self) -> Option<NaiveDate> {
        None
    }
    fn get_per_apu_cred(&self) -> Option<NaiveDate> {
        None
    }

    // Identificação por Código
    fn get_cod_cta(&self) -> Option<&str> {
        None
    }
    fn get_cod_cred(&self) -> Option<CodigoDoCredito> {
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
    // Número do documento fiscal
    fn get_num_doc(&self) -> Option<usize> {
        None
    }

    // Classificação Fiscal
    fn get_cst_pis(&self) -> Option<CodigoSituacaoTributaria> {
        None
    }
    fn get_cst_cofins(&self) -> Option<CodigoSituacaoTributaria> {
        None
    }
    fn get_cfop(&self) -> Option<u16> {
        None
    }
    fn get_nat_bc_cred(&self) -> Option<u16> {
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
    get_cod_part: cod_part, get_num_doc: num_doc, get_valor_iss: vl_iss
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
impl_dopai!(RegistroC110, {
    get_cod_inf: cod_inf
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
    get_cst_pis: cst_pis, get_cfop: cfop, get_valor_item: vl_item,
    get_valor_bc_pis: vl_bc_pis, get_aliq_pis: aliq_pis,
    get_valor_pis: vl_pis, get_cod_cta: cod_cta
});
impl_filho!(RegistroC185, {
    get_cst_cofins: cst_cofins, get_cfop: cfop, get_valor_item: vl_item,
    get_valor_bc_cofins: vl_bc_cofins, get_aliq_cofins: aliq_cofins,
    get_valor_cofins: vl_cofins, get_cod_cta: cod_cta
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
    get_num_doc: num_doc, get_valor_icms: vl_icms,
    get_cod_inf: cod_inf, get_chave: chv_doce,
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
    get_valor_bc_icms: vl_bc_icms, get_valor_icms: vl_icms,
    get_cod_inf: cod_inf, get_cod_cta: cod_cta,
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

impl_dopai!(RegistroD200, {
    get_dt_emissao: dt_ref, get_cod_mod: cod_mod, get_num_doc: num_doc_ini,
    get_cfop: cfop,
});
impl_filho!(RegistroD201, {
    get_valor_item: vl_item, get_cst_pis: cst_pis, get_aliq_pis: aliq_pis,
    get_valor_pis: vl_pis, get_cod_cta: cod_cta
});
impl_filho!(RegistroD205, {
    get_valor_item: vl_item, get_cst_cofins: cst_cofins, get_aliq_cofins: aliq_cofins,
    get_valor_cofins: vl_cofins, get_valor_bc_cofins: vl_bc_cofins, get_cod_cta: cod_cta
});

impl_filho!(RegistroD300, {
    get_dt_emissao: dt_ref, get_cfop: cfop, get_valor_item: vl_doc,
    get_cst_pis: cst_pis, get_cst_cofins: cst_cofins,
    get_aliq_pis: aliq_pis, get_aliq_cofins: aliq_cofins,
    get_valor_pis: vl_pis, get_valor_cofins: vl_cofins,
    get_valor_bc_cofins: vl_bc_cofins,
    get_cod_mod: cod_mod, get_cod_cta: cod_cta,
});
impl_filho!(RegistroD350, {
    get_valor_item: vl_brt, get_cst_pis: cst_pis, get_cst_cofins: cst_cofins, get_aliq_pis: aliq_pis,
    get_valor_pis: vl_pis, get_aliq_cofins: aliq_cofins, get_valor_cofins: vl_cofins,
    get_valor_bc_cofins: vl_bc_cofins
});

impl_dopai!(RegistroD500, {
    get_dt_emissao: dt_a_p, get_dt_entrada: dt_a_p, get_cod_part: cod_part,
    get_cod_mod: cod_mod, get_cod_inf: cod_inf,
});
impl_filho!(RegistroD501, {
    get_valor_item: vl_item, get_cst_pis: cst_pis, get_aliq_pis: aliq_pis, get_valor_pis: vl_pis,
    get_cod_cta: cod_cta
});
impl_filho!(RegistroD505, {
    get_valor_item: vl_item, get_cst_cofins: cst_cofins, get_aliq_cofins: aliq_cofins,
    get_valor_cofins: vl_cofins, get_valor_bc_cofins: vl_bc_cofins, get_cod_cta: cod_cta
});

impl_dopai!(RegistroD600, {
    get_dt_emissao: dt_doc_ini, get_cod_mod: cod_mod,
    get_valor_icms: vl_icms, get_valor_bc_icms: vl_bc_icms
});
impl_filho!(RegistroD601, {
    get_valor_item: vl_item, get_cst_pis: cst_pis, get_aliq_pis: aliq_pis,
    get_valor_pis: vl_pis, get_cod_cta: cod_cta
});
impl_filho!(RegistroD605, {
    get_valor_item: vl_item, get_cst_cofins: cst_cofins, get_aliq_cofins: aliq_cofins,
    get_valor_cofins: vl_cofins, get_valor_bc_cofins: vl_bc_cofins,  get_cod_cta: cod_cta
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
    get_cod_cta: cod_cta, get_descr_compl: desc_doc_oper,
});

impl_filho!(RegistroF120, {
    get_valor_item: vl_oper_dep, get_descr_item: desc_bem_imob, get_nat_bc_cred: nat_bc_cred,
    get_cst_pis: cst_pis, get_cst_cofins: cst_cofins, get_aliq_pis: aliq_pis,
    get_valor_pis: vl_pis, get_aliq_cofins: aliq_cofins, get_valor_cofins: vl_cofins,
    get_valor_bc_cofins: vl_bc_cofins, get_descr_compl: desc_bem_imob, get_cod_cta: cod_cta,
    get_ind_orig_cred: ind_orig_cred,
});

impl_filho!(RegistroF130, {
    get_dt_emissao: mes_oper_aquis,
    get_nat_bc_cred: nat_bc_cred, get_cod_cta: cod_cta,
    get_valor_item: vl_bc_cofins, get_descr_item: desc_bem_imob,
    get_cst_pis: cst_pis, get_cst_cofins: cst_cofins, get_aliq_pis: aliq_pis,
    get_valor_pis: vl_pis, get_aliq_cofins: aliq_cofins, get_valor_cofins: vl_cofins,
    get_valor_bc_cofins: vl_bc_cofins, get_descr_compl: desc_bem_imob,
    get_ind_orig_cred: ind_orig_cred,
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
    get_valor_pis: vl_pis, get_valor_cofins: vl_cofins, get_cod_cta: cod_cta,
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
    get_cod_cta: cod_cta, get_num_doc: num_doc,
    get_cod_item: cod_item, get_part_override: cnpj_cpf

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
impl_filho!(Registro1100, { get_cod_cred: cod_cred, get_per_apu_cred: per_apu_cred });
impl_filho!(Registro1500, { get_cod_cred: cod_cred, get_per_apu_cred: per_apu_cred });

// ============================================================================
// SEÇÃO 5: CORRELATION MANAGER (REFACTOR)
// Gerencia a lógica de correlação PIS <-> COFINS com cache de alta performance
// e algoritmo de busca por especificidade (Best Fit).
// ============================================================================

/*
Esta implementação adota o padrão de Buckets (Baldes) com Score de Especificidade.

Em vez de tentar adivinhar a chave exata gerando múltiplas tentativas, o código agrupa
todos os candidatos possíveis sob uma chave simples (CST, Valor) e depois escolhe o
melhor candidato ("Best Fit") usando uma pontuação matemática.
*/

// ============================================================================
// CONSTANTES DE PESO (Hierarquia de Especificidade)
// ============================================================================
// A soma define a prioridade.
// Ex: (CFOP + Nat_BC_Cred + Part + Cta + Vl_BC_Cred) = 31 (Máxima especificidade)
// Ex: (Genérico) = 0 (Mínima especificidade)
const WEIGHT_CFOP: u8 = 16;
const WEIGHT_NAT_BC: u8 = 8;
const WEIGHT_PART: u8 = 4;
const WEIGHT_CTA: u8 = 2;
const WEIGHT_VL_BC: u8 = 1;

/// O resultado final: alíquota e valor do PIS.
/// Mantido como Copy para evitar clones desnecessários.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct CorrelationValue {
    pub aliq_pis: Option<Decimal>,
    pub vl_pis: Option<Decimal>,
}

/// Chave Primária do Hash: (CST, Valor).
/// O Decimal é normalizado para garantir consistência (10.00 == 10.0).
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CorrelationKey {
    pub cst: Option<CodigoSituacaoTributaria>,
    pub vl_item: Decimal,
}

impl CorrelationKey {
    /// Construtor que garante a normalização dos dados.
    /// CST é dado opcional podendo ser None!
    /// Valor do Item é dado obrigatório!
    pub fn new(cst: Option<CodigoSituacaoTributaria>, vl_item: Option<Decimal>) -> Option<Self> {
        let mut val = vl_item?;

        // CRUCIAL: Normaliza o decimal (remove zeros à direita) para garantir hash único
        // Ex: 10.00 -> 10. Para que o hash de 10.00 seja igual ao de 10.0
        val.normalize_assign();

        Some(Self { cst, vl_item: val })
    }
}

/// Contexto usado para consulta (Query).
/// Usa REFERÊNCIAS (&str) para evitar alocações durante a busca (Zero-Copy).
///
/// NOTA: Assume-se que os dados aqui já estão normalizados (String vazia = None),
/// garantido pelo trait `StringParser`.
#[derive(Debug, Clone, Copy)]
pub struct CorrelationCriteria<'a> {
    pub cfop: Option<u16>,
    pub nat_bc_cred: Option<u16>,
    pub part: Option<&'a str>,    // <--- Zero-copy
    pub cod_cta: Option<&'a str>, // <--- Zero-copy
    pub vl_bc: Option<Decimal>,
}

/// Estrutura que guarda os critérios de desempate
/// (CFOP, Nat_BC_Cred, Part, Cta) e o resultado (value).
///
/// Estrutura interna para armazenamento persistente.
/// Usa OWNED types (Arc<str>) para manter os dados no cache.
#[derive(Debug, Clone)]
struct CorrelationEntry {
    // Critérios (Se Some, exige igualdade na busca)
    cfop: Option<u16>,
    nat_bc_cred: Option<u16>,
    part: Option<Arc<str>>,    // <--- Persistente
    cod_cta: Option<Arc<str>>, // <--- Persistente
    vl_bc: Option<Decimal>,

    // Resultado associado
    value: CorrelationValue,
    aliq_cofins: Option<Decimal>,
}

impl CorrelationEntry {
    /// Cria uma nova entrada convertendo referências para Arc (Owned).
    fn new(criteria: CorrelationCriteria, value: CorrelationValue) -> Self {
        Self {
            cfop: criteria.cfop,
            nat_bc_cred: criteria.nat_bc_cred,
            part: criteria.part.map(Arc::from),
            cod_cta: criteria.cod_cta.map(Arc::from),
            vl_bc: criteria.vl_bc,
            value,
            aliq_cofins: None,
        }
    }

    /// Calcula o score de correlação comparando a Regra (self) com a Consulta (ctx).
    ///
    /// - self (CorrelationEntry) retém informacoes do PIS
    /// - ctx: CorrelationCriteria consulta informacoes da COFINS
    ///
    /// A lógica "Best Fit" premia a especificidade:
    /// - Se a regra define um valor e a consulta bate: Soma o peso.
    /// - Se a regra é genérica (None): Soma 0.
    /// - Se a regra define diferente da consulta: Soma 0.
    ///
    /// O resultado é que a regra com mais matches específicos terá o maior score.
    #[inline(always)]
    fn calculate_score(&self, ctx: CorrelationCriteria) -> u8 {
        // Solução: Função interna genérica.
        // T: PartialEq permite usar '=='
        // T: Copy permite passar os valores sem 'move' ou referências desnecessárias
        fn check_match<T: PartialEq + Copy>(rule: Option<T>, query: Option<T>) -> u8 {
            // Regra: Só pontua se a regra (cache) não for genérica (is_some)
            // E se for igual ao valor consultado.
            // cast 'bool as u8' converte true -> 1, false -> 0
            (rule.is_some() && rule == query) as u8
        }

        check_match(self.cfop, ctx.cfop) * WEIGHT_CFOP
            + check_match(self.nat_bc_cred, ctx.nat_bc_cred) * WEIGHT_NAT_BC
            + check_match(self.part.as_deref(), ctx.part) * WEIGHT_PART
            + check_match(self.cod_cta.as_deref(), ctx.cod_cta) * WEIGHT_CTA
            + check_match(self.vl_bc, ctx.vl_bc) * WEIGHT_VL_BC
    }
}

#[derive(Default)]
pub struct CorrelationManager {
    // Bucket Pattern:
    // Chave primária (CST + Valor) -> Lista de candidatos (Regras com CFOP, Nat_BC_Cred, Part, Cod_CTA)
    // Vec<CorrelationEntry> é uma lista de regras que compartilham o mesmo CST e Valor do Item.
    cache: HashMap<CorrelationKey, Vec<CorrelationEntry>>,
}

impl CorrelationManager {
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Armazena dados de PIS para possível correlação com COFINS.
    pub fn store(
        &mut self,
        key: CorrelationKey,
        criteria: CorrelationCriteria, // informacoes obtidas dos registros
        aliq_pis: Option<Decimal>,
        vl_pis: Option<Decimal>,
    ) {
        let value = CorrelationValue { aliq_pis, vl_pis };
        let new_entry = CorrelationEntry::new(criteria, value);

        // Sempre adiciona uma nova entrada (push), criando slots disponíveis.
        // O campo 'aliq_cofins' inicia como None, indicando que o slot está livre.
        self.cache.entry(key).or_default().push(new_entry);
    }

    /// Encontra o MELHOR candidato para os dados de COFINS fornecidos.
    ///
    /// Algoritmo:
    /// 1. Busca o bucket pelo CST e Valor do Item (O(1)).
    /// 2. Itera linearmente sobre as regras (O(N)).
    /// 3. Calcula o score de cada regra.
    /// 4. Retorna a regra com o MAIOR score.
    pub fn resolve(
        &mut self,
        cst: Option<CodigoSituacaoTributaria>,
        vl_item: Option<Decimal>,
        ctx: CorrelationCriteria,
        aliq_cofins: Option<Decimal>,
    ) -> Option<CorrelationValue> {
        let key = CorrelationKey::new(cst, vl_item)?;
        let bucket = self.cache.get_mut(&key)?;

        /*
        // Executar teste seguinte e ver resultado.
        // cargo test -- --show-output test_analyze_one_sped_file
        if bucket.len() > 2 || cst.is_none() || vl_item.is_none() {
            println!("{key:?}");
            println!("Informações de COFINS:\n{ctx:?}");
            println!("Informações de PIS:");
            for (index, entry) in bucket.iter().enumerate() {
                let score = entry.calculate_score(ctx);
                println!("entry[{}]: {entry:?}, score: {score}", index + 1);
            }
            println!();
        }
        */

        // max_by_key()
        // Itera uma vez, mantém apenas o ponteiro para o atual vencedor.
        // Custo O(N) + Zero Alocação.
        bucket
            .iter_mut()
            .filter(|entry| entry.aliq_cofins.is_none())
            .max_by_key(|entry| entry.calculate_score(ctx))
            .map(|entry| {
                entry.aliq_cofins = aliq_cofins;
                entry.value
            })
    }
}

// ============================================================================
// SEÇÃO 6: BUILDER PARA DocsFiscais
// Padrão Builder para construção de DocsFiscais, aplicando regras de negócio
// e recuperando dados do Contexto Global.
// ============================================================================

// ============================================================================
// SEÇÃO 6: BUILDER PARA DocsFiscais
// Padrão Builder Otimizado: Zero-Cost Abstractions & Functional Style
// ============================================================================

/// Estrutura auxiliar para transportar dados do Pai de forma legível.
/// Adicionado `Copy` e `Clone` para armazenamento eficiente (stack-only).
#[derive(Debug, Default, Clone, Copy)]
struct ParentHeader<'a> {
    dt_emissao: Option<NaiveDate>,
    dt_entrada: Option<NaiveDate>,
    cfop: Option<u16>,
    chave: Option<&'a str>,
    cod_cta: Option<&'a str>,
    cod_inf: Option<&'a str>,
    cod_item: Option<&'a str>,
    cod_mod: Option<&'a str>,
    cod_ncm: Option<&'a str>,
    cod_part: Option<&'a str>,
    num_doc: Option<usize>,
    vl_iss: Option<Decimal>,
    vl_bc_icms: Option<Decimal>,
    vl_icms: Option<Decimal>,
}

impl<'a> ParentHeader<'a> {
    /// Mapear campos de RegistroPai para ParentHeader
    ///
    /// RegistroPai -> ParentHeader
    fn new<P>(p: &'a P) -> Self
    where
        P: RegistroPai + ?Sized,
    {
        Self {
            dt_emissao: p.get_dt_emissao(),
            dt_entrada: p.get_dt_entrada(),
            cfop: p.get_cfop(),
            chave: p.get_chave(),
            cod_cta: p.get_cod_cta(),
            cod_inf: p.get_cod_inf(),
            cod_item: p.get_cod_item(),
            cod_mod: p.get_cod_mod(),
            cod_ncm: p.get_cod_ncm(),
            cod_part: p.get_cod_part(),
            num_doc: p.get_num_doc(),
            vl_iss: p.get_valor_iss(),
            vl_bc_icms: p.get_valor_bc_icms(),
            vl_icms: p.get_valor_icms(),
        }
    }

    /// Extrair dados de `Option<RegistroPai>` para `ParentHeader`
    ///
    /// `Option<RegistroPai>` -> `ParentHeader`:
    /// - Se pai for None, retorna Default (Zero Cost)
    /// - Se pai existir, mapeia os campos
    fn from_record<P>(pai: Option<&'a P>) -> Self
    where
        P: RegistroPai + ?Sized,
    {
        pai.map(Self::new).unwrap_or_default()
    }
}

#[derive(Clone)]
struct DocsBuilder<'a> {
    ctx: &'a SpedContext,     // [STACK] 8 bytes (ponteiro)
    header: ParentHeader<'a>, // [STACK] ~100 bytes (apenas ponteiros e números)
    doc: DocsFiscais,         // [STACK] Estrutura contendo Arcs (ponteiros inteligentes)
}

impl<'a> DocsBuilder<'a> {
    fn new(
        ctx: &'a SpedContext,
        registro: &str,
        line_num: usize,
        current_cnpj: Option<Arc<str>>,
    ) -> Self {
        // Zero-copy se CNPJ não mudar
        let estabelecimento_cnpj = ctx.obter_cnpj_do_estabelecimento(current_cnpj);
        let estabelecimento_nome = ctx.obter_nome_do_estabelecimento(&estabelecimento_cnpj);

        let doc = DocsFiscais {
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
            ctx,
            header: ParentHeader::default(), // Inicia com header vazio
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
        current_cnpj: Option<Arc<str>>,
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

    fn from_child<F>(ctx: &'a SpedContext, reg: &F, current_cnpj: Option<Arc<str>>) -> Self
    where
        F: RegistroFilho + ?Sized,
    {
        // Passa None como pai, gerando um ParentHeader default internamente
        Self::from_child_and_parent(ctx, reg, None::<&()>, current_cnpj)
    }

    // --- Fases de Construção ---

    /// Prioridade das informações: Filho -> Pai
    fn with_header<F>(mut self, filho: &F) -> Self
    where
        F: RegistroFilho + ?Sized,
    {
        // Informações do Filho prevalesce sobre informações do Pai
        self.doc.data_emissao = filho
            .get_dt_emissao()
            .or(self.header.dt_emissao)
            .or(self.ctx.periodo_de_apuracao);
        self.doc.data_entrada = filho.get_dt_entrada().or(self.header.dt_entrada);

        self.doc.cfop = filho.get_cfop().or(self.header.cfop);
        self.doc.chave_doc = self.header.chave.unwrap_or_default().into();
        self.doc.num_doc = self.header.num_doc;
        self.doc.cod_ncm = self.header.cod_ncm.unwrap_or_default().into();

        self.doc.valor_iss = filho.get_valor_iss().or(self.header.vl_iss);

        self.doc.valor_bc_icms = filho.get_valor_bc_icms().or(self.header.vl_bc_icms);

        self.doc.valor_icms = filho.get_valor_icms().or(self.header.vl_icms);

        if let Some(m) = filho.get_cod_mod().or(self.header.cod_mod) {
            self.doc.modelo_doc_fiscal = ModeloDocFiscal::from_str(m)
                .map(|modelo| modelo.descricao_com_codigo()) // Se Ok, formata: "55 - Nota Fiscal..."
                .unwrap_or_default() // Se Err (inválido), retorna ""
                .into(); // String -> Arc<str>
        }

        self
    }

    fn with_itens_and_participant<F>(mut self, filho: &F) -> Self
    where
        F: RegistroFilho + ?Sized,
    {
        // 1. Resolve Itens e Produto
        let cod_item = filho.get_cod_item().or(self.header.cod_item);
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
        // 1. Flattening: Se não tem cod_item ou não achou o produto, retorna cedo.
        let Some(reg_0200) = cod_item.and_then(|c| self.ctx.produtos.get(c)) else {
            return;
        };

        // Só aplica NCM se estiver vazio OU igual a "00000000".

        // &* acessa o valor dentro do Arc sem clonar
        // O segredo é usar &* (deref) ou .as_ref() para transformar
        // o Arc<str> em um simples &str temporário.
        if (self.doc.cod_ncm.is_empty() || self.doc.cod_ncm.as_ref() == "00000000")
            && let Some(cod_ncm) = &reg_0200.cod_ncm
        {
            self.doc.cod_ncm = cod_ncm.clone();
        }

        self.doc.tipo_item = reg_0200.tipo_item;

        if let Some(desc) = &reg_0200.descr_item {
            self.doc.descr_item = desc.clone();
        }
    }

    /// Aplica dados do participante
    fn apply_participant_info(&mut self, cod_part: Option<&str>) {
        // filter: Ignora strings vazias ("Fail Fast")
        let Some(cod) = cod_part.filter(|s| !s.is_empty()) else {
            return;
        };

        // 1. Tentativa pelo Código do Participante (Lookup rápido no Hash de Participantes)
        if let Some(reg_0150) = self.ctx.participantes.get(cod) {
            // Zero-copy clones (Arc)
            if let Some(cnpj) = &reg_0150.cnpj {
                self.doc.participante_cnpj = cnpj.clone();
            }
            if let Some(cpf) = &reg_0150.cpf {
                self.doc.participante_cpf = cpf.clone();
            }
            if let Some(nome) = &reg_0150.nome {
                self.doc.participante_nome = nome.clone();
            }
            return;
        }

        // 2. Fallback: O código não está no cadastro (0150), mas pode ser o próprio CNPJ/CPF direto no campo.
        // Isso acontece frequentemente em registros filhos que citam terceiros não cadastrados.
        match cod.len() {
            14 => {
                // É um CNPJ
                self.doc.participante_cnpj = cod.into();
                // Busca inteligente: Exato -> Base Frequente -> Vazio
                if let Some(nome) = self.ctx.obter_nome_por_cnpj(cod) {
                    self.doc.participante_nome = nome;
                }
            }
            11 => {
                // É um CPF
                self.doc.participante_cpf = cod.into();
                if let Some(nome) = self.ctx.obter_nome_por_cpf(cod) {
                    self.doc.participante_nome = nome;
                }
            }
            _ => {
                // Código desconhecido e formato inválido
            }
        }
    }

    fn with_values_and_classification<F>(mut self, filho: &F) -> Self
    where
        F: RegistroFilho + ?Sized,
    {
        // 1. Identificadores e Classificação
        self.doc.num_item = filho.get_num_item();
        self.doc.cst = filho.get_cst_cofins();

        self.doc.natureza_bc = filho
            .get_nat_bc_cred()
            .and_then(NaturezaBaseCalculo::from_u16);

        self.doc.indicador_de_origem = filho.get_ind_orig_cred().parse_opt();

        // 2. Valores Base
        self.doc.valor_item = filho.get_valor_item();
        self.doc.valor_bc = filho.get_valor_bc_cofins();

        // 3. Tributos
        self.doc.aliq_pis = filho.get_aliq_pis();
        self.doc.aliq_cofins = filho.get_aliq_cofins();

        self.doc.valor_pis = filho.get_valor_pis();
        self.doc.valor_cofins = filho.get_valor_cofins();

        self.doc.aliq_icms = filho.get_aliq_icms();

        // Lookup de Natureza
        if let Some(desc) = filho
            .get_cod_nat()
            .and_then(|c| self.ctx.nat_operacao.get(c))
        {
            self.doc.nat_operacao = desc.clone();
        }

        // Adicionar descrição complementar
        self.apply_info_complementar(filho.get_descr_compl());

        // Lookup Contábil
        // Prioridade das informações: Filho -> Pai
        let codigo_da_conta = filho.get_cod_cta().or(self.header.cod_cta);
        self.apply_account_name(codigo_da_conta);

        self
    }

    /// Aplicar informação complementar
    fn apply_info_complementar(&mut self, descr_compl: Option<&str>) {
        if let Some(info) = descr_compl {
            // informação complementar obtida diretamente sem código
            self.doc.complementar = info.into();
        } else if let Some(info) = self
            .header
            .cod_inf
            .and_then(|c| self.ctx.complementar.get(c))
        {
            // informação complementar a partir do Código do Registro 0450
            // Ver utilização nos registros A110, C110, C500, D100, D500
            self.doc.complementar = info.clone();
        }
    }

    /// Buscar e aplicar nome da conta contábil
    fn apply_account_name(&mut self, cod_cta: Option<&str>) {
        if let Some(nome) = cod_cta.and_then(|c| self.ctx.contabil.get(c)) {
            self.doc.nome_da_conta = nome.clone();
        }
    }

    fn resolve_pis_correlation<F>(mut self, manager: &mut CorrelationManager, filho: &F) -> Self
    where
        F: RegistroFilho + ?Sized,
    {
        // 1. Se já tem valores, retorna cedo
        if self.doc.valor_pis.is_some_and(|p| p.eh_maior_que_zero())
            && self.doc.aliq_pis.is_some_and(|p| p.eh_maior_que_zero())
        {
            return self;
        }

        // 2. Determina o participante (Filho > Pai)
        let cod_participante = filho.get_part_override().or(self.header.cod_part);

        // 3. Tenta resolver via Manager
        let corr_ctx = CorrelationCriteria {
            // Informações de COFINS
            cfop: self.doc.cfop,
            nat_bc_cred: filho.get_nat_bc_cred(),
            part: cod_participante,
            cod_cta: filho.get_cod_cta(),
            vl_bc: filho.get_valor_bc_cofins(),
        };

        if let Some(pis_data) = manager.resolve(
            filho.get_cst_cofins(), // Assume-se que CST COFINS correlaciona com CST PIS armazenado
            filho.get_valor_item(),
            corr_ctx,
            filho.get_aliq_cofins(),
        ) {
            self.doc.aliq_pis = pis_data.aliq_pis;
            self.doc.valor_pis = pis_data.vl_pis;
        } else if let Some(aliq_cof) = self.doc.aliq_cofins {
            if aliq_cof == ALIQ_BASICA_COF {
                self.doc.aliq_pis = Some(ALIQ_BASICA_PIS);
            } else if aliq_cof == dec!(3.0) {
                // Regra comum para lucro presumido/cumulativo
                self.doc.aliq_pis = Some(dec!(0.65));
            }
        }
        self
    }

    fn resolve_tipo_de_operacao(mut self) -> Self {
        if self.doc.tipo_de_operacao.is_none() {
            self.doc.tipo_de_operacao = self.doc.cst.obter_tipo_operacao();
        }
        self
    }

    fn resolve_natureza_bc(mut self) -> Self {
        // Se a natureza já existe, não fazemos nada.
        // Caso contrário, tentamos resolver via CFOP/CST.
        if self.doc.natureza_bc.is_none() {
            self.doc.natureza_bc = obter_natureza_da_bc(self.doc.cfop, self.doc.cst);
        }
        self
    }

    /// Resolve o indicador da origem da operação.
    ///
    /// Ordem de Precedência (Lógica Funcional):
    /// 1. Valor explícito já setado (vindo do campo `ind_orig_cred`).
    /// 2. Derivado do Código do Crédito (se terminar em 08 = Importação).
    /// 3. Derivado do CFOP (se for 3xxx = Importação).
    /// 4. Default: Mercado Interno.
    fn resolve_indicador_de_origem(mut self) -> Self {
        self.doc.indicador_de_origem = self
            .doc
            .indicador_de_origem
            // 1. Prioridade: Se o código do crédito indica importação (YY=08)
            // .or_else é Lazy: só executa se o anterior for None
            .or_else(|| {
                self.doc
                    .cod_credito
                    .filter(|c| c.credito == TipoDeCredito::Importacao)
                    .map(|_| IndicadorDeOrigem::Importacao)
            })
            // 2. Fallback: Range de CFOP (3000-3999)
            .or_else(|| {
                self.doc
                    .cfop
                    .filter(|&c| is_importacao(c))
                    .map(|_| IndicadorDeOrigem::Importacao)
            })
            // 4. Fallback Padrão (Custo: Zero - Constante)
            // .or é aceitável aqui pois é um valor estático, mas or_else também funcionaria
            .or(Some(IndicadorDeOrigem::MercadoInterno));

        self
    }

    /// Resolve o tipo de crédito.
    ///
    /// Dependência direta: Requer que `resolve_indicador_de_origem` tenha sido executado antes.
    /// Utiliza o `IndicadorDeOrigem` resolvido para decidir entre fluxos de Importação ou Mercado Interno.
    fn resolve_tipo_de_credito(mut self) -> Self {
        // Chamada limpa: não precisa passar argumentos, o método já conhece o 'self'
        self.doc.tipo_de_credito = self.calcular_tipo_de_credito();
        self
    }

    // ------------------------------------------------------------------------
    // Método Auxiliar Privado (Lógica de Negócio)
    // ------------------------------------------------------------------------

    /// Lógica de Negócio para determinar o Tipo de Crédito.
    ///
    /// Adota um estilo funcional de "cadeia de prioridades" (Chain of Responsibility).
    fn calcular_tipo_de_credito(&self) -> Option<TipoDeCredito> {
        // Acesso direto aos campos via self.doc
        let cst_cofins = self.doc.cst;

        // 0. Validação Prévia: Se o CST não permite crédito, encerra imediatamente.
        if cst_cofins.is_some_and(|cst| !cst.eh_base_de_credito()) {
            return None;
        }

        // 1. Prioridade Máxima: Se houver um Código do Crédito (XYY) válido,
        // o Tipo de Crédito é extraído diretamente do campo 'credito' (YY).
        // 2. Segunda Prioridade: Heurística baseada no Indicador de Origem (Importação).
        // 3. Terceira Prioridade: Inferência baseada em Alíquotas e CST (Mercado Interno).
        self.doc
            .cod_credito
            .map(|cod| cod.credito)
            .or_else(|| self.inferir_por_origem())
            .or_else(|| self.inferir_por_aliquotas_e_cst())
    }

    // --- Métodos Auxiliares de Inferência ---

    #[inline]
    fn inferir_por_origem(&self) -> Option<TipoDeCredito> {
        if matches!(
            self.doc.indicador_de_origem,
            Some(IndicadorDeOrigem::Importacao)
        ) {
            Some(TipoDeCredito::Importacao)
        } else {
            None
        }
    }

    fn inferir_por_aliquotas_e_cst(&self) -> Option<TipoDeCredito> {
        let d = &self.doc;
        let cst_cofins = self.doc.cst;

        // Verifica se há alíquotas positivas (uso da trait DecimalExt/FloatExt definida anteriormente)
        let tem_aliquota = d.aliq_pis.eh_maior_que_zero() || d.aliq_cofins.eh_maior_que_zero();
        if !tem_aliquota {
            return None;
        }

        // Lógica de mapeamento baseada no CST
        match cst_cofins?.code() {
            // CSTs 50 a 56: Diferencia Básica de Diferenciada
            50..=56 => {
                let eh_aliq_basica =
                    d.aliq_pis.eh_igual(ALIQ_BASICA_PIS) && d.aliq_cofins.eh_igual(ALIQ_BASICA_COF);

                if eh_aliq_basica {
                    Some(TipoDeCredito::AliquotaBasica)
                } else {
                    Some(TipoDeCredito::AliquotasDiferenciadas)
                }
            }
            // CSTs 60 a 66: Créditos Presumidos
            60..=66 => {
                if cred_presumido(d.aliq_pis, d.aliq_cofins) {
                    Some(TipoDeCredito::PresumidoAgroindustria)
                } else {
                    Some(TipoDeCredito::OutrosCreditosPresumidos)
                }
            }
            _ => None,
        }
    }

    fn build(self) -> DocsFiscais {
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
// SEÇÃO 7: PROCESSADORES DE BLOCO
// Iteradores paralelos chamam esta função para processar blocos inteiros.
// ============================================================================

pub fn process_block_lines(
    bloco: char,
    file: &SpedFile,
    ctx: &SpedContext,
) -> (Vec<DocsFiscais>, Vec<String>) {
    let mut docs = Vec::new();
    let mut messages = Vec::new(); // Buffer local para mensagens deste bloco

    match bloco {
        // Numero de registro no BlocoA
        // rg -a '^\|A990\|' PISCOFINS_2021*.txt
        /*
        'A' => {
            let records = file.get_bloco_a();
            if !records.is_empty() {
                BlocoAExtractor::default().process(records, ctx, &mut docs);
            }
        }
        */
        'A' => {
            let records = file.get_bloco_a();
            if !records.is_empty() {
                // Identificador de "Pai Majoritário" para o Bloco A
                let is_major_header = |r: &BlocoA| matches!(r, BlocoA::RA100(_));

                let docs_a: Vec<DocsFiscais> = records
                    // CAMADA 1: Agrupar por Estabelecimento (A010)
                    .chunk_by(|_, next| !matches!(next, BlocoA::RA010(_)))
                    .collect::<Vec<&[BlocoA]>>()
                    .into_par_iter()
                    .with_max_len(NUM_DE_ESTABELECIMENTOS) // Nº de Estabelecimentos distintos
                    .flat_map(|establ_slice| {
                        // 1. Extração do contexto de CNPJ do estabelecimento
                        let current_cnpj = if let Some(BlocoA::RA010(r)) = establ_slice.first() {
                            r.cnpj.to_arc()
                        } else {
                            None
                        };

                        establ_slice
                            .chunk_by(|_current, next| !is_major_header(next))
                            .collect::<Vec<&[BlocoA]>>()
                            .into_par_iter()
                            .with_min_len(LIMITE_LINHAS) // Nº de Pais A100 distintos
                            .flat_map_iter(move |doc_chunk| {
                                /*
                                println!(
                                    "grupo do BlocoA com {n} registros (1 Pai e {f} Filhos):",
                                    n = doc_chunk.len(),
                                    f = doc_chunk.len() - 1
                                );
                                doc_chunk
                                    .iter()
                                    .enumerate()
                                    .take(5)
                                    .for_each(|(idx, reg)| {
                                        println!(" {n} {reg:?}", n = idx + 1);
                                    });
                                println!();
                                */

                                let mut local_docs = Vec::new();
                                let mut extractor = BlocoAExtractor {
                                    current_cnpj: current_cnpj.clone(),
                                    ..Default::default()
                                };
                                extractor.process(doc_chunk, ctx, &mut local_docs);
                                local_docs.into_iter()
                            })
                    })
                    .collect(); // Mantém a ordem original do arquivo

                docs.extend(docs_a);
            }
        }
        /*
        'C' => {
            let records = file.get_bloco_c();
            if !records.is_empty() {
                BlocoCExtractor::default().process(records, ctx, &mut docs);
            }
        }
        */
        'C' => {
            let records = file.get_bloco_c();
            if !records.is_empty() {
                // Auxiliar para identificar registros que "resetam" a hierarquia (Pais Majoritários)
                let is_major_header = |r: &BlocoC| {
                    matches!(
                        r,
                        BlocoC::RC100(_)
                            | BlocoC::RC180(_)
                            | BlocoC::RC190(_)
                            | BlocoC::RC380(_)
                            | BlocoC::RC395(_)
                            | BlocoC::RC400(_)
                            | BlocoC::RC405(_)
                            | BlocoC::RC490(_)
                            | BlocoC::RC500(_)
                            | BlocoC::RC600(_)
                            | BlocoC::RC860(_)
                    )
                };

                let docs_c: Vec<DocsFiscais> = records
                    // CAMADA 1: Agrupar por Estabelecimento (C010)
                    .chunk_by(|_current, next| !matches!(next, BlocoC::RC010(_)))
                    .collect::<Vec<&[BlocoC]>>()
                    .into_par_iter()
                    .with_max_len(NUM_DE_ESTABELECIMENTOS) // Nº de Estabelecimentos distintos
                    .flat_map(|establ_slice| {
                        // Captura o CNPJ do estabelecimento para este slice
                        let current_cnpj = if let Some(BlocoC::RC010(r)) = establ_slice.first() {
                            r.cnpj.to_arc()
                        } else {
                            None
                        };

                        // CAMADA 2: Particionar o estabelecimento em grupos de documentos
                        // Isso garante que um C100 e todos os seus C170 fiquem no mesmo chunk
                        establ_slice
                            .chunk_by(|_current, next| !is_major_header(next))
                            .collect::<Vec<_>>()
                            .into_par_iter()
                            .with_min_len(LIMITE_LINHAS) // Nº de Pais C100 (por exemplo) distintos
                            .flat_map_iter(move |doc_chunk| {
                                let mut local_docs = Vec::new();
                                let mut extractor = BlocoCExtractor {
                                    current_cnpj: current_cnpj.clone(),
                                    ..Default::default()
                                };
                                extractor.process(doc_chunk, ctx, &mut local_docs);
                                local_docs.into_iter()
                            })
                    })
                    .collect();

                docs.extend(docs_c);
            }
        }
        /*
        'D' => {
            let records = file.get_bloco_d();
            if !records.is_empty() {
                BlocoDExtractor::default().process(records, ctx, &mut docs);
            }
        }
        */
        'D' => {
            let records = file.get_bloco_d();
            if !records.is_empty() {
                // Auxiliar para identificar registros que iniciam um novo documento ou resumo
                let is_major_header = |r: &BlocoD| {
                    matches!(
                        r,
                        BlocoD::RD100(_)
                            | BlocoD::RD200(_)
                            | BlocoD::RD300(_)
                            | BlocoD::RD350(_)
                            | BlocoD::RD500(_)
                            | BlocoD::RD600(_)
                    )
                };

                let docs_d: Vec<DocsFiscais> = records
                    // CAMADA 1: Particionamento por Estabelecimento (D010)
                    .chunk_by(|_, next| !matches!(next, BlocoD::RD010(_)))
                    .collect::<Vec<_>>()
                    .into_par_iter()
                    .with_max_len(NUM_DE_ESTABELECIMENTOS) // Nº de Estabelecimentos distintos
                    .flat_map(|establ_slice| {
                        // Extração do contexto de CNPJ do estabelecimento atual
                        let current_cnpj = if let Some(BlocoD::RD010(r)) = establ_slice.first() {
                            r.cnpj.to_arc()
                        } else {
                            None
                        };

                        // CAMADA 2: Particionamento por Unidade de Documento
                        // Agrupa o Pai e seus respectivos Filhos para processamento na mesma thread
                        establ_slice
                            .chunk_by(|_, next| !is_major_header(next))
                            .collect::<Vec<_>>()
                            .into_par_iter()
                            .with_min_len(LIMITE_LINHAS) // Nº de Pais D100 (por exemplo) distintos
                            .flat_map_iter(move |doc_chunk| {
                                let mut local_docs = Vec::new();
                                let mut extractor = BlocoDExtractor {
                                    current_cnpj: current_cnpj.clone(),
                                    ..Default::default()
                                };
                                extractor.process(doc_chunk, ctx, &mut local_docs);
                                local_docs.into_iter()
                            })
                    })
                    .collect();

                docs.extend(docs_d);
            }
        }
        /*
        'F' => {
            let records = file.get_bloco_f();
            if !records.is_empty() {
                BlocoFExtractor::default().process(records, ctx, &mut docs);
            }
        }
        */
        // No Bloco F, os registros não dependem do anterior. Podemos processá-los em chunks.
        // O chunk_by percorre o slice e agrupa os elementos enquanto um predicado for verdadeiro.
        // Para o Bloco F, definimos que "um novo grupo deve começar sempre que encontrarmos um F010".
        'F' => {
            let records = file.get_bloco_f();
            if !records.is_empty() {
                let docs_f: Vec<DocsFiscais> = records
                    // Camada 1: Agrupa por F010 (Estabelecimentos)
                    .chunk_by(|_, next| !matches!(next, BlocoF::RF010(_)))
                    .collect::<Vec<_>>()
                    .into_par_iter()
                    .with_max_len(NUM_DE_ESTABELECIMENTOS) // Nº de Estabelecimentos distintos
                    .flat_map(|establ_slice| {
                        // Extraímos o contexto (CNPJ) do início do slice de forma segura
                        // Isso permite que cada thread interna comece já sabendo a quem pertence o dado.
                        let initial_cnpj = if let Some(BlocoF::RF010(reg)) = establ_slice.first() {
                            reg.cnpj.to_arc()
                        } else {
                            None
                        };

                        // Camada 2: Paralelismo aninhado para processar a massa de dados (ex: 1 milhão de F100s)
                        // Usamos par_chunks para que as 16 threads do i9 dividam o fardo deste slice gigante.
                        establ_slice
                            .par_chunks(LIMITE_LINHAS)
                            .flat_map_iter(move |chunk| {
                                let mut local_docs = Vec::with_capacity(chunk.len());
                                let mut extractor = BlocoFExtractor {
                                    current_cnpj: initial_cnpj.clone(),
                                };
                                extractor.process(chunk, ctx, &mut local_docs);
                                local_docs.into_iter()
                            })
                    })
                    .collect(); // O Rayon garante a ordem global (Pai antes de Filho e Establ. 1 antes de 2)

                docs.extend(docs_f);
            }
        }
        'I' => {
            let records = file.get_bloco_i();
            if !records.is_empty() {
                BlocoIExtractor::default().process(records, ctx, &mut docs);
            }
        }
        'M' => {
            let records = file.get_bloco_m();
            if !records.is_empty() {
                let mut bloco_m = BlocoMExtractor::default();
                bloco_m.process(records, ctx, &mut docs, &mut messages);

                // Se houve correlação global, geramos o relatório.
                if bloco_m.correlacao.correlacao_inexistente || log_enabled!(Level::Debug) {
                    // Gera o relatório uma única vez
                    let relatorio = bloco_m.correlacao.generate_report();

                    if log_enabled!(Level::Debug) {
                        print!("{}", relatorio);
                    }

                    if bloco_m.correlacao.correlacao_inexistente {
                        messages.push(relatorio);
                    }
                }
            }
        }
        '1' => {
            let records = file.get_bloco_1();
            if !records.is_empty() {
                Bloco1Extractor::default().process(records, ctx, &mut docs, &mut messages);
            }
        }
        _ => {}
    }

    (docs, messages)
}

// ============================================================================
// Implementação do Bloco A
// ============================================================================

/// --- Bloco A (Serviços) ---
#[derive(Default)]
struct BlocoAExtractor<'a> {
    /// Mantém o registro pai (A100) em memória para os filhos (A170)
    a100: Option<&'a RegistroA100>,
    /// CNPJ do estabelecimento atual (A010)
    current_cnpj: Option<Arc<str>>,
}
impl<'a> BlocoAExtractor<'a> {
    fn process(&mut self, records: &'a [BlocoA], ctx: &SpedContext, docs: &mut Vec<DocsFiscais>) {
        for bloco_a in records {
            match bloco_a {
                // A010: Identificação do Estabelecimento
                BlocoA::RA010(r) => {
                    self.current_cnpj = r.cnpj.to_arc();
                }

                // A100: Registro Pai (Documento Fiscal)
                BlocoA::RA100(r) => {
                    self.a100 = Some(r);
                }

                // A170: Registro Filho (Itens do Documento)
                BlocoA::RA170(filho) => {
                    process_child_and_parent!(docs, ctx, self.current_cnpj, filho, self.a100)
                }

                // Ignora outros registros (A001, A990, etc.) que não geram DocsFiscais
                _ => {}
            }
        }
    }
}

// ============================================================================
// Bloco C (Mercadorias - ICMS/IPI)
// ============================================================================

/// --- Bloco C (Mercadorias - Complexo) ---
#[derive(Default)]
struct BlocoCExtractor<'a> {
    // Referências para registros Pais (rastreamento de hierarquia)
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

    // Estado de contexto
    header: ParentHeader<'a>, // Struct auxiliar para dados do cabeçalho
    correlacao: CorrelationManager, // Gerenciador de PIS/COFINS
    c195_idxs: Vec<usize>,    // Índices para atualização tardia (C199)
    current_cnpj: Option<Arc<str>>,
}

impl<'a> BlocoCExtractor<'a> {
    fn process(&mut self, records: &'a [BlocoC], ctx: &SpedContext, docs: &mut Vec<DocsFiscais>) {
        for bloco_c in records {
            match bloco_c {
                // --- Estabelecimento ---
                BlocoC::RC010(r) => capture_cnpj!(self.current_cnpj, r),
                // --- Documentos Fiscais (Pai C100) ---
                BlocoC::RC100(pai) => {
                    self.correlacao.clear();
                    self.c100 = Some(pai);
                    self.header = ParentHeader::new(pai);
                }
                BlocoC::RC110(reg) => {
                    self.header.cod_inf = reg.cod_inf.as_deref();
                }
                BlocoC::RC170(filho) => {
                    // 1. Criação: Instancia o builder com dados básicos (Contexto Global)
                    // O header inicia vazio (Default) para economizar alocação prévia.
                    let mut builder = DocsBuilder::new(
                        ctx,
                        filho.registro_name(),
                        filho.line_number(),
                        self.current_cnpj.clone(),
                    );

                    // 2. Contexto: Extrai dados do Pai e move para dentro do Builder (Zero-Copy)
                    // Isso centraliza as referências do pai em um único struct otimizado (Stack).
                    builder.header = self.header;

                    // 3. Preenchimento: Pipeline funcional (Fluent API)
                    // Cada etapa consome o builder, aplica regras de negócio e o retorna modificado.
                    let doc = builder
                        .with_header(filho) // Mescla dados de cabeçalho (Datas, Chaves)
                        .with_itens_and_participant(filho) // Resolve Itens e Participante
                        .with_values_and_classification(filho) // Aplica valores e CSTs
                        .resolve_pis_correlation(&mut self.correlacao, filho)
                        .build();

                    docs.push(doc)
                }
                BlocoC::RC175(reg) => {
                    process_child_and_parent!(docs, ctx, self.current_cnpj, reg, self.c100)
                }
                // --- Operações com ST (C180-C185) ---
                BlocoC::RC180(pai) => {
                    self.c180 = Some(pai);
                    self.correlacao.clear();
                }
                BlocoC::RC181(r) => store_pis!(self.correlacao, r),
                BlocoC::RC185(r) => {
                    process_correlations!(
                        docs,
                        ctx,
                        self.current_cnpj,
                        &mut self.correlacao,
                        r,
                        self.c180
                    );
                }

                // --- Consolidação de Notas (C190-C199) ---
                BlocoC::RC190(pai) => {
                    self.c190 = Some(pai);
                    self.correlacao.clear();
                    self.c195_idxs.clear();
                }
                BlocoC::RC191(r) => store_pis!(self.correlacao, r),
                BlocoC::RC195(r) => {
                    if let Some(pai) = self.c190 {
                        let doc = DocsBuilder::from_child_and_parent(
                            ctx,
                            r,
                            Some(pai),
                            self.current_cnpj.clone(),
                        )
                        .resolve_pis_correlation(&mut self.correlacao, r)
                        .build();
                        docs.push(doc);
                        self.c195_idxs.push(docs.len() - 1);
                    }
                }
                BlocoC::RC199(r) => {
                    // 1. Otimização: Se não há índices para atualizar, sai cedo.
                    if !self.c195_idxs.is_empty()
                        && let Some(n) = &r.num_doc_imp
                    {
                        let info_extra = format!("Número do documento de Importação: {}", n);

                        // 2. Cria um Arc inicial para tentar reutilizar a alocação (clone barato)
                        let info_arc: Arc<str> = Arc::from(info_extra);

                        for &i in &self.c195_idxs {
                            if let Some(d) = docs.get_mut(i) {
                                d.complementar = if d.complementar.is_empty() {
                                    // 3. Clone barato: aponta para o mesmo local de memória
                                    info_arc.clone()
                                } else {
                                    // 4. Concatenação: infelizmente exige nova alocação pois Arc<str> é imutável
                                    format!("{} {}", d.complementar, info_arc).into()
                                };
                            }
                        }
                    }
                }

                // --- Notas Fiscais de Venda a Consumidor (C380-C385) ---
                BlocoC::RC380(pai) => {
                    self.c380 = Some(pai);
                    self.correlacao.clear();
                }
                BlocoC::RC381(r) => store_pis!(self.correlacao, r),
                BlocoC::RC385(r) => {
                    process_correlations!(
                        docs,
                        ctx,
                        self.current_cnpj,
                        &mut self.correlacao,
                        r,
                        self.c380
                    );
                }

                // --- Notas Fiscais de Serviço (C395-C396) ---
                BlocoC::RC395(pai) => self.c395 = Some(pai),
                BlocoC::RC396(r) => {
                    process_child_and_parent!(docs, ctx, self.current_cnpj, r, self.c395);
                }

                // --- Cupons Fiscais ECF (C400-C485) ---
                BlocoC::RC400(pai) => {
                    self.c400 = Some(pai);
                    self.correlacao.clear();
                }
                BlocoC::RC405(pai) => self.c405 = Some(pai),
                BlocoC::RC481(r) => store_pis!(self.correlacao, r),
                BlocoC::RC485(r) => {
                    process_correlations!(
                        docs,
                        ctx,
                        self.current_cnpj,
                        &mut self.correlacao,
                        r,
                        self.c405
                    );
                }

                // --- Resumo Diário (C490-C495) ---
                BlocoC::RC490(pai) => {
                    self.c490 = Some(pai);
                    self.correlacao.clear();
                }
                BlocoC::RC491(r) => store_pis!(self.correlacao, r),
                BlocoC::RC495(r) => {
                    process_correlations!(
                        docs,
                        ctx,
                        self.current_cnpj,
                        &mut self.correlacao,
                        r,
                        self.c490
                    );
                }

                // --- Energia/Gás/Água (C500-C505) ---
                BlocoC::RC500(pai) => {
                    self.c500 = Some(pai);
                    self.correlacao.clear();
                }
                BlocoC::RC501(r) => store_pis!(self.correlacao, r),
                BlocoC::RC505(r) => {
                    process_correlations!(
                        docs,
                        ctx,
                        self.current_cnpj,
                        &mut self.correlacao,
                        r,
                        self.c500
                    );
                }

                // --- Notas Consolidadas (C600-C605) ---
                BlocoC::RC600(pai) => {
                    self.c600 = Some(pai);
                    self.correlacao.clear();
                }
                BlocoC::RC601(r) => store_pis!(self.correlacao, r),
                BlocoC::RC605(r) => {
                    process_correlations!(
                        docs,
                        ctx,
                        self.current_cnpj,
                        &mut self.correlacao,
                        r,
                        self.c600
                    );
                }

                // --- Equipamento SAT (C860-C870) ---
                BlocoC::RC860(pai) => self.c860 = Some(pai),
                BlocoC::RC870(r) => {
                    process_child_and_parent!(docs, ctx, self.current_cnpj, r, self.c860);
                }

                // --- Registros Solo ---
                BlocoC::RC880(r) => {
                    process_only_child!(docs, ctx, self.current_cnpj, r);
                }

                _ => {} // Outros registros do Bloco C não processados para DocsFiscais
            }
        }
    }
}

// ============================================================================
// Bloco D (Transportes)
// ============================================================================

/// --- Bloco D (Transportes e Comunicações) ---
#[derive(Default)]
struct BlocoDExtractor<'a> {
    d100: Option<&'a RegistroD100>,
    d200: Option<&'a RegistroD200>,
    d500: Option<&'a RegistroD500>,
    d600: Option<&'a RegistroD600>,
    correlacao: CorrelationManager,
    current_cnpj: Option<Arc<str>>,
}

impl<'a> BlocoDExtractor<'a> {
    fn process(&mut self, records: &'a [BlocoD], ctx: &SpedContext, docs: &mut Vec<DocsFiscais>) {
        for bloco_d in records {
            match bloco_d {
                // Identificação do estabelecimento
                BlocoD::RD010(r) => capture_cnpj!(self.current_cnpj, r),

                // --- Grupo D100 (Notas Fiscais de Serviço de Transporte) ---
                BlocoD::RD100(r) => {
                    self.d100 = Some(r);
                    self.correlacao.clear(); // Reseta correlação para novo documento pai
                }
                BlocoD::RD101(r) => store_pis!(self.correlacao, r),
                BlocoD::RD105(r) => process_correlations!(
                    docs,
                    ctx,
                    self.current_cnpj,
                    &mut self.correlacao,
                    r,
                    self.d100
                ),

                // --- Grupo D200 (Resumo de Transporte) ---
                BlocoD::RD200(r) => {
                    self.d200 = Some(r);
                    self.correlacao.clear();
                }
                BlocoD::RD201(r) => store_pis!(self.correlacao, r),
                BlocoD::RD205(r) => process_correlations!(
                    docs,
                    ctx,
                    self.current_cnpj,
                    &mut self.correlacao,
                    r,
                    self.d200
                ),

                // --- Registros Solo (D300 e D350) ---
                BlocoD::RD300(r) => process_only_child!(docs, ctx, self.current_cnpj, r),
                BlocoD::RD350(r) => process_only_child!(docs, ctx, self.current_cnpj, r),

                // --- Grupo D500 (Serviços de Comunicação) ---
                BlocoD::RD500(r) => {
                    self.d500 = Some(r);
                    self.correlacao.clear();
                }
                BlocoD::RD501(r) => store_pis!(self.correlacao, r),
                BlocoD::RD505(r) => process_correlations!(
                    docs,
                    ctx,
                    self.current_cnpj,
                    &mut self.correlacao,
                    r,
                    self.d500
                ),

                // --- Grupo D600 (Resumo de Comunicação) ---
                BlocoD::RD600(r) => {
                    self.d600 = Some(r);
                    self.correlacao.clear();
                }
                BlocoD::RD601(r) => store_pis!(self.correlacao, r),
                BlocoD::RD605(r) => process_correlations!(
                    docs,
                    ctx,
                    self.current_cnpj,
                    &mut self.correlacao,
                    r,
                    self.d600
                ),

                // Ignora outros registros (D001, D990, etc.)
                _ => {}
            }
        }
    }
}

// ============================================================================
// --- Bloco F (Demais Documentos e Operações) ---
// ============================================================================

/// Bloco F (Demais Documentos e Operações)
#[derive(Default)]
struct BlocoFExtractor {
    /// CNPJ do estabelecimento atual rastreado via F010
    current_cnpj: Option<Arc<str>>,
}

impl BlocoFExtractor {
    /// Processa os registros do Bloco F para gerar DocsFiscais.
    ///
    /// O Bloco F é composto majoritariamente por registros autônomos (Solo),
    /// onde cada linha gera um documento fiscal completo sem necessidade de um registro pai.
    fn process(&mut self, records: &[BlocoF], ctx: &SpedContext, docs: &mut Vec<DocsFiscais>) {
        for bloco_f in records {
            match bloco_f {
                // F010: Identificação do Estabelecimento (Contexto de CNPJ)
                BlocoF::RF010(r) => capture_cnpj!(self.current_cnpj, r),

                // Registros Solo: Cada um chama DocsBuilder::from_child internamente via macro
                BlocoF::RF100(r) => process_only_child!(docs, ctx, self.current_cnpj, r),
                BlocoF::RF120(r) => process_only_child!(docs, ctx, self.current_cnpj, r),
                BlocoF::RF130(r) => process_only_child!(docs, ctx, self.current_cnpj, r),
                BlocoF::RF150(r) => process_only_child!(docs, ctx, self.current_cnpj, r),
                BlocoF::RF200(r) => process_only_child!(docs, ctx, self.current_cnpj, r),
                BlocoF::RF205(r) => process_only_child!(docs, ctx, self.current_cnpj, r),
                BlocoF::RF210(r) => process_only_child!(docs, ctx, self.current_cnpj, r),
                BlocoF::RF500(r) => process_only_child!(docs, ctx, self.current_cnpj, r),
                BlocoF::RF510(r) => process_only_child!(docs, ctx, self.current_cnpj, r),
                BlocoF::RF525(r) => process_only_child!(docs, ctx, self.current_cnpj, r),
                BlocoF::RF550(r) => process_only_child!(docs, ctx, self.current_cnpj, r),
                BlocoF::RF560(r) => process_only_child!(docs, ctx, self.current_cnpj, r),

                // Braço de fallback para registros do Bloco F que não geram DocsFiscais
                // (Ex: F001, F990, ou registros informativos de créditos/retenções)
                _ => {}
            }
        }
    }
}

// ============================================================================
// Bloco I (Operações Financeiras)
// ============================================================================

/// --- Bloco I (Operações Financeiras) ---
#[derive(Default)]
struct BlocoIExtractor {
    current_cnpj: Option<Arc<str>>,
}

impl BlocoIExtractor {
    fn process(&mut self, records: &[BlocoI], ctx: &SpedContext, docs: &mut Vec<DocsFiscais>) {
        for bloco_i in records {
            match bloco_i {
                BlocoI::RI010(r) => capture_cnpj!(self.current_cnpj, r),
                BlocoI::RI100(r) => process_only_child!(docs, ctx, self.current_cnpj, r),
                _ => {}
            }
        }
    }
}

// ============================================================================
// BLOCO M (Apuração e Ajustes)
// ============================================================================

// ============================================================================
// MANAGER ESPECÍFICO PARA BLOCO M (APURAÇÃO)
// ============================================================================

/*
O código apresentado implementa corretamente o padrão "Best Fit" (Melhor Correspondência) usando
um sistema de pontuação.
Isso é superior a uma chave de hash rígida porque permite encontrar o registro PIS que mais se
assemelha ao registro COFINS, mesmo que algum campo não essencial esteja diferente
(ex: um arredondamento no valor da base ou uma divergência no CST),
desde que seja o "vencedor" na pontuação.
*/

const PESO_NAT_BC: u8 = 10; // Maior peso: NatBC define a natureza do crédito
const PESO_CST: u8 = 5;
const PESO_VL_BC: u8 = 2;

/// Critérios usados tanto para armazenar (PIS) quanto para buscar (COFINS).
/// 'Copy' é barato aqui (u16 + u16 + Decimal de 128bit = ~20 bytes).
#[derive(Debug, Clone, Copy)]
pub struct CreditCriteria {
    pub nat_bc: Option<u16>,
    pub cst: Option<CodigoSituacaoTributaria>,
    pub vl_bc: Option<Decimal>,
}

impl CreditCriteria {
    /// Construtor único: garante que o Decimal esteja normalizado desde o nascimento.
    pub fn new(
        nat_bc: Option<u16>,
        cst: Option<CodigoSituacaoTributaria>,
        vl_bc: Option<Decimal>,
    ) -> Self {
        Self {
            nat_bc,
            cst,
            // Uso de .map funcional para normalizar o valor se ele existir
            vl_bc: vl_bc.map(|mut v| {
                v.normalize_assign();
                v
            }),
        }
    }

    /// Verifica se a base de cálculo é zero ou nula.
    pub fn is_zero_bc(&self) -> bool {
        self.vl_bc.is_none_or(|v| v.is_zero())
    }
}

/// Entrada armazenada no cache.
#[derive(Debug, Clone)]
struct CreditInfo {
    cod_cred: Option<CodigoDoCredito>, // Código do crédito original do PIS
    criteria: CreditCriteria,
    aliq_pis: Option<Decimal>,
    aliq_cofins: Option<Decimal>,
}

impl CreditInfo {
    /// Retorna score mais alto para matches exatos nos campos de maior peso.
    #[inline(always)]
    fn calculate_score(&self, query: &CreditCriteria) -> u8 {
        let mut score = 0;

        if self.criteria.nat_bc.is_some() && self.criteria.nat_bc == query.nat_bc {
            score += PESO_NAT_BC;
        }

        if self.criteria.cst.is_some() && self.criteria.cst == query.cst {
            score += PESO_CST;
        }

        // Para Valor BC, só pontuamos se houver match e o valor NÃO for zero,
        // ou se ambos forem zero mas o match for exato.
        if self.criteria.vl_bc.is_some() && self.criteria.vl_bc == query.vl_bc {
            if query.is_zero_bc() {
                score += 1; // Match de zero é fraco
            } else {
                score += PESO_VL_BC;
            }
        }
        score
    }
}

#[derive(Default, Debug)]
pub struct CreditCorrelationManager {
    /// Chave Primária: COD_CRED para busca rápida.
    cache: HashMap<Option<CodigoDoCredito>, Vec<CreditInfo>>,
    pub correlacao_inexistente: bool,
}

impl CreditCorrelationManager {
    pub fn clear(&mut self) {
        self.cache.clear();
        self.correlacao_inexistente = false;
    }

    /// Armazena (M100/M105) - Permite duplicatas para ter vagas suficientes
    ///
    /// Informações do PIS
    pub fn store(
        &mut self,
        cod_cred: Option<CodigoDoCredito>,
        aliq_pis: Option<Decimal>,
        criteria: CreditCriteria,
    ) {
        // Adicionar nova entrada de PIS
        let info = CreditInfo {
            cod_cred,
            criteria,
            aliq_pis,
            aliq_cofins: None,
        };

        // Sempre adiciona uma nova entrada (push), criando slots disponíveis.
        // O campo 'aliq_cofins' inicia como None, indicando que o slot está livre.
        self.cache.entry(cod_cred).or_default().push(info);
    }

    /// Resolve a alíquota de PIS após leitura de registros de COFINS (M500/M505).
    /// Estratégia:
    /// 1. Tenta encontrar no bucket exato do cod_cred.
    /// 2. Se não encontrar, realiza uma busca global em todos os buckets (Fuzzy Match).
    pub fn resolve(
        &mut self,
        cofins_cod_cred: Option<CodigoDoCredito>,
        aliq_cofins: Option<Decimal>,
        query: CreditCriteria,
        messages: &mut Vec<String>,
    ) -> Option<Decimal> {
        // 1. TENTATIVA LOCAL (Mesmo COD_CRED)
        // Otimização: max_by_key é O(n) e evita alocações.
        if let Some(entry) = self.cache.get_mut(&cofins_cod_cred).and_then(|infos| {
            infos
                .iter_mut()
                .max_by_key(|info| info.calculate_score(&query))
        }) {
            entry.aliq_cofins = aliq_cofins;
            return entry.aliq_pis;
        }

        // 2. Caso não encontre correlação, registra a "falha" para o relatório posterior
        self.correlacao_inexistente = true;

        // 3. Adicionar nova entrada de COFINS
        // pois não foi encontrada correlação entre PIS e Cofins
        let info = CreditInfo {
            cod_cred: cofins_cod_cred,
            criteria: query,
            aliq_pis: None, // Ausência de informação de PIS
            aliq_cofins,
        };

        self.cache.entry(cofins_cod_cred).or_default().push(info);

        let msg = format!(
            "Ausência de Correlação: COFINS cod_cred {:?} -> PIS cod_cred: None \
            (NatBC {:?}, Valor_BC_Cofins {:?})\n",
            cofins_cod_cred.map(|c| c.to_u16()),
            query.nat_bc,
            query.vl_bc
        );
        messages.push(msg);

        None
    }

    /// Gera o relatório formatado como uma String.
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        // Coletar todas as entradas e ordenar primariamente por COD_CRED do PIS
        let mut all_entries: Vec<_> = self.cache.values().flatten().collect();

        // 2. Ordenação Customizada (Dígitos finais primeiro)
        all_entries.sort_unstable_by_key(|credit_info| {
            (
                credit_info.cod_cred.map(|c| c.credito), // 1º: Tipo do crédito (Ex: 01, 08...) - O "YY" do código XYY
                credit_info.cod_cred.map(|c| c.rateio),  // 2º: Tipo de Rateio
                credit_info.criteria.nat_bc,             // 3º: Natureza da BC
                credit_info.criteria.vl_bc,              // 4º: Valor
            )
        });

        writeln!(
            report,
            "\n    === Relatório de Correlação PIS/COFINS (Bloco M) ==="
        )
        .ok();

        let mut current_cod = None;

        for entry in all_entries {
            // Se mudou o COD_CRED, imprime novo cabeçalho unificado
            if current_cod != Some(entry.cod_cred) {
                current_cod = Some(entry.cod_cred);
                let cod_str = entry.cod_cred.map_or("N/I".into(), |v| v.to_string());

                writeln!(
                    report,
                    "-------------------------------------------------------------"
                )
                .ok();
                writeln!(report, "COD_CRED: {:<4}", cod_str).ok();
            }

            let pis_aliq = entry.aliq_pis.map_or("?".into(), |v| format!("{}%", v));
            let cof_aliq = entry
                .aliq_cofins
                .map_or("N/D".into(), |v| format!("{}%", v));
            let nat_bc = entry
                .criteria
                .nat_bc
                .map_or("??".into(), |v| format!("{:>2}", v));
            let cst = entry
                .criteria
                .cst
                .map_or("??".into(), |v| format!("{:02}", v.code()));
            let valor = entry
                .criteria
                .vl_bc
                .map_or("0,00".into(), |v| v.to_formatted_string(DECIMAL_VALOR));

            writeln!(
                report,
                "   PIS: {:>6} | COFINS: {:>6} | CST: {} | NatBC: {} | ValorBC_PIS: {:>13}",
                pis_aliq, cof_aliq, cst, nat_bc, valor
            )
            .ok();
        }
        writeln!(
            report,
            "-------------------------------------------------------------\n"
        )
        .ok();
        report
    }
}

/// BLOCO M (Apuração e Ajustes)
#[derive(Default)]
struct BlocoMExtractor<'a> {
    m100: Option<&'a RegistroM100>,
    m500: Option<&'a RegistroM500>,

    correlacao: CreditCorrelationManager,
}

impl<'a> BlocoMExtractor<'a> {
    /// Lógica de Correlação de Alíquotas entre PIS e COFINS.
    fn resolve_aliq_pis(
        &mut self,
        pai: &RegistroM500,
        filho: &RegistroM505,
        criteria: CreditCriteria,
        messages: &mut Vec<String>,
    ) -> Option<Decimal> {
        // Prioridade: Cache Dinâmico (Realidade do Arquivo)
        // Tenta encontrar um M105 correspondente a este M505.

        // Resultado da correlação entre as alíquotas de PIS e COFINS
        let resultado_pis = self
            .correlacao
            .resolve(pai.cod_cred, pai.aliq_cofins, criteria, messages)
            // 2. Fallback: Tabela Estática (Legislação Padrão)
            .or_else(|| obter_pis_da_tabela_estatica(pai, filho));

        // 3. Verificação e Log de Problema
        if resultado_pis.is_none() {
            let cof_str = pai.aliq_cofins.to_formatted_string(DECIMAL_ALIQ);

            // Usa polimorfismo de exibição para o CodigoDoCredito
            let cod_str = pai.cod_cred.map_or("N/I".into(), |c| c.to_string());

            let msg = format!(
                "M505: Falha na correlação PIS/COFINS (Nenhuma alíquota encontrada).\n\
                 \tDetalhes: CodCred: {:?} | CST: {:?} | NatBC: {:?} | ValorBC: {:?} | AliqCOFINS: {}",
                cod_str, filho.cst_cofins, filho.nat_bc_cred, filho.vl_bc_cofins, cof_str
            );
            messages.push(msg);
        }

        resultado_pis
    }

    fn process(
        &mut self,
        records: &'a [BlocoM],
        ctx: &SpedContext,
        docs: &mut Vec<DocsFiscais>,
        messages: &mut Vec<String>,
    ) {
        for bloco_m in records {
            match bloco_m {
                // M100: Crédito de PIS (Pai do M105)
                BlocoM::RM100(r) => {
                    self.m100 = Some(r);
                    // Mapeador direto para gerar registros fiscais do M100
                    docs.extend(mappers::build_m100(r, ctx));
                }

                // M105: Detalhamento do Crédito de PIS
                BlocoM::RM105(filho) => {
                    // Popula o cache dinâmico com dados do M105 (Filho) e M100 (Pai)
                    // Popula o cache de correlação com dados de PIS para uso posterior no COFINS
                    if let Some(pai) = self.m100 {
                        // Cria dados normalizados de PIS
                        let criteria =
                            CreditCriteria::new(filho.nat_bc_cred, filho.cst_pis, filho.vl_bc_pis);
                        // Armazena dados do PIS
                        self.correlacao.store(pai.cod_cred, pai.aliq_pis, criteria);
                    }
                }

                // M500: Crédito de COFINS (Pai do M505)
                BlocoM::RM500(r) => {
                    self.m500 = Some(r);
                    docs.extend(mappers::build_m500(r, ctx));
                }

                // M505: Detalhamento do Crédito de COFINS
                BlocoM::RM505(filho) => {
                    if let Some(pai) = self.m500 {
                        // Cria dados normalizados de COFINS
                        let criteria = CreditCriteria::new(
                            filho.nat_bc_cred,
                            filho.cst_cofins,
                            filho.vl_bc_cofins,
                        );

                        // Tenta encontrar a alíquota de PIS correspondente
                        let aliq_pis = self.resolve_aliq_pis(pai, filho, criteria, messages);

                        // Constrói o Documento Fiscal consolidado
                        let mut b = DocsBuilder::from_child(ctx, filho, None);

                        // Dados organizacionais da Matriz (Bloco M é centralizado)
                        b.doc.estabelecimento_cnpj = ctx.matriz_estabelecimento_cnpj.clone();
                        b.doc.estabelecimento_nome = ctx.matriz_estabelecimento_nome.clone();

                        b.doc.data_emissao = ctx.periodo_de_apuracao;
                        b.doc.cod_credito = pai.cod_cred;
                        b.doc.natureza_bc =
                            filho.nat_bc_cred.and_then(NaturezaBaseCalculo::from_u16);
                        b.doc.tipo_de_operacao = Some(TipoDeOperacao::Detalhamento);

                        b.doc.aliq_pis = aliq_pis;
                        b.doc.aliq_cofins = pai.aliq_cofins;

                        docs.push(b.build());
                    }
                }

                // Tratamento dos registros de apuração (M210 e M610) inclusive versões antigas
                BlocoM::RM210(_r) => {
                    // Lógica para M210 Novo (se houver construção de Docs)
                }
                BlocoM::RM210Antigo(_r) => {
                    // Lógica para M210 Antigo
                }

                // Outros registros do Bloco M (M400, M800, etc.)
                _ => {}
            }
        }
    }
}

// ============================================================================
// Bloco 1 (Controle de Créditos)
// ============================================================================

/// --- Bloco 1 (Controle de Créditos) ---
#[derive(Default)]
struct Bloco1Extractor {
    /// Embora não utilizado diretamente nos registros 1100/1500 no seu snippet original,
    /// mantemos o estado para consistência com os outros blocos.
    current_cnpj: Option<Arc<str>>,
}

impl Bloco1Extractor {
    fn process(
        &mut self,
        records: &[Bloco1],
        ctx: &SpedContext,
        docs: &mut Vec<DocsFiscais>,
        messages: &mut Vec<String>,
    ) {
        // Reset do estado para garantir que dados de um arquivo não vazem para outro
        self.current_cnpj = None;

        for bloco_1 in records {
            match bloco_1 {
                // Registro 1100: Controle de Créditos Fiscais - PIS/PASEP
                Bloco1::R1100(r) => {
                    // r já é &Registro1100, sem necessidade de downcast_ref
                    if let Some((doc, msg)) = mappers::build_ctrl_credito(
                        r.line_number,
                        "1100",
                        r.vl_cred_desc_efd, // Valor do Crédito descontado neste período de escrituração.
                        r.get_cod_cred(),
                        r.get_per_apu_cred(),
                        ctx,
                    ) {
                        docs.push(doc);
                        messages.push(msg);
                    }
                }

                // Registro 1500: Controle de Créditos Fiscais - COFINS
                Bloco1::R1500(r) => {
                    // r já é &Registro1500
                    if let Some((doc, msg)) = mappers::build_ctrl_credito(
                        r.line_number,
                        "1500",
                        r.vl_cred_desc_efd,
                        r.get_cod_cred(),
                        r.get_per_apu_cred(),
                        ctx,
                    ) {
                        docs.push(doc);
                        messages.push(msg);
                    }
                }

                // Se houver algum registro que capture CNPJ no Bloco 1 (ex: 1010 se aplicável)
                // Bloco1::R1010(r) => capture_cnpj!(self.current_cnpj, r),

                // Todos os outros registros do Bloco 1 (1001, 1011, 1900, etc.)
                // que não geram DocsFiscais ou alteram estado são ignorados aqui.
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
    use crate::{
        DECIMAL_VALOR,
        Tributo::{self, Cofins, Pis},
    };
    use claudiofsr_lib::{FormatStyle, thousands_separator};

    pub fn build_m100(reg: &RegistroM100, ctx: &SpedContext) -> Vec<DocsFiscais> {
        let mut b = DocsBuilder::new(ctx, "M100", reg.line_number, None); // Builder base imutável

        b.doc.estabelecimento_cnpj = ctx.matriz_estabelecimento_cnpj.clone();
        b.doc.estabelecimento_nome = ctx.matriz_estabelecimento_nome.clone();

        // Configuração base (com clone barato pois usa Arc e Option<Copy>)
        let make_base = || {
            let mut builder = b.clone();
            builder.doc.data_emissao = ctx.periodo_de_apuracao;
            builder.doc.aliq_pis = reg.aliq_pis;
            builder.doc.valor_bc = reg.vl_bc_pis;
            builder.doc.cod_credito = reg.cod_cred;
            builder
        };

        // Gera os ajustes funcionalmente
        generate_adjustments(
            make_base(),
            reg.vl_ajus_acres,
            reg.vl_ajus_reduc,
            reg.vl_cred_desc,
        )
    }

    pub fn build_m500(reg: &RegistroM500, ctx: &SpedContext) -> Vec<DocsFiscais> {
        let mut b = DocsBuilder::new(ctx, "M500", reg.line_number, None);

        b.doc.estabelecimento_cnpj = ctx.matriz_estabelecimento_cnpj.clone();
        b.doc.estabelecimento_nome = ctx.matriz_estabelecimento_nome.clone();

        let make_base = || {
            let mut builder = b.clone();
            builder.doc.data_emissao = ctx.periodo_de_apuracao;
            builder.doc.aliq_cofins = reg.aliq_cofins;
            builder.doc.valor_bc = reg.vl_bc_cofins;
            builder.doc.cod_credito = reg.cod_cred;
            builder
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
    ) -> Vec<DocsFiscais> {
        // Mapeamento: (Valor, TipoDeOperacao, Multiplicador de Sinal)
        // 3: Ajuste de Acréscimo (+)
        // 4: Ajuste de Redução (-)
        // 5: Desconto da Contribuição (-)
        [
            (desc, TipoDeOperacao::DescontoNoPeriodo, dec!(-1.0)),
            (reduc, TipoDeOperacao::AjusteReducao, dec!(-1.0)),
            (acres, TipoDeOperacao::AjusteAcrescimo, dec!(1.0)),
        ]
        .into_iter()
        .filter_map(|(val_opt, op, signal)| {
            // Filtra valores > 0.0 (ignorando nulos ou zero)
            val_opt.filter(|v| v.eh_maior_que_zero()).map(|v| {
                let mut builder = base_builder.clone();
                // Aplica valor com o sinal correto (abs * signal)
                builder.doc.valor_item = Some(v.abs() * signal);
                builder.doc.tipo_de_operacao = Some(op);
                builder.build() // Chama o build final aqui
            })
        })
        .collect()
    }

    /// Constrói o documento de Controle de Crédito (1100/1500).
    ///
    /// - Filtra registros do próprio mês (tratados no Bloco M).
    /// - Valida semanticamente o código do crédito (XYY).
    /// - Atribui origem (Importação vs Mercado Interno).
    /// - Define o valor como negativo (desconto).
    ///
    /// Retorna uma tupla: (Documento Opcional, Mensagem de Erro Opcional)
    pub fn build_ctrl_credito(
        line: usize,
        reg_name: &str,
        vl_cred_desc: Option<Decimal>, // Valor do Crédito descontado neste período de escrituração.
        cod_cred: Option<CodigoDoCredito>,
        per_apu: Option<NaiveDate>,
        ctx: &SpedContext,
    ) -> Option<(DocsFiscais, String)> {
        // 1. Validação de Entrada (Fail Fast)
        let codigo_do_credito = cod_cred?;

        // Obter Tributo
        let contribuicao: Tributo = match reg_name {
            "1100" => Pis,
            "1500" => Cofins,
            _ => return None,
        };

        // 2. Filtro de Data (Fail Fast - O(1))
        // Só processa se o Período de Origem for DIFERENTE do Período Atual.
        // Se for igual, significa que é um crédito do próprio mês (já tratado no Bloco M).
        let (pa_origem, pa_atual) = match (per_apu, ctx.periodo_de_apuracao) {
            (Some(orig), Some(curr)) if orig != curr => (orig, curr),
            _ => return None,
        };

        // 3. Validação de Valor: Extrai o valor se for válido (Some e != 0)
        let credito_descontado = vl_cred_desc.map(|c| c.abs()).filter(|c| !c.is_zero())?;

        // 4. Construção do Documento
        let mut b = DocsBuilder::new(ctx, reg_name, line, None);

        // 5. Configuração dos Valores e Tipos
        b.doc.valor_item = Some(-credito_descontado); // Sinal negativo para desconto
        b.doc.cod_credito = cod_cred; // Passa o código bruto para o Builder resolver

        // Tipo 6: Desconto Efetuado em Período Posterior (Conforme Regras SPED)
        b.doc.tipo_de_operacao = Some(TipoDeOperacao::DescontoPosterior);

        // 5. Contexto Organizacional (Matriz)
        b.doc.estabelecimento_cnpj = ctx.matriz_estabelecimento_cnpj.clone();
        b.doc.estabelecimento_nome = ctx.matriz_estabelecimento_nome.clone();

        // 6. Definição Temporal
        // O registro pertence contabilmente ao passado (Origem), mas foi emitido/usado no presente (Emissão).
        b.doc.periodo_de_apuracao = Some(pa_origem);
        b.doc.mes = MesesDoAno::try_from(pa_origem.month()).ok();
        b.doc.ano = Some(pa_origem.year());
        b.doc.trimestre = Some(pa_origem.quarter());
        b.doc.data_emissao = ctx.periodo_de_apuracao;

        let doc = b.build();

        // 7. Validação Final (Segurança)
        // Se o Builder não conseguiu determinar o tipo de crédito (código inválido),
        // retornamos None para ignorar o registro, replicando o comportamento original.
        doc.tipo_de_credito?;

        // 8. Gerar Mensagem (Retorna String)
        let valor_formatado =
            thousands_separator(-credito_descontado, DECIMAL_VALOR, FormatStyle::PtBr);
        let msg = format!(
            "Verificado 'Valor do Crédito descontado neste período de escrituração', \
            porém 'Crédito Apurado em Período de Apuração Anterior'.\n\
            'Valor do Crédito descontado neste período de escrituração' --> Período de Escrituração Atual  = {:02}/{:04}.\n\
            'Crédito Apurado em Período de Apuração Anterior' --> Período de Apuração de Origem do Crédito = {:02}/{:04}.\n\
            Código do Crédito: {}\n\
            Valor das Deduções ou Descontos de {}: {}\n\n",
            pa_atual.month(),
            pa_atual.year(),
            pa_origem.month(),
            pa_origem.year(),
            codigo_do_credito,
            contribuicao,
            valor_formatado
        );

        // Retorna o documento E a mensagem
        Some((doc, msg))
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
#[path = "tests/extractor_tests.rs"]
mod extractor_tests;
