//! Legacy ways of hadling renlib, not functional.
use std::str;
use errors::*;

use board_logic::{BoardMarker, Stone, Point};
use move_node::{MoveGraph, MoveIndex};

pub fn parse_lib_legacy(file_u8: Vec<u8>) -> Result<MoveGraph> {
    let mut file_u8 = file_u8;
    //::with_capacity(match file.metadata() { Ok(meta) => meta.len() as usize, Err(err) => return Err(FileErr::OpenError)});
    let header: Vec<u8> = file_u8.drain(0..20).collect();
    // let Game = unimplemented!();
    let major_file_version = header[8] as u32;
    let minor_file_version = header[9] as u32;
    if major_file_version != 3 {
        return Err(ErrorKind::VersionNotSupported.into());
    }
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
        while current_command == 0x57 {
            println!("What is this???");
            {
                let mut skipped = command_iter.by_ref().take(2);
                println!("Skipped 0x{:02x}, 0x{:02x}, peek next",
                         skipped.next().unwrap(),
                         skipped.next().unwrap());
            }
            println!("Now on {:?}", command_iter.peek());
            command_iter.next();
            current_command = command_iter.next().unwrap();
        }
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
                    }
                    None => {
                        let val = *branches.last().ok_or("Failed reading branches.last()")?;
                        println!("adding move to last branch:{:?}", val);
                        val
                    }
                };

                // println!("\tAdded to {:?}.", last_child);
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
                                                 BoardMarker::new(Point::from_1d(5, 2),
                                                                  Stone::Empty)
                                             }));
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
                    let move_ind: MoveIndex =
                        graph.add_move(*children.last().unwrap(),
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
                    None => {
                        if command_iter.peek().is_some() {
                            return Err("No command is next".into());
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
                Some(children[children_len - 2])
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
                     branches,
                     moves);
        }
        if current_command & 0x80 == 0x80 {
            // if we are saying: This node has siblings!.
            // This means that the children are
            // TODO: A sibling can be first move.
            // NOTE: If we are both 0x80 and 0x40 what happens?
            // I believe 0x40 should be checked first.
            println!("We have some siblings. Add my parent to branches and replace with children");
            let children_len = children.len();
            if children_len <= 2 {
                // Not sure why.
                // println!("Children that error me! {:?}", children);
                // return Err(ErrorKind::LibParseError.into());
                // FIXME: May be wrong.
                if moves < 2 {
                    multiple_start = 1;

                }

                continue 'main;
            }
            // The one we just pushed has siblings, that means the branch is on the parent.
            let parent = children[children_len - 2];
            let child = children[children_len - 1];
            println!("Parent is {:?}", parent);
            branches.push(parent);
            children = vec![parent];
            children.push(child);

            // OLD CODE: May be wrong or right.
            //
            // let lost_child = match children.last() {
            //    Some(last) => last.clone(),
            //    None => {
            //        error!("Failed reading children.last()!");
            //        return Err(ErrorKind::LibParseError.into());
            //    }
            // ; // Not sure if need clone.
            // branches.push(children[children_len - 2]);
            // children = vec![children[children_len - 2]];
            // println!("--NEW-- Children: {:?}", children);
            // children.push(lost_child);
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
                                     Err(ErrorKind::LibParseError.into())
                                 }
                      }
                  } != 0x00 {
                title.push(command_iter.next().unwrap()); // This should be safe.
            }
            while match command_iter.peek() {
                      Some(command) => *command,
                      None => {
                          return {
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
            match children.last() { // FIXME: Doesn't work in main
                Some(last) => {
                    let mut marker =
                        graph
                            .get_move_mut(*last)
                            .expect("FIXME -- Child was added to vec but not to graph.");
                    println!("Setting comment on {:?}", marker);
                    marker.set_comment(format!("Title: {}, Comment: {}",
                     str::from_utf8(&title).unwrap_or("Failed to parse title!"),
                     str::from_utf8(&comment).unwrap_or("Failed to parse comment!")));
                    println!("Comment set as {:?}", marker.comment);

                }
                None => bail!("No last child found :/"),

            }

            // command_iter.skip(title.len() + comment.len() +2);
        }
    }
    Ok(graph)

}
