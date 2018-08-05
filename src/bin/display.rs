extern crate suffix_tree;

use suffix_tree::SuffixTree;

fn main() {
    let tree = SuffixTree::new("banana".to_owned());
    println!("{}", tree);
}
