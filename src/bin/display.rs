extern crate suffix_tree;

use suffix_tree::SuffixTree;

fn main() {
    let s = "When the symbol we want to add to the tree is already on the edge, we, according to Observation 1, update only active point and remainder, leaving the tree unchanged. BUT if there is an internal node marked as needing suffix link, we must connect that node with our current active node through a suffix link.";
    // let s = "add$";
    let mut tree = SuffixTree::new(s.as_bytes());
    println!("{}", s);
    for _ in 0..s.len() {
        tree.step();
        tree.visualize();
        println!("------");
    }

}
