use std::collections::HashMap;
use std::fmt;
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

pub struct SuffixTree<'a> {
    text: &'a [u8],

    nodes: Vec<Node>, 
    
    active_node: NodeId,
    active_edge: u8,
    active_length: usize,

    remaining: usize,

    position: usize,
}

impl<'a> fmt::Debug for SuffixTree<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "SuffixTree{{"));
        try!(writeln!(f, "    active_node: {}", self.active_node));
        try!(writeln!(f, "    active_edge: {}", self.active_edge as char));
        try!(writeln!(f, "    active_length: {}\n", self.active_length));

        try!(writeln!(f, "    remaining: {}", self.remaining));
        try!(writeln!(f, "    step: {}\n", self.position));

        try!(writeln!(f, "    nodes: ["));
        for (i, node) in self.nodes.iter().enumerate() {
            match node {
                &Node::Internal(ref internal) => {
                    let text = str::from_utf8(&self.text[(internal.start)..(internal.end)])
                                .unwrap_or("<invalid_string>");

                    let edges: HashMap<_, _> = internal.edges.iter().map(|(k, v)| (*k as char, *v)).collect();

                    try!(writeln!(f, "       InternalNode: {{"));
                    try!(writeln!(f, "          id: {}", i));
                    try!(writeln!(f, "          text: {}", text));
                    try!(writeln!(f, "          children: {:?}", edges));
                    try!(writeln!(f, "          suffix_link: {:?}", internal.suffix_link));
                    try!(writeln!(f, "       }}"));
                },
                &Node::Leaf(start) => {
                    let text = str::from_utf8(&self.text[(start)..self.position])
                                .unwrap_or("<invalid_string>");

                    try!(writeln!(f, "       LeafNode: {{"));
                    try!(writeln!(f, "          id: {}", i));
                    try!(writeln!(f, "          text: {}", text));
                    try!(writeln!(f, "       }}"));
                }
            }
        }
        try!(writeln!(f, "    ]"));

        writeln!(f, "}}")
    }
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
        }
    }

    fn get_node(&self, node: NodeId) -> &Node {
        &self.nodes[node]
    }

    fn get_mut_node(&mut self, node: NodeId) -> &mut Node {
        &mut self.nodes[node]
    }

    fn get_active_node(&self) -> &InternalNode {
        self.nodes[self.active_node].internal()
    }

    fn get_mut_active_node(&mut self) -> &mut InternalNode {
        self.nodes[self.active_node].mut_internal()
    }

    fn get_active_edge(&self) -> NodeId {
        *self.get_active_node().edges.get(&self.active_edge).unwrap()
    }

    fn get_substring(&self, node: NodeId) -> &[u8] {
        match self.get_node(node) {
            &Node::Internal(InternalNode { start, end, .. }) => &self.text[start..end],
            &Node::Leaf(start) => &self.text[start..self.position],
        }
    }
    
    fn get_substring_length(&self, node: NodeId) -> usize {
        match self.get_node(node) {
            &Node::Internal(InternalNode { start, end, .. }) => (end - start),
            &Node::Leaf(start) => self.position - start,
        }
    }

    fn insert_leaf_node(&mut self, previously_created_node: &mut Option<NodeId>) -> bool {
        // Check if the active nod has an edge that starts with the current character. If so we
        // dont need to to anything in this extension.
        if self.get_active_node().edges.contains_key(&self.text[self.position]) {
            return false;
        }

        // Add a leaf node to the active node.
        let leaf = self.nodes.len();
        self.nodes.push(Node::new_leaf(self.position));

        // Add a leaf node to the active node.
        let c = self.text[self.position];
        self.get_mut_active_node().edges.insert(c, leaf);

        if self.active_node != 0 && previously_created_node.is_some() {
            self.get_mut_node(previously_created_node.unwrap()).mut_internal().suffix_link = Some(self.active_node);
            *previously_created_node = Some(self.active_node);
        }

        if self.active_node != 0 {
            self.active_node = self.get_active_node().suffix_link.unwrap_or(0);
        }

        true
    }

    fn walk_down(&mut self, offset: usize) {
        let mut num_skipped = 0;
        loop {
            let label_length = self.get_substring_length(self.get_active_edge());

            if self.active_length - num_skipped < label_length {
                self.active_length -= num_skipped;
                
                break;
            } else if self.active_length - num_skipped == label_length {
                self.active_node = self.get_active_edge();
                self.active_edge = 0;
                self.active_length = 0;

                break;
            } else {
                num_skipped += label_length;

                self.active_node = self.get_active_edge();
                self.active_edge = self.text[offset + num_skipped];
            }
        }
    }

    fn insert_internal_node(&mut self, previously_created_node: &mut Option<NodeId>) -> bool {
        // Check if the next character from the active point is equal to the one we want to add. If
        // so we don't need to do anything in this extension.
        if self.get_substring(self.get_active_edge())[self.active_length] == self.text[self.position] {
            return false;
        }

        // Insert a new internal node in between the active node and the corresponding child node.
        // Add a leaf node to the new internal node.
        let label_start = match self.get_node(self.get_active_edge()) {
            &Node::Internal(InternalNode { start, .. }) => start,
            &Node::Leaf(start) => start
        };

        let internal = self.nodes.len();
        self.nodes.push(Node::new_internal(label_start, label_start + self.active_length));

        let leaf = self.nodes.len();
        self.nodes.push(Node::new_leaf(self.position));
        
        let existing_edge = self.get_active_edge();

        let length = self.active_length;
        match self.get_mut_node(existing_edge) {
            Node::Internal(InternalNode { ref mut start, .. }) => *start += length,
            Node::Leaf(ref mut start) => *start += length,
        };

        let active_to_internal = self.active_edge;
        self.get_mut_active_node().edges.insert(active_to_internal, internal);

        let internal_to_existing = self.text[label_start + self.active_length];
        let internal_to_leaf = self.text[self.position];
        self.get_mut_node(internal).mut_internal().edges.insert(internal_to_existing, existing_edge);
        self.get_mut_node(internal).mut_internal().edges.insert(internal_to_leaf, leaf);

        // if there is a previously created internal node, make suffix link from it to the node
        // created in this extension.
        if let &mut Some(node) = previously_created_node {
            self.get_mut_node(node).mut_internal().suffix_link = Some(internal);
        }
        *previously_created_node = Some(internal);

        if self.active_node == 0 {
            self.active_edge = self.text[self.position - self.remaining + 2];
            self.active_length -= 1;

            if self.active_length > 0 {
                self.walk_down(label_start + 1);
            }
        } else if self.active_node != 0 {
            self.active_node = self.get_active_node().suffix_link.unwrap_or(0);

            self.walk_down(label_start);
        }

        true
    }

    pub fn step(&mut self) {
        println!("step {}, inserting '{}', remainder {}", self.position, self.text[self.position] as char, self.remaining);

        self.remaining += 1;

        let mut previously_created_node = None;
        for _ in 0..self.remaining {
            let inserted_node = if self.active_length == 0 {
                self.insert_leaf_node(&mut previously_created_node)
            } else {
                self.insert_internal_node(&mut previously_created_node)
            };

            if inserted_node {
                println!("Inserted a node");
                self.remaining -= 1;
            } else {
                println!("Did not insert a node");
                if self.active_length != 0 || self.remaining != 1 {
                    self.active_edge = self.text[self.position - self.remaining + 2];
                    self.active_length = self.remaining - 2;

                    println!("active point is ({}, '{}', {})", self.active_node, self.active_edge as char, self.active_length);

                    let length = match self.get_node(self.get_active_edge()) {
                        &Node::Internal(InternalNode { start, end, .. }) => end - start,
                        &Node::Leaf(start) => self.position - start + 1,
                    };

                    if self.active_length == length {
                        self.active_node = self.get_active_edge();
                        self.active_edge = 0;
                        self.active_length = 0;
                    }
                }



                break;
            }
        }

        self.position += 1;
    }

    fn visualize_node(&self, node: NodeId) -> Vec<String> {
        match self.get_node(node) {
            &Node::Internal(InternalNode { start, end, ref edges, .. }) => {
                let edge_label = str::from_utf8(&self.text[start..end]).unwrap_or("<invalid_string>");
                let text = format!("({}){}", node, edge_label);
                let mut lines = Vec::new();
                for (i, &child) in edges.values().enumerate() {
                    for (j, line) in self.visualize_node(child).into_iter().enumerate() {
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
        let text = str::from_utf8(&self.text[..self.position]).unwrap_or("<invalid_string>");
        println!("'{}'", text); 
        println!("active point is ({}, '{}', {})", self.active_node, self.active_edge as char, self.active_length);
        println!("step is {}, remaining is {}", self.position, self.remaining);
        for line in self.visualize_node(0) {
            println!("{}", line);
        }

        for (i, ref node) in self.nodes.iter().enumerate() {
            if let Node::Internal(InternalNode { suffix_link: Some(link), .. }) = node {
                println!("Suffix link from {} to {}", i, link);
            }
        }

    }
}
