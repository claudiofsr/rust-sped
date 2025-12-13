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
        valor_item: 1.2,
        valor_bc: 1.2,
        valor_rbnc_trib: 0.2,
        valor_rbnc_ntrib: 0.2,
        valor_rbnc_exp: 0.2,
        valor_rb_cum: 0.2,
    };
    let val_b = Valores {
        valor_item: 0.3,
        valor_bc: 0.3,
        valor_rbnc_trib: 1.3,
        valor_rbnc_ntrib: 1.3,
        valor_rbnc_exp: 1.3,
        valor_rb_cum: 1.3,
    };
    let val_c = Valores {
        valor_item: 3.5,
        valor_bc: 3.5,
        valor_rbnc_trib: 3.5,
        valor_rbnc_ntrib: 3.5,
        valor_rbnc_exp: 3.5,
        valor_rb_cum: 3.5,
    };
    let val_d = Valores {
        valor_item: 5.0,
        valor_bc: 5.0,
        valor_rbnc_trib: 5.0,
        valor_rbnc_ntrib: 5.0,
        valor_rbnc_exp: 5.0,
        valor_rb_cum: 5.0,
    };
    assert_eq!(val_a + val_b + val_c, val_d);
}

#[test]
fn struct_multiply() {
    // cargo test
    // cargo test -- --show-output struct_mul
    let val_a1 = Valores {
        valor_item: 1.0,
        valor_bc: 1.0,
        valor_rbnc_trib: 1.0,
        valor_rbnc_ntrib: 1.0,
        valor_rbnc_exp: 1.0,
        valor_rb_cum: 1.0,
    };

    let val_a5 = val_a1 * 5.0;
    let val_a7 = val_a1.mul(7.0);

    let val_b = Valores {
        valor_item: 5.0,
        valor_bc: 5.0,
        valor_rbnc_trib: 5.0,
        valor_rbnc_ntrib: 5.0,
        valor_rbnc_exp: 5.0,
        valor_rb_cum: 5.0,
    };
    let val_c = Valores {
        valor_item: 7.0,
        valor_bc: 7.0,
        valor_rbnc_trib: 7.0,
        valor_rbnc_ntrib: 7.0,
        valor_rbnc_exp: 7.0,
        valor_rb_cum: 7.0,
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
    docs_fiscais1.estabelecimento_cnpj = "12.345.678/0001-23".to_string();
    docs_fiscais1.tipo_de_operacao = Some(TipoOperacao::Entrada); // 1: Entrada
    docs_fiscais1.cst = Some(51);
    docs_fiscais1.cfop = Some(1234);
    docs_fiscais1.tipo_de_credito = Some(TipoDeCredito::AliquotaBasica);
    docs_fiscais1.natureza_bc = Some(7);
    docs_fiscais1.valor_item = Some(15.000);
    docs_fiscais1.valor_bc = Some(10.000);

    let mut docs_fiscais2 = DocsFiscais {
        ..Default::default()
    };
    docs_fiscais2.estabelecimento_cnpj = "12.345.678/0001-23".to_string();
    docs_fiscais2.tipo_de_operacao = Some(TipoOperacao::Entrada); // 1: Entrada
    docs_fiscais2.cst = Some(51);
    docs_fiscais2.cfop = Some(2345);
    docs_fiscais2.tipo_de_credito = Some(TipoDeCredito::AliquotaBasica);
    docs_fiscais2.natureza_bc = Some(12);
    docs_fiscais2.valor_item = Some(22.000);
    docs_fiscais2.valor_bc = Some(8.000);

    let mut docs_fiscais3 = DocsFiscais {
        ..Default::default()
    };
    docs_fiscais3.estabelecimento_cnpj = "12.345.678/0001-23".to_string();
    docs_fiscais3.tipo_de_operacao = Some(TipoOperacao::Entrada); // 1: Entrada
    docs_fiscais3.cst = Some(51);
    docs_fiscais3.cfop = Some(3456);
    docs_fiscais3.tipo_de_credito = Some(TipoDeCredito::AliquotaBasica);
    docs_fiscais3.natureza_bc = Some(12);
    docs_fiscais3.valor_item = Some(8.000);
    docs_fiscais3.valor_bc = Some(2.000);

    let mut docs_fiscais4 = DocsFiscais {
        ..Default::default()
    };
    docs_fiscais4.estabelecimento_cnpj = "12.345.678/0001-23".to_string();
    docs_fiscais4.tipo_de_operacao = Some(TipoOperacao::Entrada); // 1: Entrada
    docs_fiscais4.cst = Some(51);
    docs_fiscais4.cfop = Some(4567);
    docs_fiscais4.tipo_de_credito = Some(TipoDeCredito::AliquotaBasica);
    docs_fiscais4.natureza_bc = Some(7);
    docs_fiscais4.valor_item = Some(25.000);
    docs_fiscais4.valor_bc = Some(18.000);

    let mut docs_fiscais5 = DocsFiscais {
        ..Default::default()
    };
    docs_fiscais5.estabelecimento_cnpj = "12.345.678/0001-23".to_string();
    docs_fiscais5.tipo_de_operacao = Some(TipoOperacao::Entrada); // 1: Entrada
    docs_fiscais5.cst = Some(51);
    docs_fiscais5.cfop = Some(5678);
    docs_fiscais5.tipo_de_credito = Some(TipoDeCredito::AliquotaBasica);
    docs_fiscais5.natureza_bc = Some(7);
    docs_fiscais5.valor_item = Some(10.000);
    docs_fiscais5.valor_bc = Some(12.000);

    let linhas = vec![
        docs_fiscais1,
        docs_fiscais2,
        docs_fiscais3,
        docs_fiscais4,
        docs_fiscais5,
    ];

    let somas_nat: HashMap<Chaves, Valores> = consolidar_chaves(&linhas);

    println!("somas_nat: {somas_nat:#?}");

    let chaves1 = Chaves {
        cnpj_base: "12.345.678".to_string(),
        ano: None,
        trimestre: None,
        mes: None,
        tipo_de_operacao: Some(TipoOperacao::Entrada),
        tipo_de_credito: Some(TipoDeCredito::AliquotaBasica),
        cst: Some(51),
        cfop: None,
        aliq_pis: None,
        aliq_cofins: None,
        natureza_bc: Some(7),
    };

    let valores1 = Valores::new(Some(50.00), Some(40.00));

    let chaves2 = Chaves {
        cnpj_base: "12.345.678".to_string(),
        ano: None,
        trimestre: None,
        mes: None,
        tipo_de_operacao: Some(TipoOperacao::Entrada),
        tipo_de_credito: Some(TipoDeCredito::AliquotaBasica),
        cst: Some(51),
        cfop: None,
        aliq_pis: None,
        aliq_cofins: None,
        natureza_bc: Some(12),
    };

    let valores2 = Valores::new(Some(30.00), Some(10.00));

    let hashmap = HashMap::from([(chaves1, valores1), (chaves2, valores2)]);

    assert_eq!(hashmap, somas_nat);
}
