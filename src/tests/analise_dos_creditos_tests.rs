use super::*;

#[test]
fn operacoes_com_vetores() {
    // cargo test -- --show-output operacoes_com_vetores

    let mut vec_a = vec![1, 3, 5];
    println!("vec_a: {vec_a:?}");

    let vec_b = vec![2, 3, 4, 1];
    println!("vec_b: {vec_b:?}\n");

    vec_a.extend(vec_b);
    println!("vec_a: {vec_a:?} extend");

    vec_a.sort_unstable();
    println!("vec_a: {vec_a:?} sort");

    vec_a.dedup();
    println!("vec_a: {vec_a:?} dedup");

    assert_eq!(vec_a, &[1, 2, 3, 4, 5]);
}

#[test]
fn struct_sum() {
    // cargo test
    let val_a = Valores {
        valor_item: dec!(1.2),
        valor_bc: dec!(1.2),
        valor_rbnc_trib: dec!(0.2),
        valor_rbnc_ntrib: dec!(0.2),
        valor_rbnc_exp: dec!(0.2),
        valor_rb_cum: dec!(0.2),
    };
    let val_b = Valores {
        valor_item: dec!(0.3),
        valor_bc: dec!(0.3),
        valor_rbnc_trib: dec!(1.3),
        valor_rbnc_ntrib: dec!(1.3),
        valor_rbnc_exp: dec!(1.3),
        valor_rb_cum: dec!(1.3),
    };
    let val_c = Valores {
        valor_item: dec!(3.5),
        valor_bc: dec!(3.5),
        valor_rbnc_trib: dec!(3.5),
        valor_rbnc_ntrib: dec!(3.5),
        valor_rbnc_exp: dec!(3.5),
        valor_rb_cum: dec!(3.5),
    };
    let val_d = Valores {
        valor_item: dec!(5.0),
        valor_bc: dec!(5.0),
        valor_rbnc_trib: dec!(5.0),
        valor_rbnc_ntrib: dec!(5.0),
        valor_rbnc_exp: dec!(5.0),
        valor_rb_cum: dec!(5.0),
    };
    assert_eq!(val_a + val_b + val_c, val_d);
}

#[test]
fn struct_multiply() {
    // cargo test
    // cargo test -- --show-output struct_mul
    let val_a1 = Valores {
        valor_item: dec!(1.0),
        valor_bc: dec!(1.0),
        valor_rbnc_trib: dec!(1.0),
        valor_rbnc_ntrib: dec!(1.0),
        valor_rbnc_exp: dec!(1.0),
        valor_rb_cum: dec!(1.0),
    };

    let val_a5 = val_a1 * dec!(5.0);
    let val_a7 = val_a1.mul(dec!(7.0));

    let val_b = Valores {
        valor_item: dec!(5.0),
        valor_bc: dec!(5.0),
        valor_rbnc_trib: dec!(5.0),
        valor_rbnc_ntrib: dec!(5.0),
        valor_rbnc_exp: dec!(5.0),
        valor_rb_cum: dec!(5.0),
    };
    let val_c = Valores {
        valor_item: dec!(7.0),
        valor_bc: dec!(7.0),
        valor_rbnc_trib: dec!(7.0),
        valor_rbnc_ntrib: dec!(7.0),
        valor_rbnc_exp: dec!(7.0),
        valor_rb_cum: dec!(7.0),
    };

    assert_eq!(val_a5, val_b);
    assert_eq!(val_a7, val_c);

    println!("val_a1: {val_a1:#?}");
    println!("val_a5: {val_a5:#?}");
    println!("val_a7: {val_a7:#?}");
}

#[test]
fn hashmap_sum_values_by_key() {
    // cargo test -- --show-output hashmap_sum_values_by_key

    let mut docs_fiscais1 = DocsFiscais {
        ..Default::default()
    };
    docs_fiscais1.estabelecimento_cnpj = "12.345.678/0001-23".into();
    docs_fiscais1.tipo_de_operacao = Some(TipoDeOperacao::Entrada); // 1: Entrada
    docs_fiscais1.cst = Some(CodigoSituacaoTributaria::OperacaoComDireitoACreditoVincExclusivamenteAReceitaSaoTributadaNoMI);
    docs_fiscais1.cfop = Some(1234);
    docs_fiscais1.tipo_de_credito = Some(TipoDeCredito::AliquotaBasica);
    docs_fiscais1.natureza_bc = NaturezaBaseCalculo::from_u16(7);
    docs_fiscais1.valor_item = Some(dec!(15.000));
    docs_fiscais1.valor_bc = Some(dec!(10.000));

    let mut docs_fiscais2 = DocsFiscais {
        ..Default::default()
    };
    docs_fiscais2.estabelecimento_cnpj = "12.345.678/0001-23".into();
    docs_fiscais2.tipo_de_operacao = Some(TipoDeOperacao::Entrada); // 1: Entrada
    docs_fiscais2.cst = Some(CodigoSituacaoTributaria::OperacaoComDireitoACreditoVincExclusivamenteAReceitaSaoTributadaNoMI);
    docs_fiscais2.cfop = Some(2345);
    docs_fiscais2.tipo_de_credito = Some(TipoDeCredito::AliquotaBasica);
    docs_fiscais2.natureza_bc = NaturezaBaseCalculo::from_u16(12);
    docs_fiscais2.valor_item = Some(dec!(22.000));
    docs_fiscais2.valor_bc = Some(dec!(8.000));

    let mut docs_fiscais3 = DocsFiscais {
        ..Default::default()
    };
    docs_fiscais3.estabelecimento_cnpj = "12.345.678/0001-23".into();
    docs_fiscais3.tipo_de_operacao = Some(TipoDeOperacao::Entrada); // 1: Entrada
    docs_fiscais3.cst = Some(CodigoSituacaoTributaria::OperacaoComDireitoACreditoVincExclusivamenteAReceitaSaoTributadaNoMI);
    docs_fiscais3.cfop = Some(3456);
    docs_fiscais3.tipo_de_credito = Some(TipoDeCredito::AliquotaBasica);
    docs_fiscais3.natureza_bc = NaturezaBaseCalculo::from_u16(12);
    docs_fiscais3.valor_item = Some(dec!(8.000));
    docs_fiscais3.valor_bc = Some(dec!(2.000));

    let mut docs_fiscais4 = DocsFiscais {
        ..Default::default()
    };
    docs_fiscais4.estabelecimento_cnpj = "12.345.678/0001-23".into();
    docs_fiscais4.tipo_de_operacao = Some(TipoDeOperacao::Entrada); // 1: Entrada
    docs_fiscais4.cst = Some(CodigoSituacaoTributaria::OperacaoComDireitoACreditoVincExclusivamenteAReceitaSaoTributadaNoMI);
    docs_fiscais4.cfop = Some(4567);
    docs_fiscais4.tipo_de_credito = Some(TipoDeCredito::AliquotaBasica);
    docs_fiscais4.natureza_bc = NaturezaBaseCalculo::from_u16(7);
    docs_fiscais4.valor_item = Some(dec!(25.000));
    docs_fiscais4.valor_bc = Some(dec!(18.000));

    let mut docs_fiscais5 = DocsFiscais {
        ..Default::default()
    };
    docs_fiscais5.estabelecimento_cnpj = "12.345.678/0001-23".into();
    docs_fiscais5.tipo_de_operacao = Some(TipoDeOperacao::Entrada); // 1: Entrada
    docs_fiscais5.cst = Some(CodigoSituacaoTributaria::OperacaoComDireitoACreditoVincExclusivamenteAReceitaSaoTributadaNoMI);
    docs_fiscais5.cfop = Some(5678);
    docs_fiscais5.tipo_de_credito = Some(TipoDeCredito::AliquotaBasica);
    docs_fiscais5.natureza_bc = NaturezaBaseCalculo::from_u16(7);
    docs_fiscais5.valor_item = Some(dec!(10.000));
    docs_fiscais5.valor_bc = Some(dec!(12.000));

    let linhas = vec![
        docs_fiscais1,
        docs_fiscais2,
        docs_fiscais3,
        docs_fiscais4,
        docs_fiscais5,
    ];

    let somas_nat: HashMap<Chaves, Valores> = consolidar_registros(
        &linhas,
        |line| line.entrada_de_credito() || line.saida_de_receita_bruta(),
        obter_chaves_valores,
    );

    println!("somas_nat: {somas_nat:#?}");

    let chaves1 = Chaves {
        cnpj_base: "12.345.678".into(),
        tipo_de_operacao: Some(TipoDeOperacao::Entrada),
        tipo_de_credito: Some(TipoDeCredito::AliquotaBasica),
        cst: Some(CodigoSituacaoTributaria::OperacaoComDireitoACreditoVincExclusivamenteAReceitaSaoTributadaNoMI),
        natureza_bc: NaturezaBaseCalculo::from_u16(7),
        ..Default::default()
    };

    let valores1 = Valores::new(Some(dec!(50.00)), Some(dec!(40.00)));

    let chaves2 = Chaves {
        cnpj_base: "12.345.678".into(),
        tipo_de_operacao: Some(TipoDeOperacao::Entrada),
        tipo_de_credito: Some(TipoDeCredito::AliquotaBasica),
        cst: Some(CodigoSituacaoTributaria::OperacaoComDireitoACreditoVincExclusivamenteAReceitaSaoTributadaNoMI),
        natureza_bc: NaturezaBaseCalculo::from_u16(12),
        ..Default::default()
    };

    let valores2 = Valores::new(Some(dec!(30.00)), Some(dec!(10.00)));

    let hashmap = HashMap::from([(chaves1, valores1), (chaves2, valores2)]);

    assert_eq!(hashmap, somas_nat);
}
