//! Used for reading files.
//!
//! Currently only supports _.pos_ and _.lib_ (RenLib) files of version 3.04+.
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
    /// * Libraries are stored as such: HEADER n * [POS:FLAGS:STRINGS:EXTENDEDINFO]. Since **.lib** supports
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
    // * FIXME! 
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
    /// Renju Database File
    ///
    /// These are generally quite large. They include multiple games, so these will really test my
    /// implementation of trees. They need support for findig comments as this is the way games are
    /// found.
    Rif,
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
        Err(err) => {error!("Failed opening file. {:?}", err); return Err(FileErr::OpenError)},
    };

    match filetype { 
        Some(FileType::Pos) => {
            debug!("Opening pos file. {:?}", path);
            let mut sequence: Vec<BoardMarker> = Vec::new();
            for (index, pos) in file.bytes().skip(1).enumerate() { // First value should always be the number of moves.
                sequence.push(BoardMarker::new(Point::from_1d((match pos {
                    Ok(val) => val,
                    Err(err) => {error!("Failed reading file: {:?}", err); return Err(FileErr::ParseError)},
                }) as u32, 15), if index % 2 == 0 {Stone::Black} else {Stone::White}));
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
                    Err(err) => {error!("Failed reading file: {:?}", err); return Err(FileErr::ParseError)},
                }
            }
            let header: Vec<u8> = file_u8.drain(0..20).collect();
            //let Game = unimplemented!();
            let major_file_version = header[8] as u32;
            let minor_file_version = header[9] as u32;
            println!("Opened RenLib file, v.{}.{:>02}", major_file_version, minor_file_version);            

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
            //debug!("{:?}", file_u8);
            let mut command_iter = file_u8.into_iter().peekable().clone();
            let mut moves: u32 = 1;
            let mut multiple_start: u32 = 0;
            'main: while command_iter.peek().is_some() {
                let byte: u8 = match command_iter.next(){
                    Some(val) => val,
                    None => {error!("This shoudln't have happened. Error on reading command_iter.next()!", ); return Err(FileErr::ParseError)},
                };
                                        debug!("\t\tbyte: {:x}", byte);
                println!("Current byte: 0x{:x}, current_command: 0x{:x}", byte, current_command);
                if current_command & 0x02 != 0x02 { // 0x02 is no_move.
                    if moves > 1 { // last returns a Option<&T>
                        debug!("Checking with: \n\tChildren: {:?}, branches: {:?}", children, branches);
                        moves += 1;
                        let last_child: MoveIndex = match children.last() {
                            Some(val) => val.clone(),
                            None => match branches.last() { Some(last) => last.clone(), None => { error!("Failed reading branches.last()"); return Err(FileErr::ParseError)}},
                            };
                        debug!("\tAdded to {:?}.", last_child);
                        children.push(graph.add_move(last_child,
                                                     if byte != 0x00 {
                                BoardMarker::new(
                                    Point::new((match byte.checked_sub(1) { Some(value) => value, None => return Err(FileErr::ParseError)} & 0x0f) as u32, (byte >> 4) as u32),
                                if moves % 2 == 0 {Stone::Black} else {Stone::White}) } else {BoardMarker::new(Point::from_1d(5, 2), Stone::Empty)}
                        ));
                        debug!("\tAdded {:?}:{:?} to children", match children.last() {Some(last) => graph.get_move(*last).unwrap(), None=> return Err(FileErr::ParseError)}, children.last() );
                        current_command = match command_iter.next() {Some(command) => command, None=> return Err(FileErr::ParseError)};
                    } else { // We are in as root! HACKER!
                        debug!("In root, should be empty: \n\tChildren: {:?}, branches: {:?}", children, branches);
                        if byte == 0x00 {
                            // we do not really care, we always support these files.
                            debug!("Skipped {:?}", command_iter.next());

                            continue 'main;
                            //return {error!("Tried opeing a no-move start file."); Err(FileErr::ParseError)}; // Does not currently support these types of files.
                        }
                        moves += 1;
                        if children.len() > 0 {
                            let move_ind: MoveIndex = graph.add_move(
                                *children.last().unwrap(),
                                BoardMarker::new(
                                    Point::new((byte-1 & 0x0f) as u32, (byte >> 4) as u32),
                                    if moves % 2 == 0 {Stone::Black} else {Stone::White}),
                                );
                            children.push(move_ind);
                        } else { 
                            let move_ind: MoveIndex = graph.new_root(
                                BoardMarker::new(
                                    Point::new((byte-1 & 0x0f) as u32, (byte >> 4) as u32),
                                Stone::Black));
                            children.push(move_ind);
                        }
                        current_command = match command_iter.next() {Some(command) => command, None=> return Err(FileErr::ParseError)};
                        if current_command & 0x80 == 0x80 {
                            // Multiple start
                            multiple_start = 1;     
                        }
                    }
                }
                println!("\tand now 0x{:02x}", current_command);
                if current_command & 0x80 == 0x80 { // if we are saying: This node has siblings!.
                    let children_len = children.len();
                    if children_len < 2 { println!("Children that error me! {:?}", children); return Err(FileErr::ParseError)}
                    let lost_child = match children.last() {Some(last) => last.clone(), None=>{ error!("Failed reading children.last()!"); return Err(FileErr::ParseError)}} ; // Not sure if need clone.
                    branches.push(children[children_len-2]);
                    children = vec![children[children_len-2]];
                    children.push(lost_child);
                    debug!("New subtree, adding second last child to branches.\n\tChildren: {:?}, branches: {:?}", children, branches);
                }
                if current_command & 0x40 == 0x40 { // This branch is done, return down.
                    children = match branches.last() { Some(val) => vec![val.clone()], None => vec![]};
                    if branches.len() > 1 && multiple_start == 1 {
                        branches.pop(); // Should be used when this supports multiple starts.
                    } else {
                        multiple_start = 0;
                    }
                    moves = match children.get(0) {Some(child) => 1 + graph.down_to_root(*child).len() as u32, None => 0};
                    debug!("back to subtree root, poping branches.\n\tChildren: {:?}, branches: {:?}",  children, branches);
                }

                if current_command & 0x08 == 0x08 {
                    //let cloned_cmd_iter = command_iter.clone();
                    let mut title: Vec<u8> = Vec::new();
                        //cloned_cmd_iter.take_while(|x| *x != 0x08).collect();
                    let mut comment: Vec<u8> = Vec::new();
                        //cloned_cmd_iter.clone().take_while(|x| *x != 0x08).collect();
                    // TODO: Consider using
                    // http://bluss.github.io/rust-itertools/doc/itertools/trait.Itertools.html#method.take_while_ref
                    while match command_iter.peek() {Some(command) => *command, None => return{ error!("Failed reading file while reading title!"); Err(FileErr::ParseError)}} != 0x08 {
                        title.push(command_iter.next().unwrap()); // This should be safe.
                    }
                    while match command_iter.peek() {Some(command) => *command, None => return{ error!("Failed reading file while reading comment!"); Err(FileErr::ParseError)}} != 0x00 {
                        comment.push(command_iter.next().unwrap()); // This should be safe.
                    }
                    command_iter.next(); // Skip the zero.

                    debug!("\tTitle: {}, Comment: {}", str::from_utf8(&title).unwrap_or("Failed to parse title!"), str::from_utf8(&comment).unwrap_or("Failed to parse comment!"));
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
        debug!("\n{:?}", graph);
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
