use crate::board_logic::{BoardMarker, DisplayBoard, Point};
use crate::errors::ParseError;
use daggy;
use daggy::Walker;
use std::fmt;

use std::str::FromStr;

pub type BigU = usize;
pub type NodeIndex = daggy::NodeIndex<BigU>;
pub type EdgeIndex = daggy::EdgeIndex<BigU>;

//unsafe impl daggy::petgraph::IndexType for BigU {
//    #[inline(always)]
//    fn new (x: BigU) -> Self { x }
//    fn index(&self) -> Self { *self }
//    fn max() -> Self { BigUint::MAX }
//}

//pub type MoveGraph = daggy::Dag<NodeIndex, EdgeIndex>;

#[derive(Clone, Copy, PartialEq)]
pub struct MoveIndex {
    node_index: NodeIndex,
    edge_index: Option<EdgeIndex>,
}

impl MoveIndex {
    #[must_use]
    pub fn new(edge_node: (EdgeIndex, NodeIndex)) -> Self {
        Self {
            node_index: edge_node.1,
            edge_index: Some(edge_node.0),
        }
    }

    #[must_use]
    pub fn new_node(node: NodeIndex) -> Self {
        Self {
            node_index: node,
            edge_index: None,
        }
    }

    #[must_use]
    pub fn from_option(edge_node_option: Option<(EdgeIndex, NodeIndex)>) -> Option<Self> {
        edge_node_option.map(|edge_node| Self {
            node_index: edge_node.1,
            edge_index: Some(edge_node.0),
        })
    }
}

impl fmt::Debug for MoveIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "N: {:?}", self.node_index.index())?;
        match self.edge_index {
            Some(_) => write!(f, " E: {:?}", self.edge_index.unwrap().index()),
            None => write!(f, " E: None"),
        }
    }
}

impl FromStr for MoveIndex {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        let (n, e) = {
            let mut t = s.splitn(2, ' ');
            (t.next(), t.next())
        };
        match e {
            None => Ok(Self::new_node(n.unwrap().parse::<BigU>()?.into())),
            Some(_e) => Err(ParseError::Other(
                "Edges not currently supported for parsing.".to_string(),
            )),
        }
    }
}

pub struct Board {
    graph: daggy::Dag<BoardMarker, BigU, BigU>,
    pub marked_for_branch: Vec<NodeIndex>,
    /// List of moves currently done
    move_list: Vec<MoveIndex>,
    index: usize,
}

impl Board {
    #[must_use]
    pub fn new() -> Self {
        let mut board = Self {
            graph: daggy::Dag::with_capacity(255, 255),
            marked_for_branch: vec![],
            move_list: vec![],
            index: 0,
        };

        let root = board.new_root(BoardMarker::null_move());
        board.move_list.push(root);
        board
    }

    fn new_root(&mut self, marker: BoardMarker) -> MoveIndex {
        MoveIndex::new_node(self.graph.add_node(marker))
    }

    pub fn add_move(&mut self, parent: MoveIndex, marker: BoardMarker) -> MoveIndex {
        MoveIndex::new(self.graph.add_child(parent.node_index, 0, marker))
    }
    pub fn add_move_to_move_list(&mut self, index: MoveIndex) {
        self.move_list.push(index);
        self.index = self.index.checked_add(1).unwrap();
    }

    pub fn get_move_mut(&mut self, node: MoveIndex) -> Option<&mut BoardMarker> {
        self.graph.node_weight_mut(node.node_index)
    }

    #[must_use]
    pub fn get_move(&self, node: MoveIndex) -> Option<&BoardMarker> {
        self.graph.node_weight(node.node_index)
    }

    pub fn rm_move(&mut self, node: MoveIndex) -> Option<BoardMarker> {
        self.graph.remove_node(node.node_index)
    }

    #[must_use]
    pub fn get_children(&self, parent: &MoveIndex) -> Vec<MoveIndex> {
        let mut result: Vec<MoveIndex> = Vec::new();
        for child in self.graph.children(parent.node_index).iter(&self.graph) {
            result.push(MoveIndex::new(child));
        }
        result
    }

    #[must_use]
    pub fn get_parent(&self, child: &MoveIndex) -> Option<MoveIndex> {
        let mut parent = self.graph.parents(child.node_index);
        let result = parent.walk_next(&self.graph);
        if parent.walk_next(&self.graph) != None {
            panic!("Error, shame on me! A MoveNode cannot have two parents!") //FIXME: This error message sucks.
        } else {
            MoveIndex::from_option(result)
        }
    }

    #[must_use]
    pub fn get_siblings(&self, child: &MoveIndex) -> Vec<MoveIndex> {
        let parent_opt = self.get_parent(child);
        match parent_opt {
            Some(parent) => self.get_children(&parent), // Not ideal, should not really return the original child.
            None => Vec::new(),
        }
    }
    // Convenience methods, like set comment, set pos etc. Also walk down node until multiple
    // choices. etc.

    /// Gives a simple vec of all the traversed parents including root.
    #[must_use]
    pub fn down_to_root(&self, node: &MoveIndex) -> Vec<MoveIndex> {
        let mut parent: Option<MoveIndex> = self.get_parent(node);
        if parent.is_none() {
            return vec![];
        };

        let mut result: Vec<MoveIndex> = vec![parent.unwrap()];
        while let Some(new_parent) = parent {
            result.push(new_parent);
            parent = self.get_parent(&new_parent);
        }
        result
    }

    /// Gives the amount of moves to travel to root.
    #[must_use]
    pub fn moves_to_root(&self, node: &MoveIndex) -> usize {
        let mut parent: Option<MoveIndex> = self.get_parent(node);
        if parent.is_none() {
            return 0;
        };
        let mut length = 0;
        while let Some(new_parent) = parent {
            length += 1;
            parent = self.get_parent(&new_parent);
        }
        length
    }

    /// Returns the board as it would look like when `end_node` was played.
    pub fn as_board(&self, end_node: &MoveIndex) -> Result<DisplayBoard, ParseError> {
        let mut move_list: Vec<MoveIndex> = self.down_to_root(end_node);
        move_list.push(*end_node);
        let mut board: DisplayBoard = DisplayBoard::new(15);
        for index_marker in move_list {
            board.set(match self.get_move(index_marker) {
                Some(val) => val.clone(),
                None => {
                    return Err(ParseError::Other(format!(
                        "Couldn't get move at: {:?}",
                        index_marker
                    )))
                }
            })?;
        }
        //tracing::info!("board is = {}", board.board);
        board.last_move = self.get_move(*end_node).unwrap().point.into();
        //tracing::info!("board is = {}", board.board);
        Ok(board)
    }
    /// Move up in the tree until there is a branch, i.e multiple choices for the next move.
    ///
    /// Returns the children that were walked  and the children that caused the branch, if any.
    #[must_use]
    pub fn up_to_branch(&self, node: &MoveIndex) -> (Vec<MoveIndex>, Vec<MoveIndex>) {
        // Check if we should wrap the result in an option.
        let mut branch_decendants: Vec<MoveIndex> = Vec::new();
        let mut children = self.get_children(node);
        while children.len() == 1 {
            branch_decendants.push(children[0]); // Do we need to clone? FIXME
            children = self.get_children(&children[0]);
        }
        (branch_decendants, children)
    }
    /// Move down in tree until there is a branch, i.e move has multiple children.
    ///
    /// Returns the branching node, if any.
    #[must_use]
    pub fn down_to_branch(&self, node: &MoveIndex) -> Option<MoveIndex> {
        let mut branch_ancestors: Vec<MoveIndex> = Vec::new();
        let mut parent: Option<MoveIndex> = self.get_parent(node);

        // Ehm... FIXME: Not sure if this is right. We want to go down to branch, even if it is close.
        let mut siblings: Vec<MoveIndex> = self.get_siblings(node);
        while parent.is_some() && siblings.len() == 1 {
            if self
                .marked_for_branch
                .iter()
                .any(|m| m == &parent.unwrap().node_index)
            {
                break;
            }
            // If it is a lonechild len of siblings will be 1.
            let parentunw: MoveIndex = parent.unwrap(); // Safe as parent must be some for this code to run.
            branch_ancestors.push(parentunw); // Same as in fn down_to_branch, FIXME
            parent = self.get_parent(&parentunw);
            siblings = self.get_siblings(&parentunw);
            // If a node is marked as a branch then it is also a branch.
            // FIXME: Is this correct?
        }
        parent
    }
    /// Change the move at **node**
    ///
    /// Returns Ok(()) if success
    pub fn set_pos(&mut self, node: MoveIndex, point: Point) -> Result<(), ParseError> {
        {
            let marker: &mut BoardMarker = match self.get_move_mut(node) {
                Some(val) => val,
                None => {
                    return Err(ParseError::Other(format!(
                        "Couldn't set position: {:?} at node {:?}",
                        point, node
                    )))
                }
            };
            marker.set_pos(&point);
        }
        Ok(())
    }

    pub fn mark_for_branch(&mut self, node: MoveIndex) {
        self.marked_for_branch.push(node.node_index);
    }

    // get the last branch.
    #[must_use]
    pub fn get_down(&self, index: &MoveIndex) -> Option<MoveIndex> {
        self.down_to_branch(index)
    }

    // get the last branch.
    #[must_use]
    pub fn get_right(&self, index: &MoveIndex) -> Vec<MoveIndex> {
        self.up_to_branch(index).0
    }

    #[must_use]
    pub fn get_variant(&self, index: &MoveIndex, point: &Point) -> Option<MoveIndex> {
        // this function does something.
        if let Some(node) = self.get_down(index) {
            if let Some(BoardMarker { point: point2, .. }) = self.get_move(node) {
                if point2 == point {
                    return Some(node);
                } else {
                    let node = node;
                    for right in self.get_right(&node) {
                        if let Some(BoardMarker { point: point2, .. }) = self.get_move(right) {
                            if point2 == point {
                                return Some(node);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    #[must_use]
    pub fn current_move(&self) -> MoveIndex {
        *self
            .move_list
            .get(self.index)
            .expect("index should be up to date with move_list")
    }

    #[must_use]
    pub fn get_root(&self) -> MoveIndex {
        *self
            .move_list
            .get(0)
            .expect("move_list should never be empty")
    }

    #[must_use]
    pub fn next_move(&self) -> Option<MoveIndex> {
        self.move_list.get(self.index + 1).copied()
    }

    pub fn move_to_root(&mut self) {
        self.index = 0;
    }

    #[must_use]
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn set_index(&mut self, index: usize) -> Result<(), IndexOutOfBoundsError> {
        if index <= self.move_list.len() {
            self.index = index;
            // self.move_list = self.move_list[..index].to_owned();
            Ok(())
        } else {
            Err(IndexOutOfBoundsError)
        }
    }
}

#[derive(thiserror::Error, Clone, Debug)]
#[error("index is out of bounds")]
pub struct IndexOutOfBoundsError;

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?}",
            daggy::petgraph::dot::Dot::with_config(
                self.graph.graph(),
                &[/*daggy::petgraph::dot::Config::EdgeIndexLabel,daggy::petgraph::dot::Config::NodeIndexLabel*/]
            )
        )?;
        Ok(())
    }
}

#[test]
fn does_it_work() {
    use crate::board_logic::{BoardMarker, Point, Stone};
    let mut graph = Board::new();
    let a = graph.new_root(BoardMarker::new(Point::new(7, 7), Stone::Black));
    let b_1 = BoardMarker::new(Point::new(8, 7), Stone::White);
    let a_1 = graph.add_move(a, b_1.clone());
    let b_2 = BoardMarker::new(Point::new(9, 7), Stone::Black);
    let _a_2 = graph.add_move(a, b_2);
    let b_1_1 = BoardMarker::new(Point::new(10, 7), Stone::White);
    let a_1_1 = graph.add_move(a_1, b_1_1);
    let b_1_2 = BoardMarker::new(Point::new(11, 7), Stone::Black);
    let a_1_2 = graph.add_move(a_1, b_1_2);
    let b_1_2_1 = BoardMarker::new(Point::new(12, 7), Stone::White);
    let a_1_2_1 = graph.add_move(a_1_2, b_1_2_1);
    let b_1_2_1_1 = BoardMarker::new(Point::new(8, 4), Stone::Black);
    let _a_1_2_1_1 = graph.add_move(a_1_2_1, b_1_2_1_1);
    let b_1_2_1_2 = BoardMarker::new(Point::new(7, 4), Stone::Black);
    let a_1_2_1_2 = graph.add_move(a_1_2_1, b_1_2_1_2);
    {
        let a_1_1_b = graph.get_move_mut(a_1_1).unwrap();
        *a_1_1_b = BoardMarker::new(Point::new(14, 14), Stone::White);
    }
    // for i in
    tracing::info!("{:?}", graph);
    tracing::info!("Children of {:?} {:?}", b_1, graph.get_children(&a_1));
    let branched_down = graph.down_to_branch(&a_1_2);
    tracing::info!(
        "Moving down on {:?} gives: end = {:?}",
        a_1_2,
        branched_down
    );
    tracing::info!(
        "Board from a_1_2_1_2\n{}",
        graph.as_board(&a_1_2_1_2).unwrap().board
    );
    // let branched_up = graph.up_to_branch()
    //NOTE:FIXME:TODO: Add asserts!!
}
