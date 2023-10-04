//! This module controls the style step, combining the DOM and the CSSOM into
//! a style tree (a render tree).

use crate::css_parser::{Selector, SimpleSelector, Value};
use crate::dom::{ElementData, Node};
use std::collections::HashMap;

/// Map from CSS property names to values.
type PropertyMap = HashMap<String, Value>;

/// A node with associated style data.
struct StyledNode<'a> {
    /// A pointer to a DOM node.
    node: &'a Node,
    specified_values: PropertyMap,
    children: Vec<StyledNode<'a>>,
}

fn matches(elem: &ElementData, selector: &Selector) -> bool {
    match selector {
        Selector::Simple(ref simple_selector) => match_simple_selector(elem, simple_selector),
    }
}

fn match_simple_selector(elem: &ElementData, selector: &SimpleSelector) -> bool {
    // Check type selector.
    match &selector.tag_name {
        Some(tag_name) if tag_name != &elem.tag_name => return false,
        _ => (),
    }

    // Check id selector.
    match (&selector.id, elem.id()) {
        (Some(selector_id), Some(elem_id)) if selector_id != elem_id => return false,
        (Some(_), None) => return false,
        _ => (),
    }

    // Check class selectors.
    let elem_classes = elem.classes();
    selector.class.iter().all(|c| elem_classes.contains(&c[..]))
}
