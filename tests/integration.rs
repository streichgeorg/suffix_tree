#[macro_use] extern crate indoc;
extern crate suffix_tree;

use suffix_tree::{longest_common_subsequence, SuffixTree};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[test]
fn build_suffix_tree() {
    let _ = SuffixTree::from_sequences(&[b"test", b"builder", b"asdfkljasdlf"]);
}

#[test]
fn pretty_print() {
    let expected = indoc!(
        "┳t┳est$0
         ┃ ┣$0
         ┃ ┗$1
         ┣$0
         ┣rest$1
         ┣est┳$0
         ┃   ┗$1
         ┣st┳$0
         ┃  ┗$1
         ┗$1"
    );

    let tree = SuffixTree::from_sequences(&[b"test", b"rest"]);

    assert_eq!(tree.pretty_print(), expected);
}

#[test]
fn lcs() {
    let expected = b"est";

    let sequences: &[&[u8]] = &[
        b"testing",
        b"festung",
        b"estland"
    ];

    assert_eq!(longest_common_subsequence(sequences).unwrap(), expected);
}

#[test]
fn lcs_codon_sequences() {
    let expected: &[u8] = indoc!(
        b"TATTTGGACCGACCCGCGTAAGGATAGCGAAGGAGTGGTCTAAGATAATGCTGTACTCTCGA\
          ATGCCGCCAGGCAGTAGGCGCACCGAACCCATCGCAGCTTCCCAGGGATCCCCACTGGTATA\
          TCTCTTGGTAAGGTACTTGCTACTCAGAACCCTACTGGAAGTTGGTGGGGCACAGCAGACAT\
          GGAACGGACGGGAACGGGGGGTTTTGAGGGGCATGATACTACACATGGAGAATACCTAT"
    );

    let file = File::open("tests/resources/codon_sequences.txt").unwrap();
    let strings: Vec<String> = BufReader::new(file).lines()
        .map(|line| line.unwrap().to_owned()).collect();
    let sequences: Vec<&[u8]> = strings.iter().map(|v| v.as_bytes()).collect();

    assert_eq!(longest_common_subsequence(&sequences).unwrap(), expected);
}

