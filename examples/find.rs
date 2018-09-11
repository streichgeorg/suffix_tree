#[macro_use] extern crate structopt;
extern crate suffix_tree;

use structopt::StructOpt;
use suffix_tree::SuffixTree;

#[derive(StructOpt)]
struct Options {
    #[structopt(name = "STRING")]
    string: String,
    #[structopt(name = "PATTERN")]
    pattern: String,
}

fn main() {
    let options = Options::from_args();
    let tree = SuffixTree::from_sequence(&options.string.as_bytes());

    for (_, start, end) in tree.find(&options.pattern.as_bytes()) {
        println!("{} {}", start, end);
    }
}
