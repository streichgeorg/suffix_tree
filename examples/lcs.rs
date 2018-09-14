#[macro_use] extern crate structopt;
extern crate suffix_tree;

use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str;
use structopt::StructOpt;
use suffix_tree::alphabet::Alphabet;
use suffix_tree::longest_common_subsequence;

#[derive(StructOpt)]
struct Options {
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    file_path: Option<PathBuf>,
    #[structopt(short = "a", long = "alphabet")]
    alphabet: Option<String>,
    #[structopt(name = "INPUT")]
    input: Vec<String>,

}

fn main() -> io::Result<()> {
    let options = Options::from_args();

    let owned_sequences: Vec<Vec<u8>> = if let Some(file_path) = options.file_path {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);

        let mut sequences: Vec<Vec<u8>> = Vec::new();
        loop {
            let mut sequence = Vec::new();
            if reader.read_until('\n' as u8, &mut sequence)? == 0 {
                break;
            }

            sequences.push(sequence);
        }

        sequences
    } else {
        options.input.into_iter().map(|s| s.into_bytes()).collect()
    };

    let sequences: Vec<&[u8]> = owned_sequences.iter().map(|ref v| {
        let slice = v.as_slice();
        &slice[..slice.len() - 1]
    }).collect();

    let alphabet = options.alphabet.as_ref().map(|ref s| Alphabet::new(s.as_bytes()));
    match longest_common_subsequence(&sequences, alphabet) {
        Some(sequence) => {
            let text = str::from_utf8(sequence).unwrap_or("<invalid_string>");
            println!("{}", text);
        },
        None => println!("No common subsequence."),
    };

    Ok(())
}
