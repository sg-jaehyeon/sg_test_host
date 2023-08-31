use super::property::*;

use std::fmt;

pub struct DtbNode {
    pub node_name: String,
    pub properties: Vec<DtbProperty>,
    pub child_nodes: Vec<Box<DtbNode>>,
}

impl fmt::Debug for DtbNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DtbNode")
            .field("node_name", &self.node_name)
            .field("properties", &self.properties)
            .field("child_nodes", &self.child_nodes)
            .finish()
    }
}

impl DtbNode {
    pub fn indent(n: usize) -> String {
        "\t".repeat(n)
    }

    pub fn stringify(&self, indent: usize) -> String {
        let mut ret = String::new();

        if self.node_name == "/" && indent == 0 {
            ret.push_str("/dts-v1/;");
            ret.push('\n');
            ret.push('\n');
        }

        ret.push_str(&(Self::indent(indent) + &self.node_name + " {\n")[..]);

        // properties
        for property in &self.properties {
            let mut property_string = Self::indent(indent+1);
            property_string.push_str(&property.key);
            match &property.value {
                Some(value) => {
                    property_string.push_str(" = ");
                    property_string.push_str(&value);
                },
                _ => {}
            }
            property_string.push_str(";\n");

            ret.push_str(&property_string);
        }

        // add new line between properties and child_nodes
        // if there are no child nodes, just close node.
        if self.child_nodes.len() > 0 {
            ret.push_str("\n");
        }

        // child_nodes
        for (idx, node) in self.child_nodes.iter().enumerate() {
            ret.push_str(&node.stringify(indent+1));
            if idx < self.child_nodes.len() - 1 {
                ret.push_str("\n");
            }
        }

        // close node
        ret.push_str(&Self::indent(indent));
        ret.push_str("};\n");

        ret
    }

    pub fn parse(content_vec: &Vec<&str>, brackets: &Vec<(usize, usize)>, index: usize, start: (usize, usize)) -> (usize, DtbNode) {
        let mut ret = DtbNode {
            node_name: String::new(),
            properties: Vec::<DtbProperty>::new(),
            child_nodes: Vec::<Box<DtbNode>>::new(),
        };

        ret.node_name = content_vec[start.0].trim().strip_suffix("{").unwrap().trim().to_string();

        let mut idx = start.0;
        let mut bracket_idx = index;

        loop {
            idx += 1;
            
            if brackets[bracket_idx + 1].0 == idx && brackets[bracket_idx + 1].1 == 0 {
                // open new child node
                let child: DtbNode;
                let result = Self::parse(content_vec, brackets, bracket_idx + 1, brackets[bracket_idx + 1].clone());
                bracket_idx = result.0;
                idx = brackets[bracket_idx].0;
                child = result.1;

                ret.child_nodes.push(Box::new(child));

            } else if brackets[bracket_idx + 1].0 == idx && brackets[bracket_idx + 1].1 == 1 {
                // close
                return (bracket_idx + 1, ret);
            } else if content_vec[idx].contains(" = ") {
                // property with (key, value)
                let v: Vec<&str> = content_vec[idx].trim().split(" = ").map(|s| s.trim()).collect();
                let property = DtbProperty {
                    key: String::from(v[0]),
                    value: Some(String::from(v[1].trim().strip_suffix(";").unwrap())),
                };

                ret.properties.push(property);
            } else if !content_vec[idx].contains("=") && content_vec[idx].contains(";") {
                // property with no value
                let property = DtbProperty {
                    key: String::from(content_vec[idx].trim().strip_suffix(";").unwrap()),
                    value: None,
                };
                ret.properties.push(property);
            }
            
        }

    }

    pub fn find_property(&mut self, key: &str) -> Option<&mut DtbProperty> {
        self.properties.iter_mut().find(|property| property.key == key)
    }

    pub fn find_childnode(&mut self, name: &str) -> Option<&mut Box<DtbNode>> {
        self.child_nodes.iter_mut().find(|node| node.node_name == name)
    }

    pub fn new(content: String) -> DtbNode {
        let mut node = DtbNode {
            node_name: String::new(),
            properties: Vec::<DtbProperty>::new(),
            child_nodes: Vec::<Box<DtbNode>>::new(),
        };

        node._init(content);

        node
    }

    fn _init(&mut self, content: String) {

        // find open, close brackets
        let content_vec: Vec<&str> = content.lines().collect::<Vec<&str>>();

        let mut opens: Vec<(usize, usize)> = content_vec.iter()
                                                    .enumerate()
                                                    .filter(|(_idx, line)| line.contains("{") && !line.trim().starts_with("//"))
                                                    .map(|(idx, _line)| (idx, 0))
                                                    .collect();
        let mut closes: Vec<(usize, usize)> = content_vec.iter()
                                                    .enumerate()
                                                    .filter(|(_idx, line)| line.contains("};") && !line.trim().starts_with("//"))
                                                    .map(|(idx, _line)| (idx, 1))
                                                    .collect();

        opens.append(&mut closes);
        opens.sort_unstable_by(|(idx1, _), (idx2, _)| idx1.cmp(idx2));

        *self = Self::parse(&content_vec, &opens, 0, opens[0].clone()).1;
    }
}