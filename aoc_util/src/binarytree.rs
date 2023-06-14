use crate::errors::{failure, AocResult};
use std::cell::RefCell;
use std::fmt;
use std::rc::{Rc, Weak};

pub type NodeLink = Rc<RefCell<Node>>;

#[derive(Clone, Debug)]
pub struct Node {
    data: Option<i64>,
    left: Option<NodeLink>,
    right: Option<NodeLink>,
    parent: Option<Weak<RefCell<Node>>>,
}

impl Node {
    pub fn new(data: Option<i64>) -> NodeLink {
        Rc::new(RefCell::new(Node {
            data,
            left: None,
            right: None,
            parent: None,
        }))
    }

    pub fn new_with_parent(data: Option<i64>, parent: &NodeLink) -> NodeLink {
        Rc::new(RefCell::new(Node {
            data,
            left: None,
            right: None,
            parent: Some(Rc::downgrade(parent)),
        }))
    }
}

#[derive(Clone, Debug)]
pub struct NodeWrapper(NodeLink);

impl From<NodeLink> for NodeWrapper {
    fn from(n: NodeLink) -> NodeWrapper {
        NodeWrapper(n)
    }
}

impl fmt::Display for NodeWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO currently only supports trees with (required) data at leaves.
        if self.is_leaf() && !self.has_data() {
            panic!("Invalid tree: leaf with no data");
        }
        if !self.is_leaf() && self.has_data() {
            panic!("Invalid tree: non-leaf with data");
        }
        if let Some(data) = self.get_data() {
            write!(f, "{}", data)
        } else {
            let left_string = self.get_left().unwrap().to_string();
            let right_string = self.get_right().unwrap().to_string();
            write!(
                f,
                "{}",
                "[".to_string() + left_string.as_str() + "," + right_string.as_str() + "]"
            )
        }
    }
}

impl NodeWrapper {
    pub fn new() -> NodeWrapper {
        Self(Node::new(None))
    }
    pub fn get_left(&self) -> Option<NodeWrapper> {
        self.0
            .borrow()
            .left
            .as_ref()
            .map(|left| left.clone().into())
    }

    pub fn get_right(&self) -> Option<NodeWrapper> {
        self.0
            .borrow()
            .right
            .as_ref()
            .map(|right| right.clone().into())
    }

    pub fn get_data(&self) -> Option<i64> {
        self.0.borrow().data
    }

    pub fn get_parent(&self) -> Option<NodeWrapper> {
        self.0
            .borrow()
            .parent
            .as_ref()
            .map(|parent| parent.upgrade().unwrap().into())
    }

    pub fn set_left(&self, child: Option<&NodeWrapper>) {
        if let Some(child) = child {
            self.0.borrow_mut().left = Some(child.0.clone());
            child.0.borrow_mut().parent = Some(Rc::downgrade(&self.0));
        } else {
            self.0.borrow_mut().left = None
        }
    }

    pub fn set_right(&self, child: Option<&NodeWrapper>) {
        if let Some(child) = child {
            self.0.borrow_mut().right = Some(child.0.clone());
            child.0.borrow_mut().parent = Some(Rc::downgrade(&self.0));
        } else {
            self.0.borrow_mut().right = None
        }
    }

    pub fn set_data(&self, data: Option<i64>) {
        self.0.borrow_mut().data = data;
    }

    pub fn is_leaf(&self) -> bool {
        self.get_left().is_none() && self.get_right().is_none()
    }

    pub fn has_data(&self) -> bool {
        self.get_data().is_some()
    }

    pub fn depth_first_iter(&self) -> DepthFirstIterator {
        DepthFirstIterator::new(&self.0)
    }

    pub fn from_ascii(ascii: &[u8]) -> AocResult<NodeWrapper> {
        Ok(NodeWrapper::_from_ascii(ascii)?.0)
    }

    pub fn inner(&self) -> NodeLink {
        self.0.clone()
    }

    /// Parses a NodeLink from a line of ASCII of the form:
    /// "[[1,2],[3,[4,5]]]" etc.
    /// Current limitations: no whitespace, only single digit numbers supported.
    fn _from_ascii(ascii: &[u8]) -> AocResult<(NodeWrapper, usize)> {
        if ascii[0] != b'[' {
            return failure("Invalid line start");
        }

        let mut consumed = 0;
        let mut seen_comma = false;
        let mut seen_opening_bracket = false;
        let mut pair = Vec::new();

        // Another implicit state machine :(.
        loop {
            let c = ascii[consumed];
            match c {
                b'[' => {
                    if seen_opening_bracket {
                        let (node, cons) = NodeWrapper::_from_ascii(&ascii[consumed..])?;
                        consumed += cons;
                        pair.push(node);
                    } else {
                        seen_opening_bracket = true;
                        consumed += 1;
                    }
                }
                b'0'..=b'9' => {
                    if (!seen_comma && !pair.is_empty()) || (seen_comma && pair.is_empty()) {
                        return failure("Invalid digit location");
                    }
                    pair.push(Node::new(Some((c - 48) as i64)).into());
                    consumed += 1;
                }
                b',' => {
                    if seen_comma {
                        return failure("Two commas in a node");
                    }
                    seen_comma = true;
                    consumed += 1;
                }
                b']' => {
                    if !seen_comma {
                        return failure("No comma in a node");
                    }
                    if pair.len() != 2 {
                        return failure(format!("Invalid 'pair': {:?}", pair));
                    }
                    consumed += 1;
                    let node = NodeWrapper::from(Node::new(None));
                    node.set_left(Some(&pair.remove(0)));
                    node.set_right(Some(&pair.remove(0)));
                    return Ok((node, consumed));
                }
                _ => return failure("Invalid character"),
            }
        }
    }
}

impl Default for NodeWrapper {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DepthFirstIterator {
    stack: Vec<(NodeLink, usize)>,
}

impl DepthFirstIterator {
    pub fn new(node: &NodeLink) -> Self {
        let stack = vec![(node.clone(), 0)];
        DepthFirstIterator { stack }
    }
}

impl Iterator for DepthFirstIterator {
    type Item = (NodeWrapper, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.stack.is_empty() {
            let (node, depth) = self.stack.pop().unwrap();

            // Push right first so that we pop left first.
            if let Some(right) = node.borrow().right.clone() {
                self.stack.push((right, depth + 1));
            };
            if let Some(left) = node.borrow().left.clone() {
                self.stack.push((left, depth + 1));
            }
            return Some((node.into(), depth));
        }
        None
    }
}

#[cfg(test)]
mod nodewrapper_tests {
    use super::*;

    #[test]
    fn nodewrapper_from_ascii() -> AocResult<()> {
        for s in [
            "[1,2]",
            "[[1,2],3]",
            "[1,[2,3]]",
            "[[1,2],[3,4]]",
            "[[[[[1,2],3],[4,5]],6],[7,[[8,9],0]]]",
        ] {
            let t = NodeWrapper::from_ascii(s.as_bytes())?;
            assert_eq!(s.to_string(), t.to_string());
        }
        Ok(())
    }

    #[test]
    fn nodewrapper_depth_first_traversal() -> AocResult<()> {
        for (s, v, d) in [
            ("[1,2]", vec![1, 2], vec![1, 1]),
            ("[[1,2],3]", vec![1, 2, 3], vec![2, 2, 1]),
            ("[1,[2,3]]", vec![1, 2, 3], vec![1, 2, 2]),
            ("[[1,2],[3,4]]", vec![1, 2, 3, 4], vec![2, 2, 2, 2]),
            (
                "[[[[[1,2],3],[4,5]],6],[7,[[8,9],0]]]",
                vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0],
                vec![5, 5, 4, 4, 4, 2, 2, 4, 4, 3],
            ),
        ] {
            let t = NodeWrapper::from_ascii(s.as_bytes())?;
            let data = t
                .depth_first_iter()
                .filter_map(|(node, _depth)| node.get_data())
                .collect::<Vec<_>>();
            let depths = t
                .depth_first_iter()
                .filter_map(|(node, depth)| {
                    if node.get_data().is_some() {
                        Some(depth)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            assert_eq!(data, v);
            assert_eq!(depths, d);
        }
        Ok(())
    }
}
