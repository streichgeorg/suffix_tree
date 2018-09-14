#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate smallvec;

pub mod alphabet;

use alphabet::Alphabet;
use smallvec::SmallVec;
use std::cell::Cell;
use std::collections::HashMap;
use std::iter;
use std::str;
use std::u8;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
enum Symbol {
    Terminal(usize),
    Regular(u8),
}

type SequenceId = usize;

#[derive(Copy, Clone)]
struct Sequence<'a> {
    id: SequenceId,
    data: &'a [u8],
}

impl <'a> Sequence<'a> {
    fn new(id: SequenceId, data: &'a [u8]) -> Sequence {
        Sequence { id, data }
    }

    fn len(&self) -> usize {
        self.data.len() + 1
    }

    fn at(&self, index: usize) -> Symbol {
        if index == self.data.len() {
            Symbol::Terminal(self.id)
        } else {
            Symbol::Regular(self.data[index])
        }
    }

    fn substring(&self, start: usize, maybe_end: Option<usize>) -> String {
        let end = maybe_end.unwrap_or_else(|| self.data.len());
        let substr = str::from_utf8(&self.data[start..end]).unwrap_or("<invalid_string>");

        if maybe_end.is_none() {
            format!("{}${}", substr, self.id)
        } else {
            substr.to_owned()
        }
    }
}

type NodeId = usize;

struct ChildMap {
    terminals: HashMap<usize, NodeId>,
    regular: SmallVec<[Option<NodeId>; 4]>,
}

impl ChildMap {
    fn new(alphabet_size: u8) -> ChildMap {
        ChildMap {
            terminals: HashMap::new(),
            regular: smallvec![None; alphabet_size as usize],
        }
    }

    fn add_child(&mut self, alphabet: &Alphabet, symbol: Symbol, child: NodeId) {
        match symbol {
            Symbol::Terminal(seq_id) => {
                self.terminals.insert(seq_id, child);
            }
            Symbol::Regular(symbol) => {
                let rank = alphabet.rank_of_symbol(symbol);
                self.regular[rank as usize] = Some(child);
            }
        }
    }

    fn get_child(&self, alphabet: &Alphabet, symbol: Symbol) -> Option<NodeId> {
        match symbol {
            Symbol::Terminal(seq_id) => self.terminals.get(&seq_id).cloned(),
            Symbol::Regular(symbol) =>{
                let rank = alphabet.rank_of_symbol(symbol);
                self.regular[rank as usize]
            }
        }
    }

    fn iter<'s>(&'s self) -> Box<Iterator<Item = NodeId> + 's> {
        let terminals_iter = self.terminals.values().cloned();
        let regular_iter = self.regular.iter().filter_map(|&v| v);

        Box::new(terminals_iter.chain(regular_iter))
    }
}

struct RootNode {
    children: ChildMap,
}

struct InternalNode {
    seq_id: SequenceId,
    start: usize,
    end: usize,
    children: ChildMap,
    suffix_link: Option<NodeId>,
    sequence_id_set: Cell<Option<u128>>,
}

struct LeafNode {
    seq_id: SequenceId,
    start: usize,
}

enum Node {
    Root(RootNode),
    Internal(InternalNode),
    Leaf(LeafNode),
}

impl Node {
    fn new_root(alphabet_size: u8) -> Node {
        Node::Root(RootNode { children: ChildMap::new(alphabet_size) })
    }

    fn new_internal(alphabet_size: u8, seq_id: SequenceId, start: usize, end: usize) -> Node {
        Node::Internal(InternalNode {
            seq_id,
            start,
            end,
            children: ChildMap::new(alphabet_size),
            suffix_link: None,

            sequence_id_set: Cell::new(None),
        })
    }

    fn new_leaf(seq_id: SequenceId, start: usize) -> Node {
        Node::Leaf(LeafNode { seq_id, start })
    }

    fn children(&self) -> Option<&ChildMap> {
        match *self {
            Node::Root(RootNode { ref children, .. }) |
            Node::Internal(InternalNode { ref children, .. }) => Some(children),
            Node::Leaf(_) => None,
        }
    }

    fn children_mut(&mut self) -> Option<&mut ChildMap> {
        match *self {
            Node::Root(RootNode { ref mut children, .. }) |
            Node::Internal(InternalNode { ref mut children, .. }) => Some(children),
            Node::Leaf(_) => None,
        }
    }

    fn add_child(&mut self, alphabet: &Alphabet, symbol: Symbol, child: NodeId) {
        let children = self.children_mut().unwrap();
        children.add_child(alphabet, symbol, child);
    }

    fn get_child(&self, alphabet: &Alphabet, symbol: Symbol) -> Option<NodeId> {
        let children = self.children().unwrap();
        children.get_child(alphabet, symbol)
    }

    fn is_leaf(&self) -> bool {
        if let Node::Leaf(_) = *self {
            true
        } else {
            false
        }
    }
}

pub struct SuffixTree<'a, 'b> {
    alphabet: Alphabet<'b>,
    sequences: Vec<Sequence<'a>>,
    nodes: Vec<Node>, 
}

impl<'a, 'b> SuffixTree<'a, 'b> {
    fn new(maybe_alphabet: Option<Alphabet<'b>>) -> SuffixTree<'a, 'b> {
        let alphabet = maybe_alphabet.unwrap_or_else(|| alphabet::ASCII.clone());
        let alphabet_size = alphabet.size;

        SuffixTree {
            alphabet,
            sequences: Vec::new(),
            nodes: vec![Node::new_root(alphabet_size)],
        }
    }

    pub fn from_sequence(sequence: &'a [u8], alphabet: Option<Alphabet<'b>>) -> SuffixTree<'a, 'b> {
        let mut tree_builder = SuffixTreeBuilder::new(alphabet);
        tree_builder.add_sequence(sequence);
        tree_builder.build()
    }

    pub fn from_sequences(sequences: &'a[&'a [u8]], alphabet: Option<Alphabet<'b>>)
        -> SuffixTree<'a, 'b>
    {
        let mut tree_builder = SuffixTreeBuilder::new(alphabet);
        for sequence in sequences {
            tree_builder.add_sequence(sequence);
        }
        tree_builder.build()
    }

    pub fn pretty_print(&self) -> String {
        fn _pretty_print<'a, 'b>(tree: &SuffixTree<'a, 'b>, node: NodeId) -> Vec<String> {
            let text = match tree.nodes[node] {
                Node::Root(_) => {
                    "".to_owned()
                },
                Node::Internal(InternalNode { seq_id, start, end, .. }) => {
                    tree.sequences[seq_id].substring(start, Some(end))
                },
                Node::Leaf(LeafNode { seq_id, start, .. }) => {
                    tree.sequences[seq_id].substring(start, None)
                },
            };

            if let Some(child_map) = tree.nodes[node].children() {
                let indent = " ".repeat(text.len());

                let mut children: Vec<NodeId> = child_map.iter().collect();
                children.sort();

                let mut lines = Vec::new();
                for (i, &child) in children.iter().enumerate() {
                    for (j, line) in _pretty_print(tree, child).into_iter().enumerate() {
                        let line = match (i, j) {
                            (0, 0)                           => format!("{}┳{}", text, line),
                            (_, 0) if i < children.len() - 1 => format!("{}┣{}", indent, line),
                            (_, _) if i < children.len() - 1 => format!("{}┃{}", indent, line),
                            (_, 0)                           => format!("{}┗{}", indent, line),
                            (_, _)                           => format!("{} {}", indent, line),
                        };

                        lines.push(line);
                    }
                }

                lines
            } else {
                vec![text]
            }
        }

        _pretty_print(&self, 0).join("\n")
    }

    pub fn sequence_by_id(&self, seq_id: SequenceId) -> &'a [u8] {
        self.sequences[seq_id].data
    }

    fn add_sequence(&mut self, data: &'a [u8]) {
        let seq_id = self.sequences.len();
        assert!(seq_id < 128, "this suffix tree contains more than 128 sequences");

        let sequence = Sequence::new(seq_id, data);
        self.sequences.push(sequence);
    }

    fn current_sequence(&self) -> Sequence {
        self.sequences[self.sequences.len() - 1]
    }

    fn add_node(&mut self, node: Node) -> NodeId {
        let node_id = self.nodes.len();
        self.nodes.push(node);

        node_id
    }

    fn add_child(&mut self, parent: NodeId, symbol: Symbol, child: NodeId) {
        self.nodes[parent].add_child(&self.alphabet, symbol, child);
    }

    fn get_child(&self, parent: NodeId, symbol: Symbol) -> Option<NodeId> {
        self.nodes[parent].get_child(&self.alphabet, symbol)
    }

    fn root_node(&self) -> &RootNode {
        if let Node::Root(ref node) = self.nodes[0] {
            node
        } else {
            panic!();
        }
    }

    fn internal_node(&self, node_id: NodeId) -> Option<&InternalNode> {
        if let Node::Internal(ref node) = self.nodes[node_id] {
            Some(node)
        } else {
            None
        }
    }

    fn internal_node_mut(&mut self, node_id: NodeId) -> Option<&mut InternalNode> {
        if let Node::Internal(ref mut node) = self.nodes[node_id] {
            Some(node)
        } else {
            None
        }
    }

    fn prepare_lcs(&self) {
        fn _prepare_lcs<'b, 'c>(tree: &SuffixTree<'b, 'c>, node: NodeId) -> u128 {
            match tree.nodes[node] {
                Node::Root(_) => panic!(),
                Node::Internal(InternalNode { ref children, ref sequence_id_set, .. }) => {
                    let mut id_set = 0;
                    for child in children.iter() {
                        id_set |= _prepare_lcs(tree, child);
                    }

                    sequence_id_set.set(Some(id_set));

                    id_set
                },
                Node::Leaf(LeafNode { seq_id, .. }) => 1 << seq_id,
            }
        }

        for child in self.root_node().children.iter() {
            _prepare_lcs(self, child);
        }
    }

    /// Returns all occurences of the longest common subsequence in suffix tree.
    /// If there are multiple such subsequences it just returns the occurences
    /// of a random one.
    ///
    /// #Examples
    /// ```
    /// use suffix_tree::SuffixTree;
    ///
    /// let mut tree = SuffixTree::from_sequences(&[b"test", b"rest", b"estland"], None);
    /// let mut occurences = tree.longest_common_subsequence();
    /// for (seq_id, start, end) in occurences {
    ///     assert_eq!(&tree.sequence_by_id(seq_id)[start..end], b"est")
    /// }
    /// ```
    pub fn longest_common_subsequence<'s>(&'s self)
        -> Box<Iterator<Item = (SequenceId, usize, usize)> + 's>
    {
        fn _longest_common_subsequence<'a, 'b>(
            tree: &SuffixTree<'a, 'b>,
            node: NodeId, depth: usize
        ) -> Option<(NodeId, usize)> {
            match tree.nodes[node] {
                Node::Internal(InternalNode {
                    start,
                    end,
                    ref sequence_id_set,
                    ref children, 
                    ..
                }) => {
                    let all_bits_set = u128::max_value() >> (128 - tree.sequences.len());
                    if sequence_id_set.get().unwrap() != all_bits_set {
                        return None;
                    }

                    let edge_length = end - start;
                    children.iter().filter_map(|child| {
                        _longest_common_subsequence(tree, child, depth + edge_length)
                    }).max_by_key(|&(_, depth)| {
                        depth
                    }).or_else(|| {
                        Some((node, depth + edge_length))
                    })
                },
                Node::Leaf(_) => None,
                Node::Root(_) => panic!(),
            }
        }

        let maybe_node = self.root_node().children.iter().filter_map(|child| {
            _longest_common_subsequence(self, child, 0)
        }).max_by_key(|&(_, depth)| depth);

        if let Some((node, depth)) = maybe_node {
            let edge_length = {
                let internal = self.internal_node(node).unwrap();
                internal.end - internal.start
            };

            Box::new(self.node_occurences(node, 0).map(move |(seq_id, position)| {
                let end = position + edge_length;
                let start = end - depth;
                (seq_id, start, end) 
            }))
        } else {
            Box::new(iter::empty())
        }
    }


    /// Returns true when the given pattern is contained in the suffix tree. 
    ///
    /// #Examples
    /// ```
    /// use suffix_tree::SuffixTree;
    ///
    /// let tree = SuffixTree::from_sequence(b"test", None);
    ///
    /// assert!(tree.contains(b"es"));
    /// assert!(!tree.contains(b"asdf"));
    /// ```
    pub fn contains(&self, pattern: &[u8]) -> bool {
        self.find_node(pattern).is_some()
    }

    /// Returns all the occurences of the given pattern in the suffix tree. 
    ///
    /// #Examples
    /// ```
    /// use suffix_tree::SuffixTree;
    ///
    /// let tree = SuffixTree::from_sequence(b"test", None);
    /// let mut occurences = tree.find(b"es");
    /// assert_eq!(occurences.next(), Some((0, 1, 3)));
    /// assert_eq!(occurences.next(), None);
    /// ```
    pub fn find<'s, 'c>(&'s self, pattern: &'c [u8])
        -> Box<Iterator<Item = (SequenceId, usize, usize)> + 's>
    {
        if let Some((node, remaining)) = self.find_node(pattern) {
            let pattern_len = pattern.len();

            Box::new(self.node_occurences(node, 0).map(move |(seq_id, position)| {
                let end = position + remaining;
                let start = end - pattern_len;
                (seq_id, start, end) 
            }))
        } else {
            Box::new(iter::empty())
        }
    }

    fn node_occurences<'s>(&'s self, node: NodeId, depth: usize)
        -> Box<Iterator<Item = (SequenceId, usize)> + 's>
    {
        match self.nodes[node] {
            Node::Root(_) => Box::new(iter::empty()),
            Node::Internal(InternalNode { start, end, ref children, .. }) => {
                let edge_length = end - start;

                Box::new(children.iter().flat_map(move |child| {
                    self.node_occurences(child, depth + edge_length)
                }))
            },
            Node::Leaf(LeafNode { seq_id, start, .. }) => {
                Box::new(iter::once((seq_id, start - depth)))
            }
        }
    }

    fn find_node(&self, pattern: &[u8]) -> Option<(NodeId, usize)> {
        let mut current_node = 0;
        let mut remaining = pattern.len();

        loop {
            let depth = pattern.len() - remaining;
            let next_symbol = Symbol::Regular(pattern[depth]);

            if let Some(child) = self.get_child(current_node, next_symbol) {
                let label = match self.nodes[child] {
                    Node::Root(_) => panic!(),
                    Node::Internal(InternalNode { seq_id, start, end, .. }) => {
                        &self.sequences[seq_id].data[start..end]
                    },
                    Node::Leaf(LeafNode { seq_id, start, .. }) => {
                        &self.sequences[seq_id].data[start..]
                    }
                };

                current_node = child;

                if remaining < label.len() {
                    if pattern[depth..] != label[..remaining] {
                        return None;
                    } else {
                        return Some((current_node, remaining));
                    }
                } else if &pattern[depth..(depth + label.len())] != label {
                    return None;
                } else {
                    if remaining == label.len() {
                        return Some((current_node, remaining));
                    } else if self.nodes[current_node].is_leaf() {
                        return None;
                    }

                    remaining -= label.len();
                }
            } else {
                return None;
            }
        }
    }
}

pub struct SuffixTreeBuilder<'a, 'b> {
    tree: SuffixTree<'a, 'b>,

    active_node: NodeId,
    active_edge: Option<(Symbol, usize)>,

    position: usize,
    remaining: usize,

    previously_created_node: Option<NodeId>,
}

impl<'a, 'b> SuffixTreeBuilder<'a, 'b> {
    pub fn new(alphabet: Option<Alphabet<'b>>) -> SuffixTreeBuilder<'a, 'b> {
        SuffixTreeBuilder {
            tree: SuffixTree::new(alphabet),
            active_node: 0,
            active_edge: None,
            position: 0,
            remaining: 0,
            previously_created_node: None
        }
    }

    pub fn build(self) -> SuffixTree<'a, 'b> {
        self.tree.prepare_lcs();
        self.tree
    }

    pub fn add_sequence(&mut self, sequence: &'a [u8]) {
        self.tree.add_sequence(sequence);

        self.position = 0;
        self.remaining = 0;

        self.active_node = 0;
        self.active_edge = None;

        for _ in 0..self.tree.current_sequence().len() {
            self.insert_next_symbol();
        }
    }

    fn insert_next_symbol(&mut self) {
        self.remaining += 1;
        self.previously_created_node = None;

        let next_symbol = self.tree.current_sequence().at(self.position);
        for _ in 0..self.remaining {
            if self.insert_node(next_symbol) {
                self.update_active_point();
                self.remaining -= 1;
            } else {
                self.active_edge = match self.active_edge {
                    Some((symbol, length)) => Some((symbol, length + 1)),
                    None => Some((next_symbol, 1)),
                };
                self.normalize_active_point();
                break;
            }
        }

        self.position += 1;
    }

    fn insert_node(&mut self, next_symbol: Symbol) -> bool {
        match self.active_edge {
            Some((symbol, length)) => self.insert_internal_node(next_symbol, symbol, length),
            None => self.insert_leaf_node(next_symbol),
        }
    }

    fn insert_leaf_node(&mut self, next_symbol: Symbol) -> bool {
        let insert_node = self.tree.get_child(self.active_node, next_symbol).is_none();

        if insert_node {
            let leaf_node = Node::new_leaf(self.tree.current_sequence().id, self.position);
            let leaf_node_id = self.tree.add_node(leaf_node);
            self.tree.add_child(self.active_node, next_symbol, leaf_node_id);

            if self.active_node != 0 {
                let active_node = self.active_node;
                self.set_suffix_link(active_node);
            }
        }

        insert_node
    }

    fn insert_internal_node(
        &mut self,
        next_symbol: Symbol,
        active_symbol: Symbol,
        active_length: usize
    ) -> bool {
        let active_edge_node = self.active_edge_node();
        let (active_seq_id, active_start) = match self.tree.nodes[active_edge_node] {
            Node::Internal(InternalNode { seq_id, start, .. })
            | Node::Leaf(LeafNode { seq_id, start }) => (seq_id, start),
            Node::Root(_) => panic!(),
        };
        let split_position = active_start + active_length;

        let insert_node = self.tree.sequences[active_seq_id].at(split_position) != next_symbol;

        if insert_node {
            match self.tree.nodes[active_edge_node] {
                Node::Root(_) => panic!(),
                Node::Internal(InternalNode { ref mut start, .. }) |
                Node::Leaf(LeafNode { ref mut start, .. }) => {
                   *start = split_position;
                }
            };

            let node_a = {
                let node = Node::new_internal(
                    self.tree.alphabet.size,
                    active_seq_id,
                    active_start,
                    split_position
                );

                self.tree.add_node(node)
            };
            self.tree.add_child(self.active_node, active_symbol, node_a);

            let symbol_at_split = self.tree.sequences[active_seq_id].at(split_position);
            self.tree.add_child(node_a, symbol_at_split, active_edge_node);

            let node_b = {
                let seq_id = self.tree.current_sequence().id;
                let start = self.position;
                self.tree.add_node(Node::new_leaf(seq_id, start))
            };
            self.tree.add_child(node_a, next_symbol, node_b);

            self.set_suffix_link(node_a);
            self.previously_created_node = Some(node_a);
        }

        insert_node
    }

    fn update_active_point(&mut self) {
        match self.tree.nodes[self.active_node] {
            Node::Root(_) => {
                if let Some((_, length)) = self.active_edge {
                    self.active_edge = Some((
                        self.tree.current_sequence().at(self.position + 2 - self.remaining),
                        length - 1
                    ));
                };
            },
            Node::Internal(InternalNode { suffix_link: Some(node), .. }) => {
                self.active_node = node;
            },
            Node::Internal(_) | Node::Leaf(_) => {
                self.active_node = 0;
                self.active_edge = Some((
                    self.tree.current_sequence().at(self.position + 2 - self.remaining),
                    self.remaining - 2 
                ));
            }
        }

        self.normalize_active_point();
    }

    fn active_edge_lenght(&self) -> usize {
        match self.tree.nodes[self.active_edge_node()] {
            Node::Root(_) => panic!(),
            Node::Internal(InternalNode { start, end, .. }) => end - start,
            Node::Leaf(LeafNode { seq_id, start, .. }) => {
                let offset = (seq_id == self.tree.current_sequence().id) as usize;
                (self.tree.sequences[seq_id].len() + offset) - start
            }
        }
    }

    fn normalize_active_point(&mut self) {
        loop {
            match self.active_edge {
                Some((_, 0)) => {
                    self.active_edge = None;
                    break;
                },
                Some((_, active_length)) => {
                    let edge_length = self.active_edge_lenght();
                    if active_length < edge_length {
                        break;
                    } else if active_length == edge_length {
                        self.active_node = self.active_edge_node();
                        self.active_edge = None;
                        break;
                    } else {
                        self.active_node = self.active_edge_node();
                        let active_symbol_index = self.position - active_length + edge_length;
                        self.active_edge = Some((
                            self.tree.current_sequence().at(active_symbol_index),
                            active_length - edge_length
                        ))
                    }
                },
                None => break,
            };
        }
    }

    fn set_suffix_link(&mut self, link_to: NodeId) {
        if let Some(node) = self.previously_created_node {
            self.tree.internal_node_mut(node).unwrap().suffix_link = Some(link_to);
        }

        self.previously_created_node = None;
    }

    fn active_edge_node(&self) -> NodeId {
        let (active_symbol, _) = self.active_edge.unwrap();
        self.tree.get_child(self.active_node, active_symbol).unwrap()
    }

    #[allow(dead_code)]
    fn print_ukkonen_state(&self) {
        println!("active_node is {}, active_edge is {:?}", self.active_node, self.active_edge);
        println!("position is {}, remaining is {}", self.position, self.remaining);
    }
}

pub fn longest_common_subsequence<'a>(sequences: &'a [&'a [u8]], alphabet: Option<Alphabet>)
    -> Option<&'a [u8]>
{
    let tree = SuffixTree::from_sequences(sequences, alphabet);
    let result: Option<(SequenceId, usize, usize)> = tree.longest_common_subsequence()
        .take(1).last();

    result.map(|(seq_id, start, end)| {
        &tree.sequence_by_id(seq_id)[start..end]
    })
}
