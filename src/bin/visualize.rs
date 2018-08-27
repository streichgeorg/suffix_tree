#[macro_use] extern crate structopt;
extern crate suffix_tree;

use structopt::StructOpt;
use suffix_tree::SuffixTreeBuilder;

#[derive(StructOpt)]
struct Options {
    #[structopt(name = "INPUT")]
    input: Vec<String>,

}

fn main() {
    let options = Options::from_args();

    let mut tree_builder = SuffixTreeBuilder::new();

    for string in &options.input {
        tree_builder.add_sequence(string.as_bytes());
    }

    tree_builder.tree.pretty_print();
}
