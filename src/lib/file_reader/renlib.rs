//! Functions for handling renlib files.
use crate::errors::*;
use std::io::Read;

use crate::move_node::MoveGraph;

pub mod old;
pub mod parser;

#[derive(Debug)]
#[non_exhaustive]
pub enum Version {
    V30,
    V34,
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
            Mask => 63, // 0xFFFF3F,
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
        let variants = vec![
            Down, Right, OldComment, Mark, Comment, Start, NoMove, Extension, Mask,
        ];
        variants
            .into_iter()
            .filter(|variant| self.flag(variant))
            .collect()
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

pub fn parse_lib(
    mut file: std::io::BufReader<std::fs::File>,
) -> Result<MoveGraph, color_eyre::Report> {
    let mut header = [0u8; 20];
    file.read_exact(&mut header)?;
    match validate_lib(&header)? {
        Version::V30 => parser::parse_v3x(file, Version::V30),
        Version::V34 => parser::parse_v3x(file, Version::V34),
    }
}

pub fn validate_lib(header: &[u8]) -> Result<Version, ParseError> {
    match *header {
        [0xff, 0x52, 0x65, 0x6e, 0x4c, 0x69, 0x62, 0xff, majv, minv, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff] => {
            match (majv, minv) {
                (3, 0) => Ok(Version::V30),
                (3, 4) => Ok(Version::V34),
                (majv, minv) => Err(ParseError::VersionNotSupported { majv, minv }),
            }
        }
        _ => Err(ParseError::NotSupported),
    }
}
