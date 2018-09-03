#[macro_use] extern crate criterion;
extern crate suffix_tree;

use criterion::Criterion;
use std::fs::File;
use std::io::{BufRead, BufReader};
use suffix_tree::longest_common_subsequence;


fn setup() -> Vec<String> {
    let file = File::open("benches/resources/codon_sequences.txt").unwrap();
    BufReader::new(file).lines().map(|line| line.unwrap().to_owned()).collect()
}

fn compute(strings: Vec<String>) {
    let sequences: Vec<&[u8]> = strings.iter().map(|v| v.as_bytes()).collect();
    let _ = longest_common_subsequence(&sequences).unwrap();
}

fn benchmark(c: &mut Criterion) {
    c.bench_function("lcs codon sequence", move |b| {
        b.iter_with_large_setup(|| setup(), |strings| compute(strings));
    });
}

criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = benchmark
}

criterion_main!(benches);
