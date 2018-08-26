use std::collections::HashMap;
use std::str;
use std::u8;

type NodeId = usize;

struct RootNode {
    children: HashMap<u8, NodeId>,
}

struct InternalNode {
    string_id: usize,
    start: usize,
    end: usize,
    children: HashMap<u8, NodeId>,
    suffix_link: Option<NodeId>,
}

struct LeafNode {
    string_id: usize,
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

    fn new_internal(string_id: usize, start: usize, end: usize) -> Node {
        Node::Internal(InternalNode {
            string_id,
            start,
            end,
            children: HashMap::new(),
            suffix_link: None,
        })
    }

    fn new_leaf(string_id: usize, start: usize) -> Node {
        Node::Leaf(LeafNode { string_id, start })
    }

    fn add_child(&mut self, c: u8, child: NodeId) {
        match self {
            &mut Node::Internal(InternalNode { ref mut children, .. }) |
            &mut Node::Root(RootNode { ref mut children, .. }) => children.insert(c, child),
            &mut Node::Leaf(_) => unreachable!(),
        };
    }

    fn get_child(&self, c: u8) -> Option<NodeId> {
        match self {
            &Node::Root(RootNode { ref children, .. }) |
            &Node::Internal(InternalNode { ref children, .. }) => children.get(&c).map(|&v| v),
            &Node::Leaf(_) => unreachable!(),
        }
    }
}

pub struct SuffixTree<'a> {
    strings: Vec<&'a [u8]>,

    nodes: Vec<Node>, 
    
    active_node: NodeId,
    active_edge: u8,
    active_length: usize,

    position: usize,
    remaining: usize,

    previously_created_node: Option<NodeId>,
}

impl<'a> SuffixTree<'a> {
    pub fn new() -> SuffixTree<'a> {
        SuffixTree {
            strings: Vec::new(),

            nodes: vec![Node::new_root()],

            active_node: 0,
            active_edge: 0,
            active_length: 0,

            remaining: 0,

            position: 0,

            previously_created_node: None,
        }
    }

    pub fn build_from_string(string: &'a [u8]) -> SuffixTree<'a> {
        let mut tree = SuffixTree::new();
        tree.add_string(string);
        tree
    }

    pub fn build_from_strings(strings: Vec<&'a [u8]>) -> SuffixTree<'a> {
        let mut tree = SuffixTree::new();
        for string in strings {
            tree.add_string(string);
        }
        tree
    }

    fn current_string(&self) -> &[u8]  {
        self.strings[self.current_string_id()]
    }

    fn current_string_id(&self) -> usize {
        self.strings.len() - 1
    }

    pub fn add_string(&mut self, string: &'a [u8]) {
        self.strings.push(string);

        self.active_node = 0;
        self.active_edge = 0;
        self.active_length = 0;

        for i in 0..string.len() {
            let next_char = self.current_string()[i];
            if self.active_length == 0
                && self.nodes[self.active_node].get_child(next_char).is_some()
            {
                self.active_edge = next_char;
                self.active_length = 1;
            } else if self.active_length != 0
                && self.substring(self.active_edge_node())[self.active_length] == next_char
            {
                self.active_length += 1;
            } else {
                self.remaining = i;
                self.position = i;
                break;
            }

            let active_edge_length = match &self.nodes[self.active_edge_node()] {
                &Node::Internal(InternalNode { start, end, .. }) => end - start,
                &Node::Leaf(LeafNode { string_id, start, .. }) => self.strings[string_id].len() - start,
                &Node::Root(_) => unreachable!(),
            };

            if self.active_length == active_edge_length {
                self.active_node = self.active_edge_node();
                self.active_edge = 0;
                self.active_length = 0;
            }
        }

        for _ in self.position..string.len() {
            self.step();
        }
    }

    fn step(&mut self) {
        self.remaining += 1;
        self.previously_created_node = None;

        let next_char = self.current_string()[self.position];
        for _ in 0..self.remaining {
            if self.active_length == 0 {
                if self.nodes[self.active_node].get_child(next_char).is_none() {
                    self.insert_leaf_node();

                    if self.active_node != 0 {
                        let active_node = self.active_node;
                        self.set_suffix_link(active_node);

                        self.update_active_point();
                    }

                    self.remaining -= 1;
                } else {
                    self.active_edge = next_char;
                    self.active_length = 1;
                    self.normalize_active_point();
                    break;
                }
            } else {
                if self.substring(self.active_edge_node())[self.active_length] != next_char {
                    let new_node = self.insert_internal_node();

                    self.set_suffix_link(new_node);
                    self.previously_created_node = Some(new_node);

                    if self.active_node == 0 {
                        self.active_edge = self.current_string()[(self.position + 2) - self.remaining];
                        self.active_length -= 1;
                        self.normalize_active_point();
                    } else {
                        self.update_active_point();
                    }

                    self.remaining -= 1;
                } else {
                    self.active_length += 1;
                    self.normalize_active_point();
                    break;
                }
            }
        }

        self.position += 1;
    }

    fn insert_leaf_node(&mut self) {
        let leaf_node = Node::new_leaf(self.current_string_id(), self.position);
        let leaf = self.add_node(leaf_node);

        let next_char = self.current_string()[self.position];
        self.nodes[self.active_node].add_child(next_char, leaf);
    }

    fn insert_internal_node(&mut self) -> NodeId {
        let current_string_id = self.current_string_id();
        let position = self.position;
        let active_length = self.active_length;

        let existing_node = self.active_edge_node();
        let (existing_string_id, existing_start) = match &mut self.nodes[existing_node] {
            &mut Node::Internal(InternalNode { string_id, ref mut start, .. }) |
            &mut Node::Leaf(LeafNode { string_id, ref mut start, .. }) => {
                let existing_start = *start;
                *start += active_length;

                (string_id, existing_start)
            },
            &mut Node::Root(_) => unreachable!(),
        };

        let split_position = existing_start + self.active_length;
        let node_a = self.add_node(Node::new_internal(
            existing_string_id,
            existing_start,
            split_position
        ));

        self.nodes[self.active_node].add_child(self.active_edge, node_a);

        let a_to_active_edge = self.strings[existing_string_id][split_position];
        self.nodes[node_a].add_child(a_to_active_edge, existing_node);

        let node_b = self.add_node(Node::new_leaf(current_string_id, position));
        let a_to_b = self.current_string()[self.position];
        self.nodes[node_a].add_child(a_to_b, node_b);

        node_a
    }

    fn update_active_point(&mut self) {
        match &self.nodes[self.active_node] {
            &Node::Internal(InternalNode { suffix_link: Some(node), .. }) => {
                self.active_node = node;
            },
            &Node::Internal(_) | &Node::Leaf(_) => {
                self.active_node = 0;
                let active_edge = self.current_string()[(self.position + 2) - self.remaining];
                self.active_edge = active_edge;
                self.active_length = self.remaining - 2;
            }
            &Node::Root(_) => unreachable!(),
        }

        self.normalize_active_point();
    }

    fn normalize_active_point(&mut self) {
        loop {
            if self.active_length == 0 {
                break;
            } else {
                let active_edge_length = match &self.nodes[self.active_edge_node()] {
                    &Node::Root(_) => unreachable!(),
                    &Node::Internal(InternalNode { start, end, .. }) => end - start,
                    &Node::Leaf(LeafNode { string_id, start, .. }) => {
                        let string_len = self.strings[string_id].len();
                        let offset = if string_id == self.current_string_id() { 1 } else { 0 };

                        (string_len + offset) - start
                    },
                };

                if self.active_length < active_edge_length {
                    break;
                } else if self.active_length == active_edge_length {
                    self.active_node = self.active_edge_node();
                    self.active_edge = 0;
                    self.active_length = 0;
                    break;
                } else {
                    self.active_node = self.active_edge_node();
                    self.active_edge = self.current_string()[self.position - self.active_length + active_edge_length];
                    self.active_length -= active_edge_length;
                }
            }
        }
    }

    fn set_suffix_link(&mut self, link_to: NodeId) {
        if let Some(node) = self.previously_created_node {
            match &mut self.nodes[node] {
                &mut Node::Internal(InternalNode { ref mut suffix_link, .. }) => *suffix_link = Some(link_to),
                _ => unreachable!(),
            }
        }

        self.previously_created_node = None;
    }

    fn active_edge_node(&self) -> NodeId {
        self.nodes[self.active_node].get_child(self.active_edge).unwrap()
    }

    fn substring(&self, node: NodeId) -> &[u8] {
        match &self.nodes[node] {
            &Node::Internal(InternalNode { string_id, start, end, .. }) => {
                &self.strings[string_id][start..end]
            },
            &Node::Leaf(LeafNode { string_id, start, .. }) => {
                &self.strings[string_id][start..]
            },
            &Node::Root(_) => unreachable!(),
        }
    }

    fn add_node(&mut self, node: Node) -> NodeId {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    fn _visualize_non_leaf(&self, children: &HashMap<u8, NodeId>, text: &str) -> Vec<String> {
        let mut lines = Vec::new();
        for (i, &child) in children.values().enumerate() {
            for (j, line) in self._visualize(child).into_iter().enumerate() {
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

    fn _visualize(&self, node: NodeId) -> Vec<String> {
        match &self.nodes[node] {
            &Node::Root(RootNode { ref children }) => {
                self._visualize_non_leaf(children, "")
            },
            &Node::Internal(InternalNode { string_id, start, end, ref children, .. }) => {
                let text = str::from_utf8(&self.strings[string_id][start..end])
                    .unwrap_or("<invalid_string>");
                self._visualize_non_leaf(children, text)
            },
            &Node::Leaf(LeafNode { string_id, start, .. }) => {
                let end = if string_id == self.current_string_id() {
                    self.position
                } else {
                    self.strings[string_id].len()
                };
                let text = str::from_utf8(&self.strings[string_id][start..end])
                    .unwrap_or("<invalid_string>");
                vec![format!("({}){}", node, text)]
            },
        }
    }

    pub fn visualize(&self) {
        for line in self._visualize(0) {
            println!("{}", line);
        }
    }

    fn print_info(&self) {
        println!("active node is {}, active length is {}, active edge is {}", self.active_node, self.active_length, self.active_edge as char);
        println!("remaining is {}, position is {}", self.remaining, self.position);
    }
}
