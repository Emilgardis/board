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
    pub children: Result<Vec<Box<MoveNode>>, NodeError>,
}
pub struct MoveNode {
    pub parent: Option<Box<MoveNode>>,
    pub children: Option<Vec<Box<MoveNode>>>,
    pub marker: BoardMarker,
    pub one_line_comment: &'static str,
    pub multi_line_comment: &'static str,
}
