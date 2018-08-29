extern crate suffix_tree;
#[macro_use] extern crate criterion;

use criterion::Criterion;
use std::fs::File;
use std::io::{BufRead, BufReader};
use suffix_tree::SuffixTreeBuilder;


fn setup() -> Vec<String> {
    let mut strings: Vec<String> = Vec::new();
    let file = File::open("benches/resources/codon_sequences.txt").unwrap();
    for line in BufReader::new(file).lines() {
        strings.push(line.unwrap().to_owned());
    }

    strings
}

fn compute_and_check(strings: Vec<String>) {
    let mut tree_builder = SuffixTreeBuilder::new();

    for string in &strings {
        tree_builder.add_sequence(string.as_bytes());
    }

    let mut tree = tree_builder.build();
    let _ = tree.longest_common_subsequence();
}

fn benchmark(c: &mut Criterion) {
    c.bench_function("lcs codon sequence", move |b| {
        b.iter_with_large_setup(|| setup(), |strings| compute_and_check(strings));
    });
}

criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = benchmark
}
criterion_main!(benches);
