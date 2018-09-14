#[macro_use] extern crate indoc;
extern crate suffix_tree;

use suffix_tree::{longest_common_subsequence, SuffixTree};
use suffix_tree::alphabet::Alphabet;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[test]
fn build_suffix_tree() {
    let _ = SuffixTree::from_sequences(&[b"test", b"builder", b"asdfkljasdlf"], None);
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

    let tree = SuffixTree::from_sequences(&[b"test", b"rest"], None);

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

    assert_eq!(longest_common_subsequence(sequences, None).unwrap(), expected);
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
    let mut reader = BufReader::new(file);

    let mut owned_sequences: Vec<Vec<u8>> = Vec::new();
    loop {
        let mut sequence = Vec::new();
        if reader.read_until('\n' as u8, &mut sequence).unwrap() == 0 {
            break;
        }

        owned_sequences.push(sequence);
    }

    let alphabet = Alphabet::new(b"ATGC");

    let sequences: Vec<&[u8]> = owned_sequences.iter().map(|s| {
        let slice = s.as_slice();
        &slice[..slice.len() - 1]
    }).collect();
    assert_eq!(longest_common_subsequence(&sequences, Some(alphabet)).unwrap(), expected);
}
