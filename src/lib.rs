use std::collections::HashMap;
use std::str;

type NodeId = usize;

#[derive(Debug)]
struct InternalNode {
    start: usize,
    end: usize,
    edges: HashMap<u8, NodeId>,

    suffix_link: Option<NodeId>,
}

#[derive(Debug)]
enum Node {
    Internal(InternalNode),
    Leaf(usize),
}

impl Node {
    fn new_internal(start: usize, end: usize) -> Node {
        Node::Internal(InternalNode {
            start,
            end,
            edges: HashMap::new(),

            suffix_link: None,
        })
    }

    fn new_leaf(start: usize) -> Node {
        Node::Leaf(start)
    }

    fn internal(&self) -> &InternalNode {
        if let Node::Internal(ref internal) = self {
            internal
        } else {
            panic!("Expected this node to be an internal node.")
        }
    }

    fn mut_internal(&mut self) -> &mut InternalNode {
        if let Node::Internal(ref mut internal) = self {
            internal
        } else {
            panic!("Expected this node to be an internal node.")
        }
    }
}

#[derive(Debug)]
pub struct SuffixTree<'a> {
    text: &'a [u8],

    nodes: Vec<Node>, 
    
    active_node: NodeId,
    active_edge: u8,
    active_length: usize,

    remaining: usize,
    position: usize,
    previously_created_node: Option<NodeId>,
}

impl<'a> SuffixTree<'a> {
    pub fn new(text: &'a [u8]) -> SuffixTree<'a> {
        SuffixTree {
            text,

            nodes: vec![Node::new_internal(0, 0)],

            active_node: 0,
            active_edge: 0,
            active_length: 0,

            remaining: 0,

            position: 0,

            previously_created_node: None,
        }
    }

    fn active_edge_node(&self) -> NodeId {
        *self.nodes[self.active_node].internal().edges.get(&self.active_edge).unwrap()
    }

    fn substring(&self, node: NodeId) -> &[u8] {
        match &self.nodes[node] {
            &Node::Internal(InternalNode { start, end, .. }) => &self.text[start..end],
            &Node::Leaf(start) => &self.text[start..self.position],
        }
    }

    fn add_node(&mut self, node: Node) -> NodeId {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    pub fn step(&mut self) {
        self.remaining += 1;
        self.previously_created_node = None;

        let next_char = self.text[self.position];
        for _ in 0..self.remaining {
            if self.active_length == 0 {
                if !self.nodes[self.active_node].internal().edges.contains_key(&next_char) {
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
                        self.active_edge = self.text[self.position - self.remaining + 2];
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
        let next_char = self.text[position];
        self.nodes[self.active_node].mut_internal().edges.insert(next_char, leaf);
    }

    fn insert_internal_node(&mut self) -> NodeId {
        let start_of_existing = match &self.nodes[self.active_edge_node()]  {
            &Node::Internal(InternalNode { start, .. }) => start,
            &Node::Leaf(start) => start,
        };
        
        let position = self.position;
        let active_length = self.active_length;

        let node_a = self.add_node(Node::new_internal(start_of_existing, start_of_existing + active_length));
        let node_b = self.add_node(Node::new_leaf(position));

        let active_edge_node = self.active_edge_node();
        match &mut self.nodes[active_edge_node] {
            Node::Internal(InternalNode { ref mut start, .. }) => *start += active_length,
            Node::Leaf(ref mut start) => *start += active_length,
        };


        let active_to_a = self.active_edge;
        self.nodes[self.active_node].mut_internal().edges.insert(active_to_a, node_a);

        let a_to_b = self.text[self.position];
        &mut self.nodes[node_a].mut_internal().edges.insert(a_to_b, node_b);

        let a_to_active_edge = self.text[start_of_existing + self.active_length];
        &mut self.nodes[node_a].mut_internal().edges.insert(a_to_active_edge, active_edge_node);

        node_a
    }

    fn update_active_point(&mut self) {
        if let Some(node) = self.nodes[self.active_node].internal().suffix_link {
            self.active_node = node;
        } else {
            self.active_node = 0;
            self.active_edge = self.text[self.position - self.remaining + 2];
            self.active_length = self.remaining - 2;
        }

        self.normalize_active_point();
    }

    fn normalize_active_point(&mut self) {
        loop {
            if self.active_length == 0 {
                break;
            } else {
                let active_edge_length = match &self.nodes[self.active_edge_node()]  {
                    &Node::Internal(InternalNode { start, end, .. }) => end - start, 
                    &Node::Leaf(start) => self.position - start + 1,
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
                    self.active_edge = self.text[self.position - self.active_length + active_edge_length];
                    self.active_length -= active_edge_length;
                }
            }
        }
    }

    fn set_suffix_link(&mut self, link_to: NodeId) {
        if let Some(node) = self.previously_created_node {
            self.nodes[node].mut_internal().suffix_link = Some(link_to);
        }

        self.previously_created_node = None;
    }

    fn _visualize(&self, node: NodeId) -> Vec<String> {
        match &self.nodes[node] {
            &Node::Internal(InternalNode { start, end, ref edges, .. }) => {
                let edge_label = str::from_utf8(&self.text[start..end]).unwrap_or("<invalid_string>");
                let text = format!("({}){}", node, edge_label);
                let mut lines = Vec::new();
                for (i, &child) in edges.values().enumerate() {
                    for (j, line) in self._visualize(child).into_iter().enumerate() {
                        let prefix = if i == 0 && j == 0 {
                            text.to_owned()
                        } else {
                            " ".repeat(text.len()) 
                        };

                        let line = if i == 0 && j == 0 {
                            format!("{}┳{}", prefix, line)
                        } else if i < edges.len() - 1 && j == 0 {
                            format!("{}┣{}", prefix, line)
                        } else if j == 0 {
                            format!("{}┗{}", prefix, line)
                        } else if i < edges.len() - 1 {
                            format!("{}┃{}", prefix, line)
                        } else {
                            format!("{} {}", prefix, line)
                        };

                        lines.push(line);
                    }
                }

                lines
            },
            &Node::Leaf(start) => {
                let edge_label = str::from_utf8(&self.text[start..self.position]).unwrap_or("<invalid_string>");
                let text = format!("({}){}", node, edge_label);
                vec![text.to_owned()]
            }
        }
    }

    pub fn visualize(&self) {
        for line in self._visualize(0) {
            println!("{}", line);
        }
    }
}
