extern crate suffix_tree;

use suffix_tree::SuffixTreeBuilder;
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
fn longest_common_subsequence() {
    let mut tree_builder = SuffixTreeBuilder::new();

    tree_builder.add_sequence("testing".as_bytes());
    tree_builder.add_sequence("festung".as_bytes());
    tree_builder.add_sequence("estland".as_bytes());

    let mut tree = tree_builder.build();
    let (seq_id, start, end) = tree.longest_common_subsequence().unwrap();

    assert!(&tree.sequence_by_id(seq_id)[start..end] == "est".as_bytes());
}

const CORRECT_RESULT: &str = "\
TATTTGGACCGACCCGCGTAAGGATAGCGAAGGAGTGGTCTAAGATAATG\
CTGTACTCTCGAATGCCGCCAGGCAGTAGGCGCACCGAACCCATCGCAGC\
TTCCCAGGGATCCCCACTGGTATATCTCTTGGTAAGGTACTTGCTACTCA\
GAACCCTACTGGAAGTTGGTGGGGCACAGCAGACATGGAACGGACGGGAA\
CGGGGGGTTTTGAGGGGCATGATACTACACATGGAGAATACCTAT\
";

#[test]
fn lcs_codon_sequences() {
    let mut strings: Vec<String> = Vec::new();
    let file = File::open("tests/resources/codon_sequences.txt").unwrap();
    for line in BufReader::new(file).lines() {
        strings.push(line.unwrap().to_owned());
    }

    let mut tree_builder = SuffixTreeBuilder::new();

    for string in &strings {
        tree_builder.add_sequence(string.as_bytes());
    }

    let mut tree = tree_builder.build();
    let (seq_id, start, end) = tree.longest_common_subsequence().unwrap();
    assert!(&tree.sequence_by_id(seq_id)[start..end] == CORRECT_RESULT.as_bytes());
}
