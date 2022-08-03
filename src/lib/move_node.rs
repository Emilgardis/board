use crate::board_logic::{Board, BoardMarker, Point};
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
    pub fn new(edge_node: (EdgeIndex, NodeIndex)) -> MoveIndex {
        MoveIndex {
            node_index: edge_node.1,
            edge_index: Some(edge_node.0),
        }
    }

    pub fn new_node(node: NodeIndex) -> MoveIndex {
        MoveIndex {
            node_index: node,
            edge_index: None,
        }
    }

    pub fn from_option(edge_node_option: Option<(EdgeIndex, NodeIndex)>) -> Option<MoveIndex> {
        edge_node_option.map(|edge_node| MoveIndex {
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

    fn from_str(s: &str) -> Result<MoveIndex, ParseError> {
        let (n, e) = {
            let mut t = s.splitn(2, ' ');
            (t.next(), t.next())
        };
        match e {
            None => Ok(MoveIndex::new_node(n.unwrap().parse::<BigU>()?.into())),
            Some(_e) => Err(ParseError::Other(
                "Edges not currently supported for parsing.".to_string(),
            )),
        }
    }
}

pub struct MoveGraph {
    graph: daggy::Dag<BoardMarker, BigU, BigU>,
    pub marked_for_branch: Vec<NodeIndex>,
}

impl MoveGraph {
    pub fn new() -> MoveGraph {
        MoveGraph {
            graph: daggy::Dag::with_capacity(255, 255),
            marked_for_branch: vec![],
        }
    }

    pub fn new_root(&mut self, marker: BoardMarker) -> MoveIndex {
        MoveIndex::new_node(self.graph.add_node(marker))
    }
    pub fn add_move(&mut self, parent: MoveIndex, marker: BoardMarker) -> MoveIndex {
        MoveIndex::new(self.graph.add_child(parent.node_index, 0, marker))
    }
    // Mark as deprecated
    pub fn get_marker(&self, node: MoveIndex) -> Option<&BoardMarker> {
        self.graph.node_weight(node.node_index)
    }
    pub fn get_move_mut(&mut self, node: MoveIndex) -> Option<&mut BoardMarker> {
        self.graph.node_weight_mut(node.node_index)
    }
    pub fn get_move(&self, node: MoveIndex) -> Option<&BoardMarker> {
        self.graph.node_weight(node.node_index)
    }
    pub fn rm_move(&mut self, node: MoveIndex) -> Option<BoardMarker> {
        self.graph.remove_node(node.node_index)
    }
    pub fn get_children(&self, parent: MoveIndex) -> Vec<MoveIndex> {
        let mut result: Vec<MoveIndex> = Vec::new();
        for child in self.graph.children(parent.node_index).iter(&self.graph) {
            result.push(MoveIndex::new(child));
        }
        result
    }

    pub fn get_parent(&self, child: &MoveIndex) -> Option<MoveIndex> {
        let mut parent = self.graph.parents(child.node_index);
        let result = parent.walk_next(&self.graph);
        if parent.walk_next(&self.graph) != None {
            panic!("Error, shame on me! A MoveNode cannot have two parents!") //FIXME: This error message sucks.
        } else {
            MoveIndex::from_option(result)
        }
    }

    pub fn get_siblings(&self, child: &MoveIndex) -> Vec<MoveIndex> {
        let parent_opt = self.get_parent(child);
        match parent_opt {
            Some(parent) => self.get_children(parent), // Not ideal, should not really return the original child.
            None => Vec::new(),
        }
    }
    // Convenience methods, like set comment, set pos etc. Also walk down node until multiple
    // choices. etc.

    /// Gives a simple vec of all the traversed parents including root.
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

    /// Returns the board as it would look like when end_node was played.
    pub fn as_board(&self, end_node: &MoveIndex) -> Result<Board, ParseError> {
        let mut move_list: Vec<MoveIndex> = self.down_to_root(end_node);
        move_list.push(end_node.clone());
        let mut board: Board = Board::new(15);
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
        board.last_move = self.get_move(end_node.clone()).unwrap().point.into();
        //tracing::info!("board is = {}", board.board);
        Ok(board)
    }
    /// Move up in the tree until there is a branch, i.e multiple choices for the next move.
    ///
    /// Returns the children that were walked  and the children that caused the branch, if any.
    pub fn up_to_branch(&self, node: MoveIndex) -> (Vec<MoveIndex>, Vec<MoveIndex>) {
        // Check if we should wrap the result in an option.
        let mut branch_decendants: Vec<MoveIndex> = Vec::new();
        let mut children = self.get_children(node);
        while children.len() == 1 {
            branch_decendants.push(children[0]); // Do we need to clone? FIXME
            children = self.get_children(children[0]);
        }
        (branch_decendants, children)
    }
    /// Move down in tree until there is a branch, i.e move has multiple children.
    ///
    /// Returns the branching node, if any.
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

    pub fn get_variant(&self, cur_index: Option<&MoveIndex>, point: &Point) -> Option<MoveIndex> {
        // Confusion, this searches the nodes, looking for a down command on the same position, or else, looks for a right with the same position. Why? I don't know. it's always to get the next move
        if let Some(node) = cur_index {
            if let Some(branch) = self.down_to_branch(node) {
                if let Some(pos2) = self.graph.node_weight(branch.node_index) {
                    if &pos2.point == point {
                        return Some(branch);
                    }
                }
            }
        }
        None
    }
}

impl Default for MoveGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for MoveGraph {
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
    use crate::board_logic::*;
    let mut graph = MoveGraph::new();
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
    tracing::info!("Children of {:?} {:?}", b_1, graph.get_children(a_1));
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
