// Exercise 15. Pointers, Dreaded Pointers
// #![feature(ptr_offset_from)]
fn main() {
    let ages: [i32;5] = [23, 43, 12, 89, 2];
    let names = ["Alan", "Frank", "Mary", "John", "List"];

    let count = ages.len();
    for i in 0..count {
        println!("{} has {} years alive.", names[i], ages[i]);
    }

    println!("---");

    let p_ages: *const i32 = ages.as_ptr();
    let p_names: *const &str = names.as_ptr();
    for i in 0..count {
        println!("{} is {} years old.", unsafe{*p_names.add(i)}, unsafe{*p_ages.add(i)});
    }

    println!("---");

    println!("{:#x} and {:#x}", p_ages as u64, p_names as u64);

    // only work on unstable version
    // let mut cur_name = p_names;
    // let mut cur_age = p_ages;
    // unsafe {
    //     while cur_age.offset_from(p_ages) < count as isize {
    //         println!("{} lived {} years so far.", *cur_name, *cur_age);
    //         cur_name = cur_name.add(1);
    //         cur_age = cur_age.add(1);
    //     }
    // }
}
