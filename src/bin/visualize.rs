#[macro_use] extern crate structopt;
extern crate suffix_tree;

use structopt::StructOpt;
use suffix_tree::SuffixTree;

#[derive(StructOpt, Debug)]
struct Options {
    #[structopt(short = "i", long = "intermediate")]
    intermediate: bool,
    #[structopt(name = "INPUT")]
    input: Vec<String>,

}

fn main() {
    let options = Options::from_args();

    let mut tree = SuffixTree::new();

    for string in &options.input {
        tree.add_string(string.as_bytes());
    }

    tree.visualize();
}
