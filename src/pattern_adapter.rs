/*
Padrão Adapter

O *Padrão Adapter* é um padrão de design de software que permite que objetos com interfaces
incompatíveis trabalhem juntos. Ele atua como um intermediário, convertendo a interface de
uma classe para outra, sem alterar o código existente.

*Importância:*
- Permite a reutilização de código.
- Facilita a integração de sistemas ou bibliotecas com interfaces diferentes.
- Segue o princípio de design "aberto/fechado" (Open/Closed Principle).

*Exemplo em Rust:*

Suponha que temos uma interface `Duck` e uma struct `MallardDuck` que a implementa.
Agora, queremos usar uma struct `Turkey` (com interface diferente) como se fosse um `Duck`.
*/

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//
//
// cargo test -- --help
// cargo test -- --nocapture
// cargo test -- --show-output

/// Run tests with:
/// cargo test -- --show-output parser_tests
#[cfg(test)]
mod pattern_adapter_tests {
    // Interface Duck
    trait Duck {
        fn quack(&self);
        fn fly(&self);
    }

    // MallardDuck implementa Duck
    #[derive(Default, Debug)]
    struct MallardDuck;

    impl Duck for MallardDuck {
        fn quack(&self) {
            println!("Duck: Quack");
        }
        fn fly(&self) {
            println!("Duck: Flying");
        }
    }

    // Interface Turkey
    trait Turkey {
        fn gobble(&self);
        fn fly(&self);
    }

    // Turkey existente
    #[derive(Default, Debug)]
    struct WildTurkey;

    impl Turkey for WildTurkey {
        fn gobble(&self) {
            println!("Turkey: Gobble");
        }
        fn fly(&self) {
            println!("Turkey: Flying short distance");
        }
    }

    // Adapter: Turkey -> Duck
    #[derive(Default, Debug)]
    struct TurkeyAdapter<T: Turkey> {
        turkey: T,
    }

    impl<T: Turkey> Duck for TurkeyAdapter<T> {
        fn quack(&self) {
            self.turkey.gobble();
        }
        fn fly(&self) {
            for _ in 0..5 {
                self.turkey.fly();
            }
        }
    }

    #[test]
    /// cargo test -- --show-output run_pattern
    fn run_pattern() {
        let duck = MallardDuck;
        println!("duck: {duck:?}");
        duck.quack(); // Quack
        duck.fly(); // Flying

        let turkey = WildTurkey;
        println!("turkey: {turkey:?}");
        let turkey_adapter = TurkeyAdapter { turkey };
        println!("turkey_adapter: {turkey_adapter:?}");
        turkey_adapter.quack(); // Gobble
        turkey_adapter.fly(); // Flying short distance (5x)
    }
}

/*
O `TurkeyAdapter` permite usar um `Turkey` como um `Duck`, adaptando a interface.
Isso facilita a integração sem modificar as classes originais.
*/
