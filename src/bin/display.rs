extern crate suffix_tree;

use suffix_tree::SuffixTree;

fn main() {
    let s = "abcabxabcd";
    let mut tree = SuffixTree::new(s.as_bytes());
    for _ in 0..10 {
        tree.step();
    }

    println!("{:#?}", tree);
}
