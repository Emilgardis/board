use board_logic::BoardMarker;
use daggy;
use daggy::Walker;
use std::fmt;
pub type NodeIndex = daggy::NodeIndex<usize>;
pub type EdgeIndex = daggy::EdgeIndex<usize>;

//pub type MoveGraph = daggy::Dag<NodeIndex, EdgeIndex>;

#[derive(Clone, Copy)]
pub struct MoveIndex {
    node_index: NodeIndex,
    edge_index: Option<EdgeIndex>,
}

impl MoveIndex { 
    pub fn new(edge_node: (EdgeIndex, NodeIndex)) -> MoveIndex {
        MoveIndex { node_index: edge_node.1, edge_index: Some(edge_node.0) }
    }

    pub fn new_node(node: NodeIndex) -> MoveIndex {
        MoveIndex { node_index: node, edge_index: None }
    }

    pub fn from_option(edge_node_option: Option<(EdgeIndex, NodeIndex)>) -> Option<MoveIndex> {
        match edge_node_option {
            Some(edge_node) => Some(MoveIndex { node_index: edge_node.1, edge_index: Some(edge_node.0) }),
            None => None,
        }
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
pub struct MoveGraph {
    graph: daggy::Dag<BoardMarker, usize, usize>,
}


impl MoveGraph {
    pub fn new() -> MoveGraph {
        MoveGraph {
            graph: daggy::Dag::with_capacity(255, 255)
        }
    }

    pub fn new_root(&mut self, marker: BoardMarker) -> MoveIndex {
        MoveIndex::new_node(self.graph.add_node(marker))
    }
    pub fn add_move(&mut self, parent: MoveIndex, marker: BoardMarker) -> MoveIndex {
        MoveIndex::new(self.graph.add_child(parent.node_index, 0, marker))
    }
    pub fn  get_marker(&self, node: MoveIndex) -> Option<&BoardMarker> {
        self.graph.node_weight(node.node_index)
    }
    pub fn get_mut_move(&mut self, node: MoveIndex) -> Option<&mut BoardMarker> {
        self.graph.node_weight_mut(node.node_index)
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

    pub fn get_parent(&self, child: MoveIndex) -> Option<MoveIndex> {
        let mut peek_parent = self.graph.parents(child.node_index).peekable();
        let result = peek_parent.next(&self.graph);
        if peek_parent.peek(&self.graph) != None {
            panic!("Error, shame on me!") //FIXME: This error message sucks.
        } else {
            MoveIndex::from_option(result)
        }
    }
    // Convenience methods, like set comment, set pos etc. Also walk down node until multiple
    // choices. etc.
    pub fn down_to_branch(&self, node: MoveIndex) -> (Vec<MoveIndex>, Option<Vec<MoveIndex>>) {
        let mut branch_ancestors: Vec<MoveIndex> = Vec::new();
        let mut children = self.get_children(node);
        while children.len() == 1 {
            println!("{:?}", children);
            branch_ancestors.push(children[0].clone());
            children = self.get_children(children[0]);
        }
        (branch_ancestors, if children.len() == 0 {None} else {Some(children)})
    }
}

impl fmt::Debug for MoveGraph {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", daggy::petgraph::dot::Dot::with_config(self.graph.graph(), &[/*daggy::petgraph::dot::Config::EdgeIndexLabel,daggy::petgraph::dot::Config::NodeIndexLabel*/]));
        Ok(())
    }
}

#[test]
fn does_it_work() {
    use board_logic::*;
    let mut graph = MoveGraph::new();
    let a = graph.new_root(BoardMarker::new(Point::new(7,7), Stone::Black));
    let b_1 = BoardMarker::new(Point::new(8,7), Stone::White);
    let a_1 = graph.add_move(a, b_1);
    let b_2 = BoardMarker::new(Point::new(9,7), Stone::Black);
    let a_2 = graph.add_move(a, b_2);
    let b_1_1 = BoardMarker::new(Point::new(10,7), Stone::White);
    let a_1_1 = graph.add_move(a_1, b_1_1);
    let b_1_2 = BoardMarker::new(Point::new(11,7), Stone::Black);
    let a_1_2 = graph.add_move(a_1, b_1_2);
    let b_1_2_1 = BoardMarker::new(Point::new(12,7), Stone::White);
    let a_1_2_1 = graph.add_move(a_1_2, b_1_2_1);
    let b_1_2_1_1 = BoardMarker::new(Point::new(8, 4), Stone::Black);
    let a_1_2_1_1 = graph.add_move(a_1_2_1, b_1_2_1_1);
    let b_1_2_1_2 = BoardMarker::new(Point::new(7,4), Stone::Black);
    let a_1_2_1_2 = graph.add_move(a_1_2_1, b_1_2_1_2);
    
    // for i in 
    println!("{:?}", graph);
    println!("Children of {:?} {:?}", b_1, graph.get_children(a_1));
    let branched = graph.down_to_branch(a_1_2);;
    println!("Moving down on {:?} gives: end = {:?}, remaining = {:?}", a_1_2, branched.0, branched.1);
    panic!("Hello!");
}
