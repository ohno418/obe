//! This module controls the style step, combining the DOM and the CSSOM into
//! a style tree (a render tree).

use crate::css_parser::Value;
use crate::dom::Node;
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
