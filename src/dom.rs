use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq)]
pub struct Node {
    pub children: Vec<Node>,
    pub node_type: NodeType,
}

#[derive(Debug, PartialEq)]
pub enum NodeType {
    Text(String),
    Element(ElementData),
}

#[derive(Debug, PartialEq)]
pub struct ElementData {
    pub tag_name: String,
    attributes: AttrMap,
}

pub type AttrMap = HashMap<String, String>;

pub fn text(data: String) -> Node {
    Node {
        children: Vec::new(),
        node_type: NodeType::Text(data),
    }
}

pub fn elem(name: String, attrs: AttrMap, children: Vec<Node>) -> Node {
    Node {
        children,
        node_type: NodeType::Element(ElementData {
            tag_name: name,
            attributes: attrs,
        }),
    }
}

impl ElementData {
    pub fn id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    pub fn classes(&self) -> HashSet<&str> {
        match self.attributes.get("class") {
            Some(classlist) => classlist.split(' ').collect(),
            None => HashSet::new(),
        }
    }
}

#[cfg(test)]
mod element_data_tests {
    use super::*;

    #[test]
    fn id() {
        let elem = ElementData {
            tag_name: "div".to_string(),
            attributes: HashMap::from([
                ("id".to_string(), "main".to_string()),
                ("class".to_string(), "class1 class2".to_string()),
            ]),
        };
        assert_eq!(elem.id().unwrap(), "main");

        let elem = ElementData {
            tag_name: "div".to_string(),
            attributes: HashMap::from([("class".to_string(), "class1 class2".to_string())]),
        };
        assert!(elem.id().is_none());
    }

    #[test]
    fn classes() {
        let elem = ElementData {
            tag_name: "div".to_string(),
            attributes: HashMap::from([
                ("id".to_string(), "main".to_string()),
                ("class".to_string(), "class1 class2".to_string()),
            ]),
        };
        assert_eq!(elem.classes(), HashSet::from(["class1", "class2"]));

        let elem = ElementData {
            tag_name: "div".to_string(),
            attributes: HashMap::from([("id".to_string(), "main".to_string())]),
        };
        assert_eq!(elem.classes(), HashSet::from([]));
    }
}
