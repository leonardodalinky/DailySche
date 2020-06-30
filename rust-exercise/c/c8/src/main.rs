use std::env;

fn main() {
    let mut args = env::args();
    if args.len() == 1 {
        println!("You only have one argument. You suck.");
    }
    else if args.len() > 1 && args.len() < 4 {
        println!("Here's your arguments:");
        for i in args {
            println!("{}", i);
        }
    }
    else{
        println!("You have too many arguments. You suck.");
    }
}
