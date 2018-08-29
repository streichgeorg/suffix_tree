extern crate suffix_tree;

use suffix_tree::{longest_common_subsequence, SuffixTreeBuilder};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[test]
fn build_suffix_tree() {
    let mut tree_builder = SuffixTreeBuilder::new();

    tree_builder.add_sequence("test".as_bytes());
    tree_builder.add_sequence("builder".as_bytes());
    tree_builder.add_sequence("askdfjlaksjdf".as_bytes());

    let _ = tree_builder.build();
}

#[test]
fn lcs() {
    let sequences = &[
        "testing".as_bytes(),
        "festung".as_bytes(),
        "estland".as_bytes()
    ];
    assert!(longest_common_subsequence(sequences).unwrap() == "est".as_bytes());
}

const CORRECT_RESULT: &str = "\
TATTTGGACCGACCCGCGTAAGGATAGCGAAGGAGTGGTCTAAGATAATGCTGTACTCTCGAATGCCGCCAGGCAGTAGGCGCACCGAAC\
CCATCGCAGCTTCCCAGGGATCCCCACTGGTATATCTCTTGGTAAGGTACTTGCTACTCAGAACCCTACTGGAAGTTGGTGGGGCACAGC\
AGACATGGAACGGACGGGAACGGGGGGTTTTGAGGGGCATGATACTACACATGGAGAATACCTAT\
";
#[test]
fn lcs_codon_sequences() {
    let file = File::open("tests/resources/codon_sequences.txt").unwrap();
    let strings: Vec<String> = BufReader::new(file).lines()
        .map(|line| line.unwrap().to_owned()).collect();

    let sequences: Vec<&[u8]> = strings.iter().map(|v| v.as_bytes()).collect();
    assert!(longest_common_subsequence(&sequences).unwrap() == CORRECT_RESULT.as_bytes());
}
