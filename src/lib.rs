use std::str;
use std::u8;

type NodeId = usize;

struct Node {
    start: usize,
    end: Option<usize>,
    children: Option<[Option<NodeId>; u8::MAX as usize]>,
    suffix_link: Option<NodeId>,
}

impl Node {
    fn new_internal(start: usize, end: usize) -> Node {
        Node {
            start,
            end: Some(end),
            children: Some([None; u8::MAX as usize]),
            suffix_link: None,
        }
    }

    fn new_leaf(start: usize) -> Node {
        Node {
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
    bytes: &'a [u8],

    nodes: Vec<Node>, 
    
    active_node: NodeId,
    active_edge: u8,
    active_length: usize,

    position: usize,
    remaining: usize,

    previously_created_node: Option<NodeId>,
}

impl<'a> SuffixTree<'a> {
    pub fn init_with_bytes(bytes: &'a [u8]) -> SuffixTree<'a> {
        SuffixTree {
            bytes,

            nodes: vec![Node::new_internal(0, 0)],

            active_node: 0,
            active_edge: 0,
            active_length: 0,

            remaining: 0,

            position: 0,

            previously_created_node: None,
        }
    }

    pub fn build_from_bytes(bytes: &'a [u8]) -> SuffixTree<'a> {
        let mut tree = SuffixTree::init_with_bytes(bytes);
        tree.build();
        tree
    }

    pub fn init_with_str(text: &'a str) -> SuffixTree<'a> {
        SuffixTree::init_with_bytes(text.as_bytes())
    }

    pub fn build_from_str(text: &'a str) -> SuffixTree<'a> {
        SuffixTree::build_from_bytes(text.as_bytes())
    }


    pub fn build(&mut self) {
        for _ in 0..self.bytes.len() {
            self.step();
        }
    }

    pub fn step(&mut self) {
        self.remaining += 1;
        self.previously_created_node = None;

        let next_char = self.bytes[self.position];
        for _ in 0..self.remaining {
            if self.active_length == 0 {
                if !self.nodes[self.active_node].children.unwrap()[next_char as usize].is_some() {
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
                        self.active_edge = self.bytes[self.position - self.remaining + 2];
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
        let position = self.position;
        let leaf = self.add_node(Node::new_leaf(position));
        let next_char = self.bytes[position];
        self.nodes[self.active_node].add_child(next_char, leaf);
    }

    fn insert_internal_node(&mut self) -> NodeId {
        let start_of_existing = self.nodes[self.active_edge_node()].start;
        
        let position = self.position;
        let active_length = self.active_length;

        let node_a = self.add_node(Node::new_internal(start_of_existing, start_of_existing + active_length));
        let node_b = self.add_node(Node::new_leaf(position));

        let active_edge_node = self.active_edge_node();

        self.nodes[active_edge_node].start += active_length;

        self.nodes[self.active_node].add_child(self.active_edge, node_a);

        self.nodes[node_a].add_child(self.bytes[self.position], node_b);
        self.nodes[node_a].add_child(self.bytes[start_of_existing + self.active_length], active_edge_node);

        node_a
    }

    fn update_active_point(&mut self) {
        if let Some(node) = self.nodes[self.active_node].suffix_link {
            self.active_node = node;
        } else {
            self.active_node = 0;
            self.active_edge = self.bytes[self.position - self.remaining + 2];
            self.active_length = self.remaining - 2;
        }

        self.normalize_active_point();
    }

    fn normalize_active_point(&mut self) {
        loop {
            if self.active_length == 0 {
                break;
            } else {
                let active_edge_length = {
                    let node = &self.nodes[self.active_edge_node()];
                    node.end.unwrap_or(self.position + 1) - node.start
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
                    self.active_edge = self.bytes[self.position - self.active_length + active_edge_length];
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
        let node = &self.nodes[node];
        &self.bytes[node.start..node.end.unwrap_or(self.position)]
    }

    fn add_node(&mut self, node: Node) -> NodeId {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    fn _visualize(&self, node: NodeId) -> Vec<String> {
        match &self.nodes[node] {
            &Node { start, end: Some(end), children: Some(ref children), .. } => {
                let edge_label = str::from_utf8(&self.bytes[start..end]).unwrap_or("<invalid_string>");
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
            &Node { start, .. } => {
                let edge_label = str::from_utf8(&self.bytes[start..self.position]).unwrap_or("<invalid_string>");
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
}
