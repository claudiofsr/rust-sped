// https://erasin.wang/books/easy-rust/Chapter_24.html

enum ThingsInTheSky {
    Sun(String), // Now each variant has a string
    Stars(String),
}

fn create_skystate(time: i32) -> ThingsInTheSky {
    match time {
        6..=18 => ThingsInTheSky::Sun(String::from("I can see the sun!")), // Write the strings here
             _ => ThingsInTheSky::Stars(String::from("I can see the stars!")),
    }
}

fn check_skystate(state: &ThingsInTheSky) {
    match state {
        ThingsInTheSky::Sun(description) => print_description(description.to_string()), // Give the string the name description so we can use it
        ThingsInTheSky::Stars(n)         => print_description(n.to_string()),           // Or you can name it n. Or anything else - it doesn't matter
    }
}

fn print_description (description: String){
    println!("{}", description);
}

fn main() {
    let time = 8; // it's 8 o'clock
    let skystate = create_skystate(time); // create_skystate returns a ThingsInTheSky
    check_skystate(&skystate); // Give it a reference so it can read the variable skystate
}