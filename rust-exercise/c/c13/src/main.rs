// Exercise 13. For-Loops and Arrays of Strings
use std::env;
use std::collections::linked_list;

fn main() {
    let envargs: Vec<String> = env::args().collect();
    if envargs.len() != 2 {
        panic!("ERROR: You need an argument.\n");
    }
    let str2: &String = &envargs[1];
    for c in str2.chars() {
        match c {
            'a' | 'A' => println!("{}: 'A'", c),
            'e' | 'E' => println!("{}: 'E'", c),
            'i' | 'I' => println!("{}: 'I'", c),
            'o' | 'O' => println!("{}: 'O'", c),
            'u' | 'U' => println!("{}: 'U'", c),
            _ => println!("{} isn't vowel", c)
        }
    }
}
