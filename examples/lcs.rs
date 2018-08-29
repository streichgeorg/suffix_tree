#[macro_use] extern crate structopt;
extern crate suffix_tree;

use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str;
use structopt::StructOpt;
use suffix_tree::SuffixTreeBuilder;

#[derive(StructOpt)]
struct Options {
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    file_path: Option<PathBuf>,
    #[structopt(name = "INPUT")]
    input: Vec<String>,

}

fn main() -> io::Result<()> {
    let options = Options::from_args();
    let mut strings: Vec<String> = Vec::new();
    let mut tree_builder = SuffixTreeBuilder::new();

    if let Some(file_path) = options.file_path {
        let file = File::open(file_path)?;
        for line in BufReader::new(file).lines() {
            strings.push(line.unwrap().to_owned());
        }
    } else {
        strings = options.input;
    }

    for string in &strings {
        tree_builder.add_sequence(string.as_bytes());
    }

    let mut tree = tree_builder.build();

    match tree.longest_common_subsequence() {
        Some((seq_id, start, end)) => {
            let text = str::from_utf8(&tree.sequence_by_id(seq_id)[start..end])
                .unwrap_or("<invalid_string>");
            println!("{}", text);
        },
        None => println!("No common subsequence."),
    };

    Ok(())
}
