extern crate bit_vec;

use bit_vec::BitVec;
use std::borrow::Cow;
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
        let end = maybe_end.unwrap_or(self.data.len());
        let substr = str::from_utf8(&self.data[start..end]).unwrap_or("<invalid_string>");

        if maybe_end.is_none() {
            format!("{}${}", substr, self.id)
        } else {
            substr.to_owned()
        }
    }
}

type NodeId = usize;

struct RootNode {
    children: HashMap<Symbol, NodeId>,
}

struct InternalNode {
    seq_id: SequenceId,
    start: usize,
    end: usize,
    children: HashMap<Symbol, NodeId>,
    suffix_link: Option<NodeId>,

    sequence_id_set: Option<BitVec>,
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
    fn new_root() -> Node {
        Node::Root(RootNode { children: HashMap::new() })
    }

    fn new_internal(seq_id: SequenceId, start: usize, end: usize) -> Node {
        Node::Internal(InternalNode {
            seq_id,
            start,
            end,
            children: HashMap::new(),
            suffix_link: None,

            sequence_id_set: None,
        })
    }

    fn new_leaf(seq_id: SequenceId, start: usize) -> Node {
        Node::Leaf(LeafNode { seq_id, start })
    }

    fn children(&self) -> Option<&HashMap<Symbol, NodeId>> {
        match self {
            &Node::Root(RootNode { ref children, .. }) |
            &Node::Internal(InternalNode { ref children, .. }) => Some(children),
            &Node::Leaf(_) => None,
        }
    }

    fn children_mut(&mut self) -> Option<&mut HashMap<Symbol, NodeId>> {
        match self {
            &mut Node::Root(RootNode { ref mut children, .. }) |
            &mut Node::Internal(InternalNode { ref mut children, .. }) => Some(children),
            &mut Node::Leaf(_) => None,
        }
    }

    fn add_child(&mut self, symbol: Symbol, child: NodeId) {
        let children = self.children_mut().unwrap();
        children.insert(symbol, child);
    }

    fn get_child(&self, symbol: Symbol) -> Option<NodeId> {
        let children = self.children().unwrap();
        children.get(&symbol).map(|&v| v)
    }

    fn is_leaf(&self) -> bool {
        if let &Node::Leaf(_) = self {
            true
        } else {
            false
        }
    }
}

pub struct SuffixTree<'a> {
    sequences: Vec<Sequence<'a>>,
    nodes: Vec<Node>, 

    prepared_lcs: bool,
}

impl<'a> SuffixTree<'a> {
    fn new() -> SuffixTree<'a> {
        SuffixTree {
            sequences: Vec::new(),
            nodes: vec![Node::new_root()],

            prepared_lcs: false,
        }
    }

    pub fn from_sequence(sequence: &'a [u8]) -> SuffixTree {
        let mut tree_builder = SuffixTreeBuilder::new();
        tree_builder.add_sequence(sequence);
        tree_builder.build()
    }

    pub fn from_sequences(sequences: &'a[&'a [u8]]) -> SuffixTree {
        let mut tree_builder = SuffixTreeBuilder::new();
        for sequence in sequences {
            tree_builder.add_sequence(sequence);
        }
        tree_builder.build()
    }

    pub fn pretty_print(&self) -> String {
        fn _pretty_print<'a>(tree: &SuffixTree<'a>, node: NodeId) -> Vec<String> {
            let text = match &tree.nodes[node] {
                &Node::Root(_) => {
                    "".to_owned()
                },
                &Node::Internal(InternalNode { seq_id, start, end, .. }) => {
                    tree.sequences[seq_id].substring(start, Some(end))
                },
                &Node::Leaf(LeafNode { seq_id, start, .. }) => {
                    tree.sequences[seq_id].substring(start, None)
                },
            };

            if let Some(child_map) = tree.nodes[node].children() {
                let indent = " ".repeat(text.len());

                let mut children: Vec<NodeId> = child_map.values().map(|&v| v).collect();
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
        self.sequences.push(Sequence::new(seq_id, data));
    }

    fn current_sequence(&self) -> Sequence {
        self.sequences[self.sequences.len() - 1]
    }

    fn add_node(&mut self, node: Node) -> NodeId {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    fn root_node(&self) -> &RootNode {
        if let &Node::Root(ref node) = &self.nodes[0] {
            node
        } else {
            panic!();
        }
    }

    fn internal_node(&self, node_id: NodeId) -> Option<&InternalNode> {
        if let &Node::Internal(ref node) = &self.nodes[node_id] {
            Some(node)
        } else {
            None
        }
    }

    fn internal_node_mut(&mut self, node_id: NodeId) -> Option<&mut InternalNode> {
        if let &mut Node::Internal(ref mut node) = &mut self.nodes[node_id] {
            Some(node)
        } else {
            None
        }
    }

    fn prepare_lcs(&mut self) {
        assert!(!self.prepared_lcs);

        fn _prepare_lcs<'a, 'b>(tree: &'a mut SuffixTree<'b>, node: NodeId) -> Cow<'a, BitVec> {
            if tree.internal_node(node).is_some() {
                let children: Vec<_> = tree.internal_node(node).unwrap()
                    .children.values().cloned().collect();

                let id_set = {
                    let mut id_set = BitVec::from_elem(tree.sequences.len(), false);
                    for child in children {
                        id_set.union(&_prepare_lcs(tree, child));
                    }

                    id_set
                };

                tree.internal_node_mut(node).unwrap().sequence_id_set = Some(id_set);
            }

            match &tree.nodes[node] {
                &Node::Internal(InternalNode{ sequence_id_set: Some(ref id_set), .. }) => {
                    Cow::Borrowed(id_set)
                },
                &Node::Leaf(LeafNode { seq_id, .. }) => {
                    let mut id_set = BitVec::from_elem(tree.sequences.len(), false);
                    id_set.set(seq_id, true);
                    Cow::Owned(id_set)
                },
                _ => panic!(),
            }
        }

        let children: Vec<_> = self.root_node().children.values().cloned().collect();
        for child in children {
            _prepare_lcs(self, child);
        }

        self.prepared_lcs = true;
    }

    /// Returns all occurences of the longest common subsequence in suffix tree.
    /// If there are multiple such subsequences it just returns the occurences
    /// of a random one.
    ///
    /// #Examples
    /// ```
    /// use suffix_tree::SuffixTree;
    ///
    /// let mut tree = SuffixTree::from_sequences(&[b"test", b"rest", b"estland"]);
    /// let mut occurences = tree.longest_common_subsequence();
    /// for (seq_id, start, end) in occurences {
    ///     assert_eq!(&tree.sequence_by_id(seq_id)[start..end], b"est")
    /// }
    /// ```
    pub fn longest_common_subsequence<'s>(&'s self)
        -> Box<Iterator<Item = (SequenceId, usize, usize)> + 's>
    {
        assert!(self.prepared_lcs);

        fn _longest_common_subsequence<'a>(tree: &SuffixTree<'a>, node: NodeId, depth: usize)
            -> Option<(NodeId, usize)>
        {
            match &tree.nodes[node] {
                &Node::Internal(InternalNode {
                    start,
                    end,
                    sequence_id_set: Some(ref id_set),
                    ref children, 
                    ..
                }) => {
                    if !id_set.all() {
                        return None;
                    }

                    let edge_length = end - start;
                    children.values().filter_map(|&child| {
                        _longest_common_subsequence(tree, child, depth + edge_length)
                    }).max_by_key(|&(_, depth)| {
                        depth
                    }).or_else(|| {
                        Some((node, depth + edge_length))
                    })
                },
                &Node::Leaf(_) => None,
                _ => panic!(),
            }
        }

        let maybe_node = self.root_node().children.values().filter_map(|&child| {
            let result = _longest_common_subsequence(self, child, 0);
            result
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
    /// let tree = SuffixTree::from_sequence(b"test");
    ///
    /// assert!(tree.contains(b"es"));
    /// assert!(!tree.contains(b"asdf"));
    /// ```
    pub fn contains(&self, pattern: &[u8]) -> bool {
        if let Some(_) = self.find_node(pattern) {
            true
        } else {
            false
        }
    }

    /// Returns all the occurences of the given pattern in the suffix tree. 
    ///
    /// #Examples
    /// ```
    /// use suffix_tree::SuffixTree;
    ///
    /// let tree = SuffixTree::from_sequence(b"test");
    /// let mut occurences = tree.find(b"es");
    /// assert_eq!(occurences.next(), Some((0, 1, 3)));
    /// assert_eq!(occurences.next(), None);
    /// ```
    pub fn find<'s, 'b>(&'s self, pattern: &'b [u8])
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
        match &self.nodes[node] {
            &Node::Root(_) => Box::new(iter::empty()),
            &Node::Internal(InternalNode { start, end, ref children, .. }) => {
                let edge_length = end - start;

                Box::new(children.values().flat_map(move |&child| {
                    self.node_occurences(child, depth + edge_length)
                }))
            },
            &Node::Leaf(LeafNode { seq_id, start, .. }) => {
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

            if let Some(&child) = self.nodes[current_node].children().unwrap().get(&next_symbol) {
                let label = match &self.nodes[child] {
                    &Node::Root(_) => panic!(),
                    &Node::Internal(InternalNode { seq_id, start, end, .. }) => {
                        &self.sequences[seq_id].data[start..end]
                    },
                    &Node::Leaf(LeafNode { seq_id, start, .. }) => {
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

pub struct SuffixTreeBuilder<'a> {
    tree: SuffixTree<'a>,

    active_node: NodeId,
    active_edge: Option<(Symbol, usize)>,

    position: usize,
    remaining: usize,

    previously_created_node: Option<NodeId>,
}

impl<'a> SuffixTreeBuilder<'a> {
    pub fn new() -> SuffixTreeBuilder<'a> {
        SuffixTreeBuilder {
            tree: SuffixTree::new(),
            active_node: 0,
            active_edge: None,
            position: 0,
            remaining: 0,
            previously_created_node: None
        }
    }

    pub fn build(mut self) -> SuffixTree<'a> {
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
        let insert_node = self.tree.nodes[self.active_node].get_child(next_symbol).is_none();

        if insert_node {
            let leaf_node = Node::new_leaf(self.tree.current_sequence().id, self.position);
            let leaf = self.tree.add_node(leaf_node);
            self.tree.nodes[self.active_node].add_child(next_symbol, leaf);

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
        let (active_seq_id, active_start) = match &self.tree.nodes[active_edge_node] {
            &Node::Internal(InternalNode { seq_id, start, .. })
            | &Node::Leaf(LeafNode { seq_id, start }) => (seq_id, start),
            &Node::Root(_) => panic!(),
        };
        let split_position = active_start + active_length;

        let insert_node = self.tree.sequences[active_seq_id].at(split_position) != next_symbol;

        if insert_node {
            match &mut self.tree.nodes[active_edge_node] {
                &mut Node::Root(_) => panic!(),
                &mut Node::Internal(InternalNode { ref mut start, .. }) |
                &mut Node::Leaf(LeafNode { ref mut start, .. }) => {
                   *start = split_position;
                }
           };

            let node_a = self.tree.add_node(Node::new_internal(
                active_seq_id,
                active_start,
                split_position
            ));

            self.tree.nodes[self.active_node].add_child(active_symbol, node_a);

            self.tree.nodes[node_a].add_child(
                self.tree.sequences[active_seq_id].at(split_position),
                active_edge_node
            );

            let node_b = {
                let seq_id = self.tree.current_sequence().id;
                let start = self.position;
                self.tree.add_node(Node::new_leaf(seq_id, start))
            };
            self.tree.nodes[node_a].add_child(next_symbol, node_b);

            self.set_suffix_link(node_a);
            self.previously_created_node = Some(node_a);
        }

        insert_node
    }

    fn update_active_point(&mut self) {
        match &self.tree.nodes[self.active_node] {
            &Node::Root(_) => {
                if let Some((_, length)) = self.active_edge {
                    self.active_edge = Some((
                        self.tree.current_sequence().at(self.position + 2 - self.remaining),
                        length - 1
                    ));
                };
            },
            &Node::Internal(InternalNode { suffix_link: Some(node), .. }) => {
                self.active_node = node;
            },
            &Node::Internal(_) | &Node::Leaf(_) => {
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
        match &self.tree.nodes[self.active_edge_node()] {
            &Node::Root(_) => panic!(),
            &Node::Internal(InternalNode { start, end, .. }) => end - start,
            &Node::Leaf(LeafNode { seq_id, start, .. }) => {
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
        self.tree.nodes[self.active_node].get_child(active_symbol).unwrap()
    }

    #[allow(dead_code)]
    fn print_ukkonen_state(&self) {
        println!("active_node is {}, active_edge is {:?}", self.active_node, self.active_edge);
        println!("position is {}, remaining is {}", self.position, self.remaining);
    }
}

pub fn longest_common_subsequence<'a>(sequences: &'a [&'a [u8]]) -> Option<&'a [u8]> {
    let tree = SuffixTree::from_sequences(sequences);
    let result: Option<(SequenceId, usize, usize)> = tree.longest_common_subsequence()
        .take(1).last().clone();

    result.map(|(seq_id, start, end)| {
        &tree.sequence_by_id(seq_id)[start..end]
    })
}
