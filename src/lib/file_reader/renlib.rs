//! Functions for handling renlib files.
use bitflags::bitflags;

use crate::errors::*;
use std::{
    convert::{TryFrom, TryInto},
    io::{BufRead, Read},
};

use crate::move_node::MoveGraph;

pub mod parser;

#[derive(Debug)]
#[non_exhaustive]
pub enum Version {
    V30,
    V34,
}

pub const MASK: u32 = 0xFFFF3F;

bitflags! {
    #[repr(transparent)]
    pub struct CommandVariant: u32 {
        // Extensions

        const BOARDTEXT = 0x100; //

        const DOWN = 0x80;       // 0b10000000
        const RIGHT = 0x40;      // 0b01000000
        const OLDCOMMENT = 0x20; // 0b00100000
        const MARK = 0x10;       // 0b00010000
        const COMMENT = 0x08;    // 0b00001000
        const START = 0x04;      // 0b00000100
        const NOMOVE = 0x02;     // 0b00000010
        const EXTENSION = 0x01;  // 0b00000001
    }
}

bitflags! {
    pub struct Extension: u16 {
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Command(CommandVariant);

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Command").field(&self.0).finish()
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum CommandError {
    #[error("unknown command encountered: {0:#b}, {0:#10x?}")]
    UnknownCommand(u32),
}
impl Command {
    #[inline]
    pub fn new(bits: u32) -> Result<Self, CommandError> {
        Ok(Command(
            CommandVariant::from_bits(bits).ok_or(CommandError::UnknownCommand(bits))?,
        ))
    }
    fn flag(&self, command: CommandVariant) -> bool {
        self.0.contains(command)
    }

    pub fn is_down(&self) -> bool {
        self.flag(CommandVariant::DOWN)
    }

    pub fn is_right(&self) -> bool {
        self.flag(CommandVariant::RIGHT)
    }

    pub fn is_old_comment(&self) -> bool {
        self.flag(CommandVariant::OLDCOMMENT)
    }

    pub fn is_mark(&self) -> bool {
        self.flag(CommandVariant::MARK)
    }

    pub fn is_comment(&self) -> bool {
        self.flag(CommandVariant::COMMENT)
    }

    pub fn is_start(&self) -> bool {
        self.flag(CommandVariant::START)
    }

    pub fn is_no_move(&self) -> bool {
        self.flag(CommandVariant::NOMOVE)
    }

    pub fn is_extension(&self) -> bool {
        self.flag(CommandVariant::EXTENSION)
    }

    pub fn is_board_text(&self) -> bool {
        self.flag(CommandVariant::BOARDTEXT)
    }
}

pub fn parse_lib(mut file: impl BufRead) -> Result<MoveGraph, color_eyre::Report> {
    let vec = match read_header(&mut file)? {
        v @ (Version::V30 | Version::V34) => parser::parse_v3x(file, v),
    }?;
    todo!()
}

pub fn read_header(file: &mut impl BufRead) -> Result<Version, ParseError> {
    let mut header = [0u8; 20];
    file.read_exact(&mut header)?;
    validate_lib(&header)
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
