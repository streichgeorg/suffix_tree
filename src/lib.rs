use std::fmt;

#[derive(Debug)]
struct Node {
    children: Option<Vec<usize>>,
    start: usize,
    end: usize,
}

impl Node {
    fn new(start: usize, end: usize) -> Node {
        Node {
            children: None,
            start,
            end,
        }
    }

    fn add_child(&mut self, child_index: usize) {
        match self.children {
            None => {
                self.children = Some(vec![child_index]);
            },
            Some(ref mut children) => {
                children.push(child_index)
            }
        }
    }
}

#[derive(Debug)]
pub struct SuffixTree {
    nodes: Vec<Node>, 
    root: usize,
    text: String,
}

impl fmt::Display for SuffixTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "tree"));
        try!(writeln!(f, "  root   0: children: {:?}", self.get_node(0).children.as_ref().unwrap()));
        for (i, node) in self.nodes.iter().enumerate().skip(1) {
            match node {
                Node {children: Some(ref children), start, end} => {
                    let text: &str = &self.text[*start..*end];
                    try!(writeln!(f, "  parent {}: children: {:?} text: {}", i, children, text));
                },
                Node {children: None, start, end} => {
                    let text: &str = &self.text[*start..*end];
                    try!(writeln!(f, "  child  {}: text: {}", i, text));
                }
            }
        }
         
        write!(f, "")
    }
}

enum TreeLocation {
    Node(usize),
    Edge(usize, usize, usize),
}

impl SuffixTree {
    pub fn new(mut text: String) -> SuffixTree {
        text.push_str("$");
        let mut tree = SuffixTree {
            nodes: vec![Node::new(0, 0)],
            root: 0,
            text,
        };
        tree.build();

        tree
    }

    fn build(&mut self) {
        for i in 0..self.text.len() {
            let tree_location = {
                let suffix = &self.text[i..];
                self.follow_path(self.root, suffix)
            };

            match tree_location {
                (num_matched, _) if num_matched == (self.text.len() - i) => {},
                (num_matched, TreeLocation::Node(node_index)) => {
                    let end = self.text.len();
                    self.add_node_with_parent(node_index, Node::new(i + num_matched, end));
                },
                (num_matched, TreeLocation::Edge(a_index, child, length)) => {
                    let b_index = self.get_child_index(a_index, child);

                    let c_index = self.add_node(Node::new(i + num_matched - length, i + num_matched));

                    let d = Node::new(i + num_matched, self.text.len());
                    self.add_node_with_parent(c_index, d);

                    {
                        let b = self.get_mut_node(b_index);
                        b.start += length;
                    }

                    {
                        let c = self.get_mut_node(c_index);
                        c.add_child(b_index);
                    }

                    {
                        let a = self.get_mut_node(a_index);
                        let children = a.children.as_mut().unwrap();
                        children[child] = c_index;
                    }

                },
            }
        }
    }

    fn follow_path(&self, start_node: usize, path: &str) -> (usize, TreeLocation) {
        let mut current_node = start_node; 
        let mut num_matched = 0;

        loop {
            if let Some(ref children) = self.get_node(current_node).children {
                let mut has_match = false;
                for (i, &child) in children.iter().enumerate() {
                    let node_text = self.get_node_text(child);

                    let remaining_chars = path[num_matched..].chars();
                    let prefix_len = remaining_chars.zip(node_text.chars())
                                                    .take_while(|(a, b)| a == b)
                                                    .count();
                    num_matched += prefix_len;

                    if prefix_len == node_text.len() {
                        current_node = child;
                        has_match = true;
                        break;
                    } else if prefix_len > 0 {
                        return (num_matched, TreeLocation::Edge(current_node, i, prefix_len))
                    }
                }
                if !has_match {
                    return (num_matched, TreeLocation::Node(current_node))
                }
            } else {
                return (num_matched, TreeLocation::Node(current_node))
            }
        }
    }

    fn get_mut_node(&mut self, node_index: usize) -> &mut Node {
        &mut self.nodes[node_index]
    }

    fn get_node(&self, node_index: usize) -> &Node {
        &self.nodes[node_index]
    }

    fn get_node_text(&self, node_index: usize) -> &str {
        let node = self.get_node(node_index);
        &self.text[node.start..node.end]
    }

    fn get_child_index(&self, parent_index: usize, child: usize) -> usize {
        if let Some(ref children) = self.get_node(parent_index).children {
            return children[child]
        }
        panic!();
    }

    fn add_node(&mut self, node: Node) -> usize {
        let node_index = self.nodes.len();
        self.nodes.push(node);

        node_index
    }

    fn add_node_with_parent(&mut self, parent_index: usize, node: Node) -> usize {
        let node_index = self.nodes.len();
        self.nodes.push(node);

        self.get_mut_node(parent_index).add_child(node_index);

        node_index
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
