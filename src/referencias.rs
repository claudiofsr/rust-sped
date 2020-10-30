fn main () {

    let mut x: i32 = 10;
    passar_referencia_de_uma_variavel_mutavel(&mut x);

    x += 5;

    println!("1: x = {}", x);
}

fn passar_referencia_de_uma_variavel_mutavel(v: &mut i32){
    *v += 1;
    println!("2: x = {:?}", v);
}
