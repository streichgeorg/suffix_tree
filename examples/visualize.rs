#[macro_use] extern crate structopt;
extern crate suffix_tree;

use structopt::StructOpt;
use suffix_tree::SuffixTree;

#[derive(StructOpt)]
struct Options {
    #[structopt(name = "INPUT")]
    strings: Vec<String>,
}

fn main() {
    let options = Options::from_args();
    let sequences: Vec<&[u8]> = options.strings.iter().map(|s| s.as_bytes()).collect();
    SuffixTree::from_sequences(&sequences).pretty_print();
}
