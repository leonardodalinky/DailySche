fn main() {
    let distance: i32 = 100;
    let power: f32 = 2.345;
    let super_power: f64 = 56789.4532;
    let initial: char = 'A';
    let first_name: &'static str = "Zed";
    let last_name: &str = "Shaw";

    println!("You are {} miles away.", distance);
    println!("You have {} levels of power.", power);
    println!("You have {} awesome super powers.", super_power);
    println!("I have an initial {}.", initial);
    println!("I have a first name {}.", first_name);
    println!("I have a last name {}.", last_name);
    println!("My whole name is {} {}. {}.", first_name, initial, last_name);

    let bugs: i32 = 100;
    let bug_rate: f64 = 1.2;

    println!("You have {} bugs at the imaginary rate of {}.", bugs, bug_rate);

    let universe_of_defects: i64 = 1 * 1024 * 1024 * 1024;
    println!("The entire universe has {} bugs.", universe_of_defects);

    let expected_bugs: f64 = bugs as f64 * bug_rate;
    println!("You are expected to have {} bugs.", expected_bugs);

    let part_of_universe: f64 = expected_bugs / universe_of_defects as f64;
    println!("That is only a {} portion of the universe.", part_of_universe);

    let nul_byte = '\0';
    let care_percentage = bugs * nul_byte as i32;
    println!("Which means you should care {}.",care_percentage);
}
