// extern use /*module with support for mutable nodes that keep their structure */ 

use board_logic::{BoardMarker, Point, Stone};
//use std::cell::RefCell;

pub enum NodeError {
    IsRoot,
    NoChildren,
}

pub trait Node {
    fn add_child(&self) -> Result<MoveNode, NodeError>;
}

pub struct RootNode {
    children: Result<Vec<Box<MoveNode>>, NodeError>,
}
pub struct MoveNode {
    parent: Option<Box<MoveNode>>,
    children: Option<Vec<Box<MoveNode>>>,
    marker: BoardMarker,
}


