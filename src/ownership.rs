// https://www.youtube.com/watch?v=G7OyygheR4c&t=1629s
// Rust para Embarcadores

fn main (){
    let mut s = String::from("hello");
    //let s = String::from("hello");
    let num = 42;

    //recebe_ownership(s,num);
    //recebe_ownership(s.clone(),num);
    //recebe_referencia(&s);
    recebe_referencia_mut(&mut s);

    println!("s = {}, num = {}", s, num);
}

#[allow(dead_code)]
fn recebe_ownership (_s: String, _num: usize) {
    // agora essa função é dona da String 's',
    // 's' será liberada no final dessa função.
}

#[allow(dead_code)]
fn recebe_referencia (_s: String) {
    // recebe uma referência imutável

    // erro:
    //_s.push_str(" world");
}

#[allow(dead_code)]
fn recebe_referencia_mut (_s: &mut String) {
    // referência mutável
    _s.push_str(" world");
}