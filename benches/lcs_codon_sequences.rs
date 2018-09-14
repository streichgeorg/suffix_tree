#[macro_use] extern crate criterion;
extern crate suffix_tree;

use criterion::Criterion;
use std::fs::File;
use std::io::{BufRead, BufReader};
use suffix_tree::longest_common_subsequence;
use suffix_tree::alphabet::Alphabet;


fn setup() -> Vec<Vec<u8>> {
    let file = File::open("benches/resources/codon_sequences.txt").unwrap();
    let mut reader = BufReader::new(file);

    let mut sequences: Vec<Vec<u8>> = Vec::new();
    loop {
        let mut sequence = Vec::new();
        if reader.read_until('\n' as u8, &mut sequence).unwrap() == 0 {
            break;
        }

        sequences.push(sequence);
    }

    sequences
}

fn compute(strings: Vec<Vec<u8>>) {
    let sequences: Vec<&[u8]> = strings.iter().map(|v| {
        let slice = v.as_slice();
        &slice[..slice.len() - 1]
    }).collect();
    let alphabet = Alphabet::new(b"ATGC");
    let _ = longest_common_subsequence(&sequences, Some(alphabet)).unwrap();
}

fn benchmark(c: &mut Criterion) {
    c.bench_function("lcs codon sequence", move |b| {
        b.iter_with_large_setup(|| setup(), |strings| compute(strings));
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
