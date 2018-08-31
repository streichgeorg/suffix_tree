extern crate suffix_tree;

use suffix_tree::{longest_common_subsequence, SuffixTreeBuilder};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[test]
fn build_suffix_tree() {
    let mut tree_builder = SuffixTreeBuilder::new();

    tree_builder.add_sequence(b"test");
    tree_builder.add_sequence(b"builder");
    tree_builder.add_sequence(b"askdfjlaksjdf");

    let _ = tree_builder.build();
}

#[test]
fn lcs() {
    let sequences: &[&[u8]] = &[
        b"testing",
        b"festung",
        b"estland"
    ];

    assert!(longest_common_subsequence(sequences).unwrap() == b"est");
}

const CORRECT_RESULT: &[u8] = b"\
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
    assert!(longest_common_subsequence(&sequences).unwrap() == CORRECT_RESULT);
}
