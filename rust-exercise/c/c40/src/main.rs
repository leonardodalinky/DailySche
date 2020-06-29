// Exercise 40. Binary Search Trees

mod bin_tree;

use bin_tree::BinTree;

fn main() {
    let mut tree: BinTree<i32> = BinTree::new();
    tree.insert(5);
    println!("{}", tree.search(6));
}
