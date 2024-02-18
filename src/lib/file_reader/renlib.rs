//! Functions for handling renlib files.
use bitflags::bitflags;

use crate::{board_logic::Stone, errors::ParseError};
use std::io::{BufRead, Read};

use crate::board::Board;

pub mod parser;

#[derive(Debug)]
#[non_exhaustive]
pub enum Version {
    V30,
    V34,
}

pub const MASK: u32 = 0x00FF_FF3F;

bitflags! {
    #[repr(transparent)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Command(CommandVariant);

impl std::ops::DerefMut for Command {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Deref for Command {
    type Target = CommandVariant;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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
        Ok(Self(
            CommandVariant::from_bits(bits).ok_or(CommandError::UnknownCommand(bits))?,
        ))
    }
    fn flag(&self, command: CommandVariant) -> bool {
        self.0.contains(command)
    }

    #[must_use]
    pub fn is_down(&self) -> bool {
        self.flag(CommandVariant::DOWN)
    }

    #[must_use]
    pub fn is_right(&self) -> bool {
        self.flag(CommandVariant::RIGHT)
    }

    #[must_use]
    pub fn is_old_comment(&self) -> bool {
        self.flag(CommandVariant::OLDCOMMENT)
    }

    #[must_use]
    pub fn is_mark(&self) -> bool {
        self.flag(CommandVariant::MARK)
    }

    #[must_use]
    pub fn is_comment(&self) -> bool {
        self.flag(CommandVariant::COMMENT)
    }

    #[must_use]
    pub fn is_start(&self) -> bool {
        self.flag(CommandVariant::START)
    }

    #[must_use]
    pub fn is_no_move(&self) -> bool {
        self.flag(CommandVariant::NOMOVE)
    }

    #[must_use]
    pub fn is_extension(&self) -> bool {
        self.flag(CommandVariant::EXTENSION)
    }

    #[must_use]
    pub fn is_board_text(&self) -> bool {
        self.flag(CommandVariant::BOARDTEXT)
    }

    #[must_use]
    pub fn is_move(&self) -> bool {
        !self.is_no_move()
    }
}

pub fn parse_lib(mut file: impl Read, board: &mut Board) -> Result<(), color_eyre::Report> {
    let moves = match read_header(&mut file)? {
        (v @ (Version::V30 | Version::V34), i) => parser::parse_v3x(file, v, i),
    }?;
    let mut _new_moves = 0;
    let mut first_move = None;
    let mut check_root = true;
    let mut stack = vec![];
    // An adaptation of CRenLibDoc::AddLibrary
    board.move_to_root();
    let mut cur_move = board.current_move();
    tracing::debug!("starting parse of file");
    let mut i = 0;
    let ten_percent = moves.len() / 10;
    for mut marker in moves {
        if i % ten_percent == 0 {
            tracing::trace!(?i, "processing");
        }
        i += 1;
        //tracing::trace!(marker = format!("{:#?}", marker), ?cur_move, "processing");
        if marker.command.is_move() {
            let last_move = board
                .move_list()
                .iter()
                .rev()
                .filter_map(|i| board.get_move(*i))
                .map(|m| m.color)
                .find(|c| !c.is_empty())
                .unwrap_or(Stone::White);
            marker.color = match last_move {
                Stone::Black => Stone::White,
                _ => Stone::Black,
            };
        }
        // Not sure this is correct, oh well...
        let next_move = board.get_variant_weird(&cur_move, &marker.point, &marker.color);
        if let Some((m, next)) = next_move {
            tracing::debug!(variant = ?(m, next), "found variant");
            cur_move = next;
        } else {
            let next = board.insert_move(cur_move, marker.clone());
            cur_move = next;
            if marker.command.is_move() {
                _new_moves += 1;
                if first_move.is_none() {
                    first_move = Some(cur_move)
                }
            }
        }
        if check_root && marker.point.is_null {
            check_root = false;
        } else {
            board.add_move_to_move_list(cur_move);
        }

        if marker.command.is_down() {
            stack.push(board.index())
        }

        if marker.command.is_right() && !stack.is_empty() {
            // FIXME: Proper error pls
            let top = stack.pop().expect("stack should not be empty");
            board.set_index(top - 1)?;
            cur_move = board.current_move();
        }
    }
    Ok(())
}

pub fn read_header(mut file: impl Read) -> Result<(Version, usize), ParseError> {
    let mut header = [0u8; 20];
    file.read_exact(&mut header)?;
    Ok((validate_lib(&header)?, 20))
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
