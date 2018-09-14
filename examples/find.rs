#[macro_use] extern crate structopt;
extern crate suffix_tree;

use structopt::StructOpt;
use suffix_tree::SuffixTree;
use suffix_tree::alphabet::Alphabet;

#[derive(StructOpt)]
struct Options {
    #[structopt(short = "a", long = "alphabet")]
    alphabet: Option<String>,
    #[structopt(name = "STRING")]
    string: String,
    #[structopt(name = "PATTERN")]
    pattern: String,
}

fn main() {
    let options = Options::from_args();

    let alphabet = options.alphabet.as_ref().map(|ref s| Alphabet::new(s.as_bytes()));

    let tree = SuffixTree::from_sequence(&options.string.as_bytes(), alphabet);
    for (_, start, end) in tree.find(&options.pattern.as_bytes()) {
        println!("{} {}", start, end);
    }
}
