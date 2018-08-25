#[macro_use] extern crate structopt;
extern crate suffix_tree;

use structopt::StructOpt;
use suffix_tree::SuffixTree;

#[derive(StructOpt, Debug)]
struct Options {
    #[structopt(short = "i", long = "intermediate")]
    intermediate: bool,
    #[structopt(name = "INPUT")]
    input: String,

}

fn main() {
    let options = Options::from_args();

    let s = format!("{}$", options.input);
    let mut tree = SuffixTree::new();

    tree.add_string(&s.as_bytes());

    tree.visualize();
}
