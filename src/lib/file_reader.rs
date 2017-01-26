//! Used for reading files.
//!
//! Currently only supports _.pos_ and _.lib_ (RenLib) files of version 3.04+.
#![feature(io)]
use std::io;
use std::str;
use std::io::prelude::*;
use std::path::Path;
use std::fs::File;
use std::error::Error;
use std::slice::IterMut;

use board_logic::{BoardMarker, Stone, Point};
use move_node::{MoveGraph, MoveIndex};
use errors::*;

/// Describes the file
pub enum FileType {
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
    ///         0xFF,  'R',  'e',  'n',  'L',  'i',  'b', 0xFF, MAJV, MINV, 
    ///         0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
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

impl FileType {
    fn new(path: &Path) -> Option<FileType> {
        match path.extension() {
            Some(pos) if (pos == "pos") => Some(FileType::Pos),
            Some(lib) if (lib == "lib") => Some(FileType::Lib),
            Some(_) => None,
            None => None,
        }
    }
}

pub enum FileErr {
    ParseError,
}

pub fn open_file(path: &Path) -> Result<MoveGraph> {
    let display = path.display();
    let filetype = FileType::new(path);
    let mut file: File = File::open(&path)?;

    match filetype { 
        Some(FileType::Pos) => {
            println!("Opening pos file. {:?}", path);
            let mut sequence: Vec<BoardMarker> = Vec::new();
            for (index, pos) in file.bytes().skip(1).enumerate() {
                // First value should always be the number of moves.
                sequence.push(BoardMarker::new(
                        Point::from_1d(
                            pos.chain_err(|| "Couldn't get position").chain_err(|| ErrorKind::PosParseError)? as u32, 15),
                        if index % 2 == 0 {Stone::Black} else {Stone::White}));
            }
            let mut root = MoveGraph::new();
            let mut latest: MoveIndex = root.new_root(sequence[0].clone());
            for marker_move in sequence.into_iter().skip(1) {
                latest = root.add_move(latest, marker_move)
            }
            Ok(root)
        }
        Some(FileType::Lib) => {
            let mut file_u8: Vec<u8> = Vec::new();
            for byte in file.bytes() {
                file_u8.push(byte.chain_err(|| "while loading file")?)
            }
            parse_lib(file_u8).chain_err(|| ErrorKind::LibParseError)
        }
        _ => Err(ErrorKind::NotSupported.into()),
    }
}

pub fn parse_lib(file_u8: Vec<u8>) -> Result<MoveGraph> {
    let mut file_u8 = file_u8;
    //::with_capacity(match file.metadata() { Ok(meta) => meta.len() as usize, Err(err) => return Err(FileErr::OpenError)});
    let header: Vec<u8> = file_u8.drain(0..20).collect();
    // let Game = unimplemented!();
    let major_file_version = header[8] as u32;
    let minor_file_version = header[9] as u32;
    println!("Opened RenLib file, v.{}.{:>02}",
             major_file_version,
             minor_file_version);

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
    // println!("{:?}", file_u8);
    let mut command_iter = file_u8.into_iter().peekable().clone();
    let mut moves: u32 = 1;
    let mut multiple_start: u32 = 0;
    'main: while command_iter.peek().is_some() {
        let byte: u8 = command_iter.next().unwrap(); // This shouldn't fail.
        println!("\t\tbyte: {:x}", byte);
        println!("Current byte: 0x{:0>2x}, current_command: 0x{:x}",
                 byte,
                 current_command);
        if current_command & 0x02 != 0x02 {
            // 0x02 is no_move.
            if moves > 1 {
                // last returns a Option<&T>
                println!("Checking with: \n\tChildren: {:?}, branches: {:?}",
                       children,
                       branches);
                moves += 1;
                let last_child: MoveIndex = match children.last() {
                    Some(val) => {
                        println!("adding move to child:{:?}", val);
                        val.clone()
                    },
                    None => {
                        let val = *branches.last().ok_or("Failed reading branches.last()")?;
                        println!("adding move to last branch:{:?}", val);
                        val
                    },
                };

                //println!("\tAdded to {:?}.", last_child);
                children.push(graph.add_move(last_child,
                    if byte != 0x00 {
                        BoardMarker::new(
                            Point::new(
                                (match byte.checked_sub(1) {
                                    Some(value) => value,
                                    None => return Err("Underflowed position".into())
                                } & 0x0f) as u32,
                                (byte >> 4) as u32),
                        if moves % 2 == 0 {
                            Stone::Black
                        } else {
                            Stone::White
                        })
                    } else {
                        BoardMarker::new(Point::from_1d(5, 2), Stone::Empty)
                    }
                ));
                println!("Added {:?}:{:?} to children: {:?}", match children.last() {
                        Some(last) => graph.get_move(*last).unwrap(),
                        None => return Err("Couldn't get last child".into()),
                    },
                    children.last().unwrap(), children,
                );
                println!("Stepping forward in command.");
                current_command = match command_iter.next() {
                    Some(command) => command,
                    None => return Err("Uanble to get next command".into()),
                };

            } else {
                // We are in as root! HACKER!
                println!("In root, should be empty: \n\tChildren: {:?}, branches: {:?}",
                       children,
                       branches);
                if byte == 0x00 {
                    // we do not really care, we always support these files.
                    println!("Skipped {:?}", command_iter.next());

                    continue 'main;
                    // return {error!("Tried opeing a no-move start file."); Err(FileErr::ParseError)}; // Does not currently support these types of files.
                }
                moves += 1;
                if children.len() > 0 {
                    let move_ind: MoveIndex = graph.add_move(*children.last().unwrap(),
                                  BoardMarker::new(Point::new((byte - 1 & 0x0f) as u32,
                                                              (byte >> 4) as u32),
                                                   if moves % 2 == 0 {
                                                       Stone::Black
                                                   } else {
                                                       Stone::White
                                                   }));
                    children.push(move_ind);
                } else {
                    let move_ind: MoveIndex =
                        graph.new_root(BoardMarker::new(Point::new((byte - 1 & 0x0f) as u32,
                                                                   (byte >> 4) as u32),
                                                        Stone::Black));
                    children.push(move_ind);
                }
                current_command = match command_iter.next() {
                    Some(command) => command,
                    None =>  {
                        if command_iter.peek().is_some() {
                            return Err("No command is next".into())
                        }
                        break 'main;
                    }
                };
                if current_command & 0x80 == 0x80 {
                    // Multiple start, this may be wrong.
                    multiple_start = 1;
                }
            }
        }
        println!("New command now 0x{:02x}", current_command);
        if current_command & 0x40 == 0x40 {
            // This branch is done, return down.
            println!("Branching down");
            let lost_child = if current_command & 0x80 == 0x80 {
                // We need to add the current branch to branches, not sure what this actually is...
                // I believe this is when only one stone is on the upcomming branch.
                println!("Uncertain about 0xc0, add second last child to branches.");
                println!("Branches: {:?}, children: {:?}", branches, children);
                let children_len = children.len();
                Some(children[children_len-2])
            } else {
                None
            };
            children = match branches.last() {
                Some(val) => vec![val.clone()],
                None => vec![],
            };
            children.extend(lost_child);
            if branches.len() > 1 {
                branches.pop(); // Should be used when this supports multiple starts.
            }
            moves = match children.get(0) {
                Some(child) => 1 + graph.down_to_root(*child).len() as u32,
                None => 1,
            };
            println!("back to subtree root, poping branches.\n\tChildren: {:?}, branches: \
                    {:?}. Moves: {}",
                   children,
                   branches, moves);
        }
        if current_command & 0x80 == 0x80 {
            // if we are saying: This node has siblings!.
            // This means that the children are 
            // TODO: A sibling can be first move.
            // NOTE: If we are both 0x80 and 0x40 what happens?
            // I believe 0x40 should be checked first.
            println!("We have some siblings. Add my parent to branches and replace with children");
            let children_len = children.len();
            if children_len <= 2 { // Not sure why.
                //println!("Children that error me! {:?}", children);
                //return Err(ErrorKind::LibParseError.into());
                // FIXME: May be wrong.
                if moves < 2 {
                    multiple_start = 1;
                
                    continue 'main;
                }
                
            }
            // The one we just pushed has siblings, that means the branch is on the parent.
            let parent = children[children_len - 2];
            let child = children[children_len -1];
            println!("Parent is {:?}", parent);
            branches.push(parent);
            children = vec![parent];
            children.push(child);

            // OLD CODE: May be wrong or right.
            //
            //let lost_child = match children.last() {
            //    Some(last) => last.clone(),
            //    None => {
            //        error!("Failed reading children.last()!");
            //        return Err(ErrorKind::LibParseError.into());
            //    }
            //}; // Not sure if need clone.
            //branches.push(children[children_len - 2]);
            //children = vec![children[children_len - 2]];
            //println!("--NEW-- Children: {:?}", children);
            //children.push(lost_child);
            println!("New subtree, adding last child to branches.\n\tChildren: \
                    {:?}, branches: {:?}",
                   children,
                   branches);
        }

        if current_command & 0x08 == 0x08 {
            // let cloned_cmd_iter = command_iter.clone();
            let mut title: Vec<u8> = Vec::new();
            // cloned_cmd_iter.take_while(|x| *x != 0x08).collect();
            let mut comment: Vec<u8> = Vec::new();
            // cloned_cmd_iter.clone().take_while(|x| *x != 0x08).collect();
            // TODO: Consider using
            // http://bluss.github.io/rust-itertools/doc/itertools/trait.Itertools.html#method.take_while_ref
            while match command_iter.peek() {
                Some(command) => *command,
                None => {
                    return {
                        error!("Failed reading file while reading title!");
                        Err(ErrorKind::LibParseError.into())
                    }
                }
            } != 0x08 {
                title.push(command_iter.next().unwrap()); // This should be safe.
            }
            while match command_iter.peek() {
                Some(command) => *command,
                None => {
                    return {
                        error!("Failed reading file while reading comment!");
                        Err(ErrorKind::LibParseError.into())
                    }
                }
            } != 0x00 {
                comment.push(command_iter.next().unwrap()); // This should be safe.
            }
            command_iter.next(); // Skip the zero.

            println!("\tTitle: {}, Comment: {}",
                   str::from_utf8(&title).unwrap_or("Failed to parse title!"),
                   str::from_utf8(&comment).unwrap_or("Failed to parse comment!"));
            // command_iter.skip(title.len() + comment.len() +2);
        }
    }
    Ok(graph)

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use board_logic as bl;
    use move_node as mn;

    #[test]
    fn open_pos_file() {
        let file = Path::new("examplefiles/example.pos");
        let mut graph: mn::MoveGraph = match open_file(file) {
            Ok(gr) => gr,
            Err(desc) => panic!("{:?}", desc),
        };
        println!("\n{:?}", graph);
    }
    #[test]
    fn open_lib_file() {
        let file = Path::new("examplefiles/lib_documented.lib");
        let mut graph: mn::MoveGraph = match open_file(file) {
            Ok(gr) => gr,
            Err(desc) => panic!("err, {:?}", desc),
        };
        println!("\n{:?}", graph);
        // panic!("Intended!");
    }
}
