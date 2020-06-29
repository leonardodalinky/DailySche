mod lists;

use lists::list;

fn main() {
    let mut list: list<i32> = list::new();
    list.insert(0, 1);
    println!("{}", list.getvalue(0));
    println!("{}", list.getvalue(0));
}
