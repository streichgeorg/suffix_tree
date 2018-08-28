extern crate bit_vec;

use bit_vec::BitVec;
use std::borrow::Cow;
use std::collections::HashMap;
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

    fn string_repr_internal(&self, start: usize, end: usize) -> String {
        str::from_utf8(&self.data[start..end]).unwrap_or("<invalid_string>").to_owned()
    }

    fn string_repr_leaf(&self, start: usize) -> String {
        let text = str::from_utf8(&self.data[start..]).unwrap_or("<invalid_string>");
        format!("{}${}", text, self.id)
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

    fn add_child(&mut self, symbol: Symbol, child: NodeId) {
        match self {
            &mut Node::Internal(InternalNode { ref mut children, .. }) |
            &mut Node::Root(RootNode { ref mut children, .. }) => children.insert(symbol, child),
            &mut Node::Leaf(_) => unreachable!(),
        };
    }

    fn get_child(&self, symbol: Symbol) -> Option<NodeId> {
        match self {
            &Node::Root(RootNode { ref children, .. }) |
            &Node::Internal(InternalNode { ref children, .. }) => children.get(&symbol).map(|&v| v),
            &Node::Leaf(_) => unreachable!(),
        }
    }
}

pub struct SuffixTree<'a> {
    sequences: Vec<Sequence<'a>>,
    nodes: Vec<Node>, 

    prepared_lcs: bool,
}

impl<'a> SuffixTree<'a> {
    pub fn new() -> SuffixTree<'a> {
        SuffixTree {
            sequences: Vec::new(),
            nodes: vec![Node::new_root()],

            prepared_lcs: false,
        }
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

    fn prepare_lcs(&mut self) {
        assert!(!self.prepared_lcs);

        fn _prepare_lcs<'a, 'b>(tree: &'a mut SuffixTree<'b>, node: NodeId) -> Cow<'a, BitVec> {
            match &tree.nodes[node] {
                &Node::Internal(InternalNode { ref children, .. }) => {
                    Some(children.values().cloned().collect())
                },
                _ => None,
            }.map(|children: Vec<usize>| {
                let mut id_set = BitVec::from_elem(tree.sequences.len(), false);
                for child in children {
                    id_set.union(&_prepare_lcs(tree, child));
                }

                id_set
            }).into_iter().for_each(|id_set| {
                match &mut tree.nodes[node] {
                    &mut Node::Internal(InternalNode { ref mut sequence_id_set, .. }) => {
                        *sequence_id_set = Some(id_set);
                    },
                    _ => unreachable!(),
                };
            });

            match &tree.nodes[node] {
                &Node::Internal(InternalNode{ sequence_id_set: Some(ref id_set), .. }) => {
                    Cow::Borrowed(id_set)
                },
                &Node::Leaf(LeafNode { seq_id, .. }) => {
                    let mut id_set = BitVec::from_elem(tree.sequences.len(), false);
                    id_set.set(seq_id, true);
                    Cow::Owned(id_set)
                },
                _ => unreachable!(),
            }
        }

        let children: Vec<_> = match &self.nodes[0] {
            Node::Root(RootNode { ref children }) => children.values().cloned().collect(),
            _ => unreachable!(),
        };

        for child in children {
            _prepare_lcs(self, child);
        }

        self.prepared_lcs = true;
    }

    pub fn longest_common_subsequence(&mut self) -> Option<(SequenceId, usize, usize)> {
        if !self.prepared_lcs {
            self.prepare_lcs();
        }

        fn _lcs<'a>(tree: &SuffixTree<'a>, node: NodeId)
            -> Option<(SequenceId, usize)>
        {
            match &tree.nodes[node] {
                &Node::Internal(InternalNode {
                    seq_id,
                    end,
                    sequence_id_set: Some(ref id_set),
                    ref children, 
                    ..
                }) => {
                    if id_set.all() {
                        children.values().filter_map(|&child| {
                            _lcs(tree, child)
                        }).max_by_key(|&(_, end)| {
                            end
                        }).or_else(|| {
                            Some((seq_id, end))
                        })
                    } else {
                        None
                    }
                },
                &Node::Leaf(_) => None,
                _ => unreachable!(),
            }
        }

        match self.nodes[0] {
            Node::Root(RootNode { ref children, .. }) => {
                children.values().filter_map(|&child| {
                    let start = match &self.nodes[child] {
                        &Node::Internal(InternalNode { start, .. })
                        | &Node::Leaf(LeafNode { start, .. }) => start,
                        Node::Root(_) => unreachable!(),
                    };

                    _lcs(self, child).map(|(seq_id, end)| (seq_id, start, end))
                }).max_by_key(|(_, start, end)| end - start)
            },
            _ => unreachable!(),
        }
    }

    fn pretty_print_parent(&self, children: &HashMap<Symbol, NodeId>, text: String) -> Vec<String> {
        let mut lines = Vec::new();
        for (i, &child) in children.values().enumerate() {
            for (j, line) in self.pretty_print_node(child).into_iter().enumerate() {
                lines.push(
                    if i == 0 && j == 0 {
                        format!("{}┳{}", text, line)
                    } else if i < children.len() - 1 && j == 0 {
                        format!("{}┣{}", " ".repeat(text.len()), line)
                    } else if j == 0 {
                        format!("{}┗{}", " ".repeat(text.len()), line)
                    } else if i < children.len() - 1 {
                        format!("{}┃{}", " ".repeat(text.len()), line)
                    } else {
                        format!("{} {}", " ".repeat(text.len()), line)
                    }
                );
            }
        }

        lines
    }

    fn pretty_print_node(&self, node: NodeId) -> Vec<String> {
        match &self.nodes[node] {
            &Node::Root(RootNode { ref children }) => {
                self.pretty_print_parent(children, "".to_owned())
            },
            &Node::Internal(InternalNode { seq_id, start, end, ref children, .. }) => {
                let text = self.sequences[seq_id].string_repr_internal(start,end);
                self.pretty_print_parent(
                    children,
                    format!("({}){}", node, text) 
                )
            },
            &Node::Leaf(LeafNode { seq_id, start, .. }) => {
                vec![self.sequences[seq_id].string_repr_leaf(start)]
            },
        }
    }

    pub fn pretty_print(&self) {
        for line in self.pretty_print_node(0) {
            println!("{}", line);
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

    pub fn build(self) -> SuffixTree<'a> {
        self.tree
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

    fn insert_leaf_node(&mut self, next_symbol: Symbol) -> bool{
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
            &Node::Root(_) => unreachable!(),
        };
        let split_position = active_start + active_length;

        let insert_node = self.tree.sequences[active_seq_id].at(split_position) != next_symbol;

        if insert_node {
            match &mut self.tree.nodes[active_edge_node] {
                &mut Node::Internal(InternalNode { ref mut start, .. }) |
                &mut Node::Leaf(LeafNode { ref mut start, .. }) => {
                    *start = split_position;
                },
                &mut Node::Root(_) => unreachable!(),
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
                self.active_edge = self.active_edge.map(|(_, length)| ( 
                    self.tree.current_sequence().at(self.position + 2 - self.remaining),
                    length - 1
                ));
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

    fn normalize_active_point(&mut self) {
        loop {
            match self.active_edge {
                Some((_, 0)) => {
                    self.active_edge = None;
                    break;
                },
                Some((_, active_length)) => {
                    let edge_length = match &self.tree.nodes[self.active_edge_node()] {
                        &Node::Root(_) => unreachable!(),
                        &Node::Internal(InternalNode { start, end, .. }) => end - start,
                        &Node::Leaf(LeafNode { seq_id, start, .. }) => {
                            let offset = (seq_id == self.tree.current_sequence().id) as usize;
                            (self.tree.sequences[seq_id].len() + offset) - start
                        },
                    };

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
            match &mut self.tree.nodes[node] {
                &mut Node::Internal(InternalNode { ref mut suffix_link, .. }) => *suffix_link = Some(link_to),
                _ => unreachable!(),
            }
        }

        self.previously_created_node = None;
    }

    fn active_edge_node(&self) -> NodeId {
        match self.active_edge {
            Some((symbol, _)) => {
                self.tree.nodes[self.active_node].get_child(symbol).unwrap()
            },
            None => unreachable!(),
        }
        
    }

    #[allow(dead_code)]
    fn print_info(&self) {
        println!("active_node is {}, active_edge is {:?}", self.active_node, self.active_edge);
        println!("position is {}, remaining is {}", self.position, self.remaining);
    }
}
