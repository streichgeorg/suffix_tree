#[macro_use] extern crate structopt;
extern crate suffix_tree;

use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str;
use structopt::StructOpt;
use suffix_tree::longest_common_subsequence;

#[derive(StructOpt)]
struct Options {
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    file_path: Option<PathBuf>,
    #[structopt(name = "INPUT")]
    input: Vec<String>,

}

fn main() -> io::Result<()> {
    let options = Options::from_args();

    let strings = if let Some(file_path) = options.file_path {
        let file = File::open(file_path)?;
        BufReader::new(file).lines().map(|line| line.unwrap().to_owned()).collect()
    } else {
        options.input
    };

    let sequences: Vec<&[u8]> = strings.iter().map(|v| v.as_bytes()).collect();
    match longest_common_subsequence(&sequences) {
        Some(sequence) => {
            let text = str::from_utf8(sequence).unwrap_or("<invalid_string>");
            println!("{}", text);
        },
        None => println!("No common subsequence."),
    };

    Ok(())
}
