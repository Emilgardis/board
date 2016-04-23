#![feature(io)]
use std::io;
use std::str;
use std::io::prelude::*;
use std::path::{Path};
use std::fs::{File};
use std::error::Error;
use std::slice::IterMut;

use board_logic::{BoardMarker, Stone, Point};
use move_node::{MoveGraph, MoveIndex};

/// Describes the file
pub enum FileType{
    /// Generic Renju _.pos_ file.
    ///
    /// These files seems to always assume a field of size 15*15
    /// # Layout in binary
    /// * 0: N moves
    /// * 1: Black move #1
    /// * 2: White move #2
    /// * N: Last move
    /// 
    Pos,
    /// RenLib, _.lib_ file.
    ///
    /// This is not supported yet as I have not fully reversed the protocol yet.
    /// 
    /// # About
    ///
    /// _.lib_ files are most used for it's excellent support for saving multiple games of one play. This enables 
    /// a way to do analysis. 
    ///
    /// #### Example
    /// The move sequence is H8 I9 G7 H7 D11 etc. We want to analyse and document what would
    /// have happened if we played H6 instead of H7 on move #4. So we would go back to move #3 and continue from there. Now,
    /// when walking through the game from the beginning, we will see that there are 2 choices 
    /// highlighted on move #4, we have created a _sub game_.
    /// Clicking on either will continue this pattern of finding sub-games like before, repeating itself if needed. If
    /// you don't choose any of those choices it makes a new sub-game.
    /// 
    /// This is why RenLib's _.lib_ format is so praised in the renju community.
    ///
    /// ...* _.lib_ files come in two variations, either as Libraries or as Positions. I have yet
    /// to identify what the signature is for both.
    ///
    /// ## Known:
    /// * Libraries are stored as such: HEADER n * [POS:FLAG:EXTENDEDINFO]. Since **.lib** supports
    /// trees, we had to implement it [in rust too](#move_node::MoveGraph)
    /// Positions are stored in one byte. This means that 0x78 is the middle.
    ///
    ///     This is the layout for X, Y:
    ///     
    ///          0: . . . . . . . . . . . . . . .  
    ///          1: . . . . . . . . . . . . . . . 
    ///          2: . . . . . . . . . . . . . . . 
    ///          3: . . . . . . . . . . . . . . . 
    ///          4: . . . O . . . . . . . . . . . 
    ///          5: . . . . . . . . . . . . . . . 
    ///          6: . . . . . . . . . . . . . . . 
    ///          7: . . . . . . . X . . . . . . . 
    ///          8: . . . . . . . . . . . . . . . 
    ///          9: . . . . . . . . . . . . . . . 
    ///          A: . . . . . . . . . . . . . . . 
    ///          B: . . . . . . . . . . . . . . . 
    ///          C: . . . . . . . . . . . . . . . 
    ///          D: . . . . . . . . . . . . . . . 
    ///          E: . . . . . . . . . . . . . . . 
    ///             1 2 3 4 5 6 7 8 9 A B C D E F  
    ///
    /// The _O_ is on `0x44`, the _X_ is on `0x78` (the middle)
    /// 
    /// * The header consists of 20 bytes:
    ///         0xFF,  'R',  'e',  'n',  'L',  'i',  'b', 0xFF,
    ///         MAJOR_VERSION, MINOR_VERSION, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    ///         0xFF, 0xFF, 0xFF, 0xFF
    ///
    /// This can be shown with the command `xxd -g 1 -c8 <.lib file>`
    ///
    /// Note that the minor version doesn't reflect the software version, it stands for the
    /// files "protocol" version, i.e it is updated only on a breaking change.
    /// `RenLib v37` has major 3, minor 0. (It has support for 3.4 but files seem to never get
    /// stored as such.)
    ///
    ///     
    /// * FIXME! 
    /// These are set in RenLib/MoveNode.cpp
    ///
    ///         const DOWN        = 0x000080;
    ///         const RIGHT       = 0x000040;
    ///         const OLD_COMMENT = 0x000020;
    ///         const MARK        = 0x000010;
    ///         const COMMENT     = 0x000008;
    ///         const START       = 0x000004;
    ///         const NO_MOVE     = 0x000002;
    ///         const EXTENSION   = 0x000001;
    ///         const MASK        = 0xFFFF3F;
    ///
    ///
    ///
    /// *    See RenLib/RenLibDoc.cpp for implementation.
    Lib,
}

impl FileType{
    fn new(path: &Path) -> Option<FileType> {
        match path.extension() {
            Some(pos) if (pos == "pos") => Some(FileType::Pos),
            Some(lib) if (lib == "lib") => Some(FileType::Lib),
            Some(_) => None,
            None => None,
        }
    }
}

#[derive(Debug)]
pub enum FileErr {
    OpenError,
    ParseError,
    NotSupported,
}

pub fn open_file(path: &Path) -> Result<MoveGraph, FileErr> {
    let display = path.display();
    let filetype = FileType::new(path);
    let mut file: File = match File::open(&path) {
        Ok(file) => file,
        Err(desc) => return Err(FileErr::OpenError),
    };

    match filetype { 
        Some(FileType::Pos) => {
            let mut sequence: Vec<BoardMarker> = Vec::new();
            for (index, pos) in file.bytes().skip(1).enumerate() { // First value should always be the number of moves.
                sequence.push(BoardMarker::new(Point::from_1d(pos.ok().unwrap() as u32, 15), if index % 2 == 0 {Stone::Black} else {Stone::White})); // was pos.ok().unwrap()
            }
            let mut root = MoveGraph::new();
            let mut latest: MoveIndex = root.new_root(sequence[0]);
            for marker_move in sequence.into_iter().skip(1) {
                latest = root.add_move(latest, marker_move)
            }
            return Ok(root);
        },
        Some(FileType::Lib) => {
            let mut file_u8: Vec<u8> = Vec::new();
                //::with_capacity(match file.metadata() { Ok(meta) => meta.len() as usize, Err(err) => return Err(FileErr::OpenError)});
            for byte in file.bytes() {
                match byte {
                    Ok(val) => file_u8.push(val),
                    Err(err) => { println!("{:?}", err); return Err(FileErr::OpenError);},
                }
            }
            let header: Vec<u8> = file_u8.drain(0..20).collect();
            //let Game = unimplemented!();
            let major_file_version = header[8] as u32;
            let minor_file_version = header[9] as u32;
            

            // Here we will want to do everything that is needed.
            // First value is "always" the starting position.
            //
            let mut graph: MoveGraph = MoveGraph::new();
            // A stack like thing, 0x80 pushes, 0x40 removes.
            let mut branches: Vec<MoveIndex> = vec![];
            // The children of the last entry in branches. First one is the "parent".
            let mut children: Vec<MoveIndex> = vec![];
            // If 0x00, then we are adding to branches.
            let mut current_command: u8 = 0x00;
            println!("{:?}", file_u8);
            let mut command_iter = file_u8.into_iter().peekable().clone();
            let mut moves: u32 = 1;
            let mut subtrees: i32 = 1; // If 0x80, increment, if 0x40, decrement, if this is 0, switch tree.
            while command_iter.peek().is_some() {
                let byte: u8 = match command_iter.next(){
                    Some(val) => val,
                    None => panic!("Fail here!"),
                };
                println!("Current byte: 0x{:x}, current_command: 0x{:x}", byte, current_command);
                if current_command & 0x02 != 0x02 { // 0x02 is no_move.
                    if moves > 1 { // last returns a Option<&T>
                        println!("Checking with: \n\tChildren: {:?}, branches: {:?}", children, branches);
                        moves += 1;
                        let last_child: MoveIndex = match children.last() {
                            Some(val) => val.clone(),
                            None => branches.last().unwrap().clone(),
                            };
                        children.push(graph.add_move(last_child,
                                BoardMarker::new(
                                    Point::new(((byte & 0x0F)-1) as u32, (byte >> 4) as u32),
                                if moves % 2 == 0 {Stone::Black} else {Stone::White})
                        ));
                        println!("\tAdded {:?} to children", children.last().unwrap());
                        current_command = command_iter.next().unwrap(); // FIXME: Could be none
                    } else { // We are in as root! HACKER!
                        println!("In root, should be empty: \n\tChildren: {:?}, branches: {:?}", children, branches);
                        if byte == 0x00 {
                            return Err(FileErr::ParseError); // Does not currently support multiple start positions.
                        }
                        moves += 1;
                        let move_ind: MoveIndex = graph.new_root(
                                BoardMarker::new(
                                    Point::new(((byte & 0x0F)-1) as u32, (byte >> 4) as u32),
                                Stone::Black));
                        children.push(move_ind);
                        current_command = command_iter.next().unwrap();
                    }
                }
                println!("\tand now 0x{:02x}", current_command);
                if current_command & 0x80 == 0x80 { // if we are saying: This node has siblings!.
                    let children_len = children.len();
                    subtrees += 1;
                    if branches.last() != children.get(0) {
                        let branched_child = children.get(children_len-2).unwrap().clone();
                        println!("Entering subtree! {:?}", branched_child);
                        branches.push(branched_child); // Add the node that is a parent of multiple nodes.
                        children = vec![branched_child];
                        println!("\tChildren: {:?}, branches: {:?}, subtree: {}", children, branches, subtrees);
                    } else {
                        // We are already in this sub-tree! Yeah!
                        branches.push(children[children_len-2]);
                        children = vec![children[children_len-2]];
                        println!("New subtree, adding second last child to branches.\n\tChildren: {:?}, branches: {:?}, subtree: {}", children, branches, subtrees);
                    }
                    //NOTE:current_command = command_iter.next().unwrap();
                }
                if current_command & 0x40 == 0x40 { // This branch is done, return down.
                    subtrees -= 1;
                    if subtrees < 0 {
                        branches.pop(); // Should be used when this supports multiple starts.
                    }
                    children = match branches.last() { Some(val) => vec![val.clone()], None => vec![]};
                    println!("exiting subtree,{}poping branches, adding new tree.\n\tChildren: {:?}, branches: {:?}",if subtrees == 0 {" "} else {" not "}, children, branches);
                }
                subtrees = 1;
                if current_command & 0x08 == 0x08 {
                    //let cloned_cmd_iter = command_iter.clone();
                    let mut title: Vec<u8> = Vec::new();
                        //cloned_cmd_iter.take_while(|x| *x != 0x08).collect();
                    let mut comment: Vec<u8> = Vec::new();
                        //cloned_cmd_iter.clone().take_while(|x| *x != 0x08).collect();
                    // TODO: Consider using
                    // http://bluss.github.io/rust-itertools/doc/itertools/trait.Itertools.html#method.take_while_ref
                    while *command_iter.peek().unwrap() != 0x08 {
                        title.push(command_iter.next().unwrap());
                    }
                    while *command_iter.peek().unwrap() != 0x00 {
                        comment.push(command_iter.next().unwrap());
                    }
                    command_iter.next(); // Skip the zero.

                    println!("\tTitle: {}, Comment: {}", str::from_utf8(&title).unwrap_or("Failed to parse!"), str::from_utf8(&comment).unwrap_or("Failed to parse!"));
                    //command_iter.skip(title.len() + comment.len() +2);
                }
            }
            Ok(graph)
        },
        _ => unimplemented!(),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use board_logic as bl;
    use move_node as mn;

    #[test]
    fn open_pos_file(){
        let file = Path::new("examplefiles/example.pos");
        let mut graph: mn::MoveGraph = match open_file(file) {
            Ok(gr) => gr,
            Err(desc) => panic!("{:?}", desc),
        };
        println!("\n{:?}", graph);
    }
    #[test]
    fn open_lib_file(){
        let file = Path::new("examplefiles/lib_documented.lib");
        let mut graph: mn::MoveGraph = match open_file(file) {
            Ok(gr) => gr,
            Err(desc) => panic!("err, {:?}", desc),
        };
        println!("\n{:?}", graph);
        //panic!("Intended!");
    }
}
