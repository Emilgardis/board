use crate::board_logic::{BoardArr, BoardMarker, Point, Stone};
use crate::errors::ParseError;
use daggy;
use daggy::Walker;
use std::{collections::BTreeSet, fmt};

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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Board {
    graph: daggy::Dag<BoardMarker, BigU, BigU>,
    /// List of moves currently done
    move_list: Vec<MoveIndex>,
    index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum VariantType {
    /// Is a variant which may be a transformation also.
    Variant,
    /// Is a transformation of this same branch
    Transformation,
}

impl Board {
    #[must_use]
    pub fn new() -> Self {
        let mut board = Self {
            graph: daggy::Dag::with_capacity(255, 255),
            move_list: vec![],
            index: 0,
        };

        let root = board.new_root(BoardMarker::null());
        board.move_list.push(root);
        board
    }

    fn new_root(&mut self, marker: BoardMarker) -> MoveIndex {
        MoveIndex::new_node(self.graph.add_node(marker))
    }

    #[tracing::instrument(skip(self))]
    pub fn insert_move(&mut self, parent: MoveIndex, marker: BoardMarker) -> MoveIndex {
        tracing::trace!(
            index_in_file = format!("0x{:X}", marker.index_in_file.unwrap_or_default()),
            "inserting move to graph"
        );
        MoveIndex::new(self.graph.add_child(parent.node_index, 255, marker))
    }

    #[tracing::instrument(skip(self))]
    pub fn add_edge(
        &mut self,
        left: &MoveIndex,
        right: &MoveIndex,
    ) -> Result<(), daggy::WouldCycle<usize>> {
        self.graph
            .add_edge(left.node_index, right.node_index, 0)
            .map(|_| ())
    }
    /// Add move to graph and move_list
    pub fn add_move(&mut self, parent: MoveIndex, marker: BoardMarker) -> MoveIndex {
        let idx = self.insert_move(parent, marker.clone());
        if marker.command.is_move() {
            self.add_move_to_move_list(idx);
        }
        idx
    }
    #[tracing::instrument(skip(self))]
    pub fn add_move_to_move_list(&mut self, index: MoveIndex) {
        tracing::trace!(move_list = ?self.move_list, "adding move to move list");
        self.move_list.push(index);
        self.index = self.index.checked_add(1).unwrap();
    }

    pub fn set_moves(&mut self, idx: usize, list: Vec<MoveIndex>) {
        self.move_list = list;
        self.index = idx;
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
    #[tracing::instrument(skip(self))]
    pub fn get_parent_strong(&self, child: &MoveIndex) -> Option<MoveIndex> {
        let mut parents = self.graph.parents(child.node_index);
        if let Some(mut parent) = parents.walk_next(&self.graph) {
            while let Some(other) = parents.walk_next(&self.graph) {
                if other.0 > parent.0 {
                    parent = other;
                    tracing::debug!("found better fit for parent");
                }
            }
            Some(MoveIndex::new(parent))
        } else {
            None
        }
    }

    #[must_use]
    pub fn get_siblings(&self, child: &MoveIndex) -> Vec<MoveIndex> {
        let parent_opt = self.get_parent_strong(child);
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
        let mut parent: Option<MoveIndex> = self.get_parent_strong(node);
        if parent.is_none() {
            return vec![*node];
        };

        let mut result: Vec<MoveIndex> = vec![*node];
        while let Some(new_parent) = parent {
            result.push(new_parent);
            parent = self.get_parent_strong(&new_parent);
        }
        result
    }

    /// Gives the amount of moves to travel to root.
    #[must_use]
    pub fn moves_to_root(&self, node: &MoveIndex) -> usize {
        let mut parent: Option<MoveIndex> = self.get_parent_strong(node);
        if parent.is_none() {
            return 0;
        };
        let mut length = 0;
        while let Some(new_parent) = parent {
            length += 1;
            parent = self.get_parent_strong(&new_parent);
        }
        length
    }

    /// Returns the board as it would look like when `end_node` was played.
    pub fn as_board(&self, end_node: &MoveIndex) -> Result<(BoardArr, Vec<Point>), ParseError> {
        let move_list: Vec<MoveIndex> = self.down_to_root(end_node);
        let mut moves: Vec<Point> = Vec::with_capacity(move_list.len());

        let mut board: BoardArr = BoardArr::new(15);
        for index_marker in move_list.iter().rev() {
            let m = match self.get_move(*index_marker) {
                Some(val) => val.clone(),
                None => {
                    return Err(ParseError::Other(format!(
                        "Couldn't get move at: {:?}",
                        index_marker
                    )))
                }
            };
            if m.command.is_move() {
                moves.push(m.point)
            };
            if !m.point.is_null {
                board.set(m)?;
            }
        }
        //tracing::info!("board is = {}", board.board);
        Ok((board, moves))
    }
    /// Move up in the tree until there is a branch, i.e multiple choices for the next move, or no more moves.
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
    /// Returns the branching node, e.g the node which has multiple children, if any.
    #[must_use]
    pub fn down_to_branch(&self, node: &MoveIndex) -> Option<MoveIndex> {
        let mut parent: Option<MoveIndex> = self.get_parent_strong(node);

        // Ehm... FIXME: Not sure if this is right. We want to go down to branch, even if it is close.
        let mut siblings: Vec<MoveIndex> = self.get_siblings(node);
        // while siblings is not multiple.
        while parent.is_some() && siblings.len() == 1 {
            // If it is a lonechild len of siblings will be 1.
            let parentunw: MoveIndex = parent.unwrap(); // Safe as parent must be some for this code to run.
            parent = self.get_parent_strong(&parentunw);
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

    /// get the first branch below this node. The children of this node are the branches
    ///
    /// Only works if the node is marked as down
    #[must_use]
    pub fn get_down(&self, index: &MoveIndex) -> Option<MoveIndex> {
        self.down_to_branch(index)
    }

    /// get the branches next to this node
    #[must_use]
    pub fn get_right(&self, index: &MoveIndex) -> Vec<MoveIndex> {
        self.get_children(index)
    }

    /// Get indexes for all paths which lead to the same outcome
    #[tracing::instrument(skip(self))]
    pub fn get_variants_and_transformations(
        &self,
        index: MoveIndex,
    ) -> Result<Vec<(BoardMarker, MoveIndex, Transformation, VariantType)>, ParseError> {
        // recursive walk up the tree, discarding all branches that don't fit.
        fn walk_up(
            walked: Vec<(Point, &Stone, &MoveIndex)>,
            graph: &Board,
            move_list: &Vec<(&Point, &Stone, &MoveIndex)>,
            index: MoveIndex,
        ) -> Vec<(BoardMarker, MoveIndex, Transformation, VariantType)> {
            tracing::debug!(?walked, ?move_list, ?index);
            // FIXME: currently we assume that no node can hit the same point twice, that might not be wise.

            tracing::trace!("{:?}", &walked[..walked.len().saturating_sub(1)]);
            tracing::trace!("{move_list:?}");

            let mut diff_explored = 0;
            'transform: for transform in Transformation::types() {
                // FIXME: single H8 is special, there are no valid variants on it except identity.
                if transform != Transformation::identity()
                    && matches!(
                        move_list[..],
                        [(
                            &Point {
                                x: 7,
                                y: 7,
                                is_null: false
                            },
                            ..
                        )]
                    )
                {
                    diff_explored += 1;
                    continue;
                }

                let mut diff = None;
                let mut walked = walked
                    .iter()
                    .map(|(p, c, i)| (transform.apply(*p), c, i))
                    .collect::<Vec<_>>();

                // FIXME: We should discard transforms we already know are not possible.
                // We could just check the last two moves walked I think
                for (point, stone, &index) in &walked {
                    if !move_list.iter().any(|(p, s, _)| (p, s) == (&&point, stone)) {
                        // if there's two mismatches, this couldn't possible be the right path...
                        if diff.is_some() {
                            tracing::trace!("found mismatches");
                            diff_explored += 1;
                            continue 'transform;
                        }
                        diff = Some((point, index));
                    }
                }

                if walked.len() == move_list.len() + 1 && diff.is_some() {
                    tracing::debug!("diff {diff:?}, walked: {walked:?}, move_list: {move_list:?}");
                    // if exactly the same path, not a variant...
                    let mut same = true;
                    for (w, ml) in walked
                        .iter()
                        .filter(|m| !m.1.is_empty())
                        .zip(move_list.iter().filter(|m| !m.1.is_empty()))
                    {
                        if w.0 != *ml.0 || w.1 != &ml.1 {
                            same = false;
                            break;
                        }
                    }
                    if same && transform == Transformation::identity() {
                        continue;
                    }
                    if same
                        && matches!(
                            transform,
                            Transformation {
                                rotation: Rotation::None,
                                mirror: Mirror::Horizontal | Mirror::Vertical
                            }
                        )
                    {
                        continue;
                    }
                    let diff = diff.unwrap();
                    // we've found a variant, return it.
                    let mut marker = graph.get_move(*diff.1).unwrap().clone();
                    marker.point = *diff.0;
                    tracing::debug!("we got a variant on {transform:?}, {marker:?} at {index:?}");
                    let variant_type = if move_list
                        .iter()
                        .map(|(p, ..)| p)
                        .zip(walked.iter().map(|(p, ..)| p))
                        .any(|(p1, p2)| p1 != &p2)
                    {
                        VariantType::Variant
                    } else {
                        VariantType::Transformation
                    };
                    return vec![(marker, index, transform, variant_type)];
                }
            }
            if diff_explored == Transformation::types().len() {
                return vec![];
            }
            let children = graph.get_children(&index);
            tracing::trace!(?children);

            let mut result = Vec::new();
            for child in children {
                if let Some(child_m) = graph.get_move(child) {
                    let mut new_walked = walked.clone();
                    new_walked.push((child_m.point, &child_m.color, &child));
                    result.extend(walk_up(new_walked, graph, move_list, child));
                } else {
                    todo!()
                }
            }
            result
        }

        let mut moves = self
            .move_list()
            .iter()
            .filter_map(|mi| Some((self.get_move(*mi)?, mi)))
            .filter(|(m, _)| !m.color.is_empty())
            .map(|(m, mi)| (&m.point, &m.color, mi))
            .collect::<Vec<_>>();
        if moves.is_empty() {
            return Ok(vec![]);
        }
        tracing::info!(moves = moves.len().saturating_sub(1), "starting walk");
        let result = walk_up(Default::default(), self, &moves, self.get_root());
        tracing::info!("walk ended");
        Ok(result)
    }

    /// Only used for renlib parsing
    ///
    /// RenLib/RenLibDoc.cpp:625
    #[must_use]
    pub(crate) fn get_variant_weird(
        &self,
        index: &MoveIndex,
        point: &Point,
        color: &Stone,
    ) -> Option<(&BoardMarker, MoveIndex)> {
        // this function does something.
        // Get the first branch below this node
        if let Some(node) = self.get_down(index) {
            if let Some(
                marker @ BoardMarker {
                    point: point2,
                    color: color2,
                    command,
                    ..
                },
            ) = self.get_move(node)
            {
                if !command.is_down() {
                    return None;
                }
                // if that branch is the same point return it.
                if point2 == point {
                    return Some((marker, node));
                } else if command.is_right() {
                    // Get
                    for right in self.get_right(&node) {
                        if let Some(marker @ BoardMarker { point: point2, .. }) =
                            self.get_move(right)
                        {
                            if point2 == point {
                                return Some((marker, right));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    #[must_use]
    #[track_caller]
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
    pub fn prev_move(&self) -> Option<MoveIndex> {
        self.move_list.get(self.index.checked_sub(1)?).copied()
    }

    #[must_use]
    pub fn next_move(&self) -> Option<MoveIndex> {
        self.move_list.get(self.index + 1).copied()
    }

    pub fn move_to_root(&mut self) {
        self.set_index(0).unwrap();
    }

    #[must_use]
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn set_index(&mut self, index: usize) -> Result<(), IndexOutOfBoundsError> {
        if index <= self.move_list.len() {
            self.index = index;
            self.move_list.truncate(index + 1);
            Ok(())
        } else {
            Err(IndexOutOfBoundsError)
        }
    }

    pub fn move_list(&self) -> &[MoveIndex] {
        self.move_list.as_ref()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Transformation {
    pub rotation: Rotation,
    pub mirror: Mirror,
}
impl Transformation {
    pub fn rotate(&mut self, rotation: Rotation) {
        self.rotation.rotate(rotation);
    }
    pub const fn identity() -> Self {
        Self {
            rotation: Rotation::None,
            mirror: Mirror::None,
        }
    }

    pub fn transform(mut self, transform: Transformation) -> Self {
        self.mirror = match (self.mirror, transform.mirror) {
            (Mirror::Horizontal, Mirror::Vertical) | (Mirror::Vertical, Mirror::Horizontal) => {
                self.rotate(Rotation::OneEighty);
                self.mirror
            }
            (Mirror::None, m) | (m, Mirror::None) => m,
            (Mirror::Horizontal, Mirror::Horizontal) | (Mirror::Vertical, Mirror::Vertical) => {
                Mirror::None
            }
        };

        self.rotate(transform.rotation);
        self
    }

    pub const fn apply(&self, point: Point) -> Point {
        self.mirror.apply(self.rotation.apply(point))
    }
    pub const fn types() -> [Transformation; 12] {
        use self::{Mirror::*, Rotation::*};
        [
            Transformation {
                rotation: Rotation::None,
                mirror: Mirror::None,
            },
            Transformation {
                rotation: Rotation::None,
                mirror: Horizontal,
            },
            Transformation {
                rotation: Rotation::None,
                mirror: Vertical,
            },
            Transformation {
                rotation: Ninety,
                mirror: Mirror::None,
            },
            Transformation {
                rotation: Ninety,
                mirror: Horizontal,
            },
            Transformation {
                rotation: Ninety,
                mirror: Vertical,
            },
            Transformation {
                rotation: OneEighty,
                mirror: Mirror::None,
            },
            Transformation {
                rotation: OneEighty,
                mirror: Horizontal,
            },
            Transformation {
                rotation: OneEighty,
                mirror: Vertical,
            },
            Transformation {
                rotation: TwoSeventy,
                mirror: Mirror::None,
            },
            Transformation {
                rotation: TwoSeventy,
                mirror: Horizontal,
            },
            Transformation {
                rotation: TwoSeventy,
                mirror: Vertical,
            },
        ]
    }
}

#[test]
fn unique_rotations() {
    let variants = Transformation::types();
    let moves = vec![
        Point::new(0, 0),
        Point::new(8, 7),
        Point::new(0, 14),
        Point::new(14, 0),
        Point::new(14, 3),
        Point::new(14, 14),
    ];

    for variant in variants.iter() {
        for other in variants.iter().filter(|v| v != &variant) {
            assert_ne!(
                moves
                    .clone()
                    .into_iter()
                    .map(|p| variant.apply(p))
                    .collect::<Vec<_>>(),
                moves
                    .clone()
                    .into_iter()
                    .map(|p| other.apply(p))
                    .collect::<Vec<_>>()
            );
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Rotation {
    None,
    Ninety,
    OneEighty,
    TwoSeventy,
}

impl Rotation {
    pub const fn apply(&self, p: Point) -> Point {
        // Assumes grid of 15x15
        match self {
            Rotation::None => p,
            Rotation::Ninety => Point::new(p.y, 14 - p.x),
            Rotation::OneEighty => Point::new(14 - p.x, 14 - p.y),
            Rotation::TwoSeventy => Point::new(14 - p.y, p.x),
        }
    }

    pub const fn rotations() -> &'static [Rotation] {
        &[
            Rotation::None,
            Rotation::Ninety,
            Rotation::OneEighty,
            Rotation::TwoSeventy,
        ]
    }
    pub fn rotate(&mut self, rotation: Rotation) {
        *self = *Rotation::rotations()
            .iter()
            .cycle()
            .skip_while(|r| *r != &*self)
            .nth(match rotation {
                Rotation::None => return,
                Rotation::Ninety => 1,
                Rotation::OneEighty => 2,
                Rotation::TwoSeventy => 3,
            })
            .unwrap();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Mirror {
    None,
    Horizontal,
    Vertical,
}

impl Mirror {
    pub const fn apply(&self, p: Point) -> Point {
        // Assumes grid of 15x15
        match self {
            Mirror::None => p,
            Mirror::Horizontal => Point::new(14 - p.x, p.y),
            Mirror::Vertical => Point::new(p.x, 14 - p.y),
        }
    }

    pub const fn mirrors() -> &'static [Mirror] {
        &[Mirror::None, Mirror::Horizontal, Mirror::Vertical]
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
    let a_1 = graph.insert_move(a, b_1.clone());
    let b_2 = BoardMarker::new(Point::new(9, 7), Stone::Black);
    let _a_2 = graph.insert_move(a, b_2);
    let b_1_1 = BoardMarker::new(Point::new(10, 7), Stone::White);
    let a_1_1 = graph.insert_move(a_1, b_1_1);
    let b_1_2 = BoardMarker::new(Point::new(11, 7), Stone::Black);
    let a_1_2 = graph.insert_move(a_1, b_1_2);
    let b_1_2_1 = BoardMarker::new(Point::new(12, 7), Stone::White);
    let a_1_2_1 = graph.insert_move(a_1_2, b_1_2_1);
    let b_1_2_1_1 = BoardMarker::new(Point::new(8, 4), Stone::Black);
    let _a_1_2_1_1 = graph.insert_move(a_1_2_1, b_1_2_1_1);
    let b_1_2_1_2 = BoardMarker::new(Point::new(7, 4), Stone::Black);
    let a_1_2_1_2 = graph.insert_move(a_1_2_1, b_1_2_1_2);
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
        graph.as_board(&a_1_2_1_2).unwrap().0
    );
    // let branched_up = graph.up_to_branch()
    //NOTE:FIXME:TODO: Add asserts!!
}
