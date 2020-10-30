use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
struct Campos {
    nivel: i32,
    reg: f32,
}

#[derive(Default, Debug)]
struct ProductRecord {
    times_bought: i32,
    bloco0000: HashMap<i32, Campos>,
    original_key: String,
}

//let mut registro0001: HashMap<&str, &str> = HashMap::new();

fn main() {
    let mut registros_efd: HashMap<i32, ProductRecord> = HashMap::new();
    let mut bought1: HashMap<i32, Campos> = HashMap::new();
    bought1.insert(2, Campos{nivel: 50, reg: 0.0});
    registros_efd.insert(1, ProductRecord{times_bought: 100, bloco0000: bought1, original_key: "Whatever".into()});

    let mut bought2: HashMap<i32, Campos> = HashMap::new();
    bought2.insert(1, Campos{nivel: 50, reg: 0.0});
    registros_efd.insert(2, ProductRecord{times_bought: 150, bloco0000: bought2, original_key: "Whatever".into()});

    let times_bought: HashMap<i32, f32> = registros_efd.iter().map(|(k,v)| (*k, v.times_bought as f32)).collect();
    for main in registros_efd.values_mut() {
        let a = main.times_bought as f32;

        for (key, value) in main.bloco0000.clone().iter() {
            let other_together = value.nivel as f32;
            let reg = other_together / (a + times_bought[key] - other_together);
            main.bloco0000.get_mut(key).unwrap().reg = reg;
        }
    }
    
    println!("data {:#?}", registros_efd);

}