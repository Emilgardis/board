//! Functions for handling renlib files.
use std::str;
use errors::*;

use board_logic::{BoardMarker, Stone, Point};
use move_node::{MoveGraph, MoveIndex};

pub enum Version {
    V30,
    _extended, // Reserve right to extend enum
}

#[derive(Debug)]
pub enum CommandVariant {
    Down,
    Right,
    OldComment,
    Mark,
    Comment,
    Start,
    NoMove,
    Extension,
    Mask,
}

impl CommandVariant {
    pub fn to_u8(&self) -> u8 {
        use self::CommandVariant::*;
        match *self {
            Down => 0x80,
            Right => 0x40,
            OldComment => 0x20,
            Mark => 0x10,
            Comment => 0x08,
            Start => 0x04,
            NoMove => 0x02,
            Extension => 0x01,
            Mask => 0xFFFF3F,
        }
    }
}

#[derive(Debug)]
pub struct Command(pub u8);

impl Command {
    fn flag(&self, command: &CommandVariant) -> bool {
        let check = command.to_u8();
        self.0 & check == check
    }

    pub fn get_all(&self) -> Vec<CommandVariant> {
        use self::CommandVariant::*;
        let mut variants = vec![Down,Right,OldComment,Mark,Comment,Start,NoMove,Extension,Mask];
        variants.into_iter().filter(|variant| self.flag(variant)).collect()
    }

    pub fn is_down(&self) -> bool {
        self.flag(&CommandVariant::Down)
    }

    pub fn is_right(&self) -> bool {
        self.flag(&CommandVariant::Right)
    }
    
    pub fn is_old_comment(&self) -> bool {
        self.flag(&CommandVariant::OldComment)
    }

    pub fn is_mark(&self) -> bool {
        self.flag(&CommandVariant::Mark)
    }

    pub fn is_comment(&self) -> bool {
        self.flag(&CommandVariant::Comment)
    }

    pub fn is_start(&self) -> bool {
        self.flag(&CommandVariant::Start)
    }
    
    pub fn is_no_move(&self) -> bool {
        self.flag(&CommandVariant::NoMove)
    }

    pub fn is_extension(&self) -> bool {
        self.flag(&CommandVariant::Extension)
    }

    pub fn is_mask(&self) -> bool {
        self.flag(&CommandVariant::Mask)
    }
}

pub fn parse_lib(file_u8: Vec<u8>) -> Result<MoveGraph> {
    
    let (header, file) = file_u8.split_at(20);
    match validate_lib(header)? {
        Version::V30 => {
            parse_v30(file)
        }
        _ => unimplemented!(),
    }
}

pub fn validate_lib(header: &[u8]) -> Result<Version> {
    match header {
        &[0xff, 0x52, 0x65, 0x6e, 0x4c, 0x69, 0x62, 0xff, majv, minv,
          0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff] => {
            match (majv, minv) {
                (3, 0) => {
                    Ok(Version::V30)
                }
                _ =>bail!(ErrorKind::VersionNotSupported),
            }
         }
        _ => bail!(ErrorKind::NotSupported),
    } 
}

pub fn byte_to_point(byte: &u8) -> Result<Point> {
    Ok(Point::new(
        (match byte.checked_sub(1) {
            Some(value) => value,
            None => return Err("Underflowed position".into())
        } & 0x0f) as u32,
        (byte >> 4) as u32
    ))
}

fn parse_v30(file: &[u8]) -> Result<MoveGraph> {
    let mut graph = MoveGraph::new();
    let mut prev_index: Option<MoveIndex> = None;
    let mut cur_index: Option<MoveIndex> = None;
    let mut cur_root: Option<MoveIndex> = None;
    let mut iter = file.iter().peekable();
    if iter.peek().map(|u| *u) == Some(&0x00) {
        // No move start, ignore.
        // TODO: Is this valid?
        iter.next();
    }
    // It should just work to do this sequentially and use move_graph functions, let's try that
    while iter.peek().is_some() {
        let mut cur_marker: Option<BoardMarker> = None;
        let byte = iter.next().unwrap();
        let command = Command(*iter.next().ok_or_else(|| "Expected a command byte, got nothing")?);
        let point = byte_to_point(byte)?;
        println!("Point: {:?} Command: {:?} Previous Index: {:?}", point, command.get_all(), cur_index);
        cur_marker = Some(BoardMarker::new(point, Stone::Black)); // First stone is black.

        if cur_index.is_none() {
            prev_index = cur_index;
            cur_index = Some(graph.new_root(cur_marker.clone().unwrap()));
            cur_root = cur_index;
        } else if !(command.is_down() && command.is_right()){
            prev_index = cur_index;
            cur_index = Some(graph.add_move(cur_index.unwrap(), cur_marker.clone().unwrap()));
            
        }
        if command.is_right() && command.is_down() {
            //println!("Popped markeds");
            //graph.marked_for_branch.pop();
            prev_index = cur_index;
            // This branch leaf is alone, go down immidiatly
            cur_index = graph.down_to_branch(cur_index.unwrap());
            graph.add_move(cur_index.unwrap(), cur_marker.unwrap());
        } else {
            if command.is_right() {
                prev_index = None;
                cur_index = graph.down_to_branch(cur_index.unwrap());
                println!("Branching down to, res: {:?}", cur_index.unwrap());
            }
            if command.is_down() {
                println!("Marking {:?} as branch.", prev_index.unwrap_or(cur_root.unwrap()));
                graph.mark_for_branch(prev_index.unwrap_or(cur_root.unwrap()));
            }
        }

    }
    Ok(graph)
}
