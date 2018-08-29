extern crate suffix_tree;

use suffix_tree::SuffixTreeBuilder;

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





