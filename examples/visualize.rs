#[macro_use] extern crate structopt;
extern crate suffix_tree;

use structopt::StructOpt;
use suffix_tree::SuffixTree;
use suffix_tree::alphabet::Alphabet;

#[derive(StructOpt)]
struct Options {
    #[structopt(short = "a", long = "alphabet")]
    alphabet: Option<String>,
    #[structopt(name = "INPUT")]
    strings: Vec<String>,
}

fn main() {
    let options = Options::from_args();

    let alphabet = options.alphabet.as_ref().map(|ref s| Alphabet::new(s.as_bytes()));
    let sequences: Vec<&[u8]> = options.strings.iter().map(|s| s.as_bytes()).collect();

    let output = SuffixTree::from_sequences(&sequences, alphabet).pretty_print();
    println!("{}", output);
}
