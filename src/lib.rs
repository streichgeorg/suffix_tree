use std::str;
use std::u8;

type NodeId = usize;

struct Node {
    string_id: usize,
    start: usize,
    end: Option<usize>,
    children: Option<[Option<NodeId>; u8::MAX as usize]>,
    suffix_link: Option<NodeId>,
}

impl Node {
    fn new_internal(string_id: usize, start: usize, end: usize) -> Node {
        Node {
            string_id,
            start,
            end: Some(end),
            children: Some([None; u8::MAX as usize]),
            suffix_link: None,
        }
    }

    fn new_leaf(string_id: usize, start: usize) -> Node {
        Node {
            string_id,
            start,
            end: None,
            children: None,
            suffix_link: None,
        }
    }

    fn add_child(&mut self, c: u8, child: NodeId) {
        self.children.as_mut().unwrap()[c as usize] = Some(child);
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

            nodes: vec![Node::new_internal(0, 0, 0)],

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
                && self.nodes[self.active_node].children.unwrap()[next_char as usize].is_some()
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
                &Node { start, end: Some(end), .. } => end - start,
                &Node { string_id, start, .. } => self.strings[string_id].len() - start
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
                if self.nodes[self.active_node].children.unwrap()[next_char as usize].is_none() {
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

        let &Node {
            string_id: existing_string_id,
            start: existing_start,
            ..
        } = &self.nodes[self.active_edge_node()];

        let active_edge_node = self.active_edge_node();
        self.nodes[active_edge_node].start += self.active_length;

        let split_position = existing_start + self.active_length;
        let node_a = self.add_node(Node::new_internal(
            existing_string_id,
            existing_start,
            split_position
        ));

        self.nodes[self.active_node].add_child(self.active_edge, node_a);

        let a_to_active_edge = self.strings[existing_string_id][split_position];
        self.nodes[node_a].add_child(a_to_active_edge, active_edge_node);

        let node_b = self.add_node(Node::new_leaf(current_string_id, position));
        let a_to_b = self.current_string()[self.position];
        self.nodes[node_a].add_child(a_to_b, node_b);

        node_a
    }

    fn update_active_point(&mut self) {
        if let Some(node) = self.nodes[self.active_node].suffix_link {
            self.active_node = node;
        } else {
            self.active_node = 0;
            let active_edge = self.current_string()[(self.position + 2) - self.remaining];
            self.active_edge = active_edge;
            self.active_length = self.remaining - 2;
        }

        self.normalize_active_point();
    }

    fn normalize_active_point(&mut self) {
        loop {
            if self.active_length == 0 {
                break;
            } else {
                let active_edge_length = match &self.nodes[self.active_edge_node()] {
                    &Node { start, end: Some(end), .. } => end - start,
                    &Node { string_id, start, end: None, .. } => {
                        let end = self.strings[string_id].len() + if string_id == self.current_string_id() {
                            1
                        } else {
                            0
                        };
                        end - start
                    }
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
            self.nodes[node].suffix_link = Some(link_to);
        }

        self.previously_created_node = None;
    }

    fn active_edge_node(&self) -> NodeId {
        self.nodes[self.active_node].children.unwrap()[self.active_edge as usize].unwrap()
    }

    fn substring(&self, node: NodeId) -> &[u8] {
        match &self.nodes[node] {
            &Node { string_id, start, end: Some(end), .. } => &self.strings[string_id][start..end],
            &Node { string_id, start, end: None, .. } => &self.strings[string_id][start..],
        }
    }

    fn add_node(&mut self, node: Node) -> NodeId {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    fn _visualize(&self, node: NodeId) -> Vec<String> {
        match &self.nodes[node] {
            &Node { string_id, start, end: Some(end), children: Some(ref children), .. } => {
                let edge_label = str::from_utf8(&self.strings[string_id][start..end]).unwrap_or("<invalid_string>");
                let text = format!("({}){}", node, edge_label);

                let children: Vec<(usize, NodeId)> = children.iter().filter_map(|&e| e).enumerate().collect();
                let mut lines = Vec::new();
                for &(i, child) in &children {
                    for (j, line) in self._visualize(child).into_iter().enumerate() {
                        let prefix = if i == 0 && j == 0 {
                            text.to_owned()
                        } else {
                            " ".repeat(text.len()) 
                        };

                        let line = if i == 0 && j == 0 {
                            format!("{}┳{}", prefix, line)
                        } else if i < children.len() - 1 && j == 0 {
                            format!("{}┣{}", prefix, line)
                        } else if j == 0 {
                            format!("{}┗{}", prefix, line)
                        } else if i < children.len() - 1 {
                            format!("{}┃{}", prefix, line)
                        } else {
                            format!("{} {}", prefix, line)
                        };

                        lines.push(line);
                    }
                }

                lines
            },
            &Node { string_id, start, .. } => {
                let end = if string_id == self.current_string_id() {
                    self.position
                } else {
                    self.strings[string_id].len()
                };
                let edge_label = str::from_utf8(&self.strings[string_id][start..end]).unwrap_or("<invalid_string>");
                let text = format!("({}){}", node, edge_label);
                vec![text.to_owned()]
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
