#![allow(dead_code)]

use crate::errors::ParseError;
use crate::file_reader::renlib::Command;
use crate::file_reader::renlib::CommandVariant;

use std::char;
use std::fmt;
use std::iter::FromIterator;
use std::ops::Deref;

#[macro_export]
macro_rules! p {
    [$([$($i:tt)*]),* $(,)?] => {
        [
            $(
                $crate::p![$($i)*]
            ),*
        ]
    };
    [$x:ident, $y:literal] => {
        Point::new((stringify!($x).chars().next().unwrap() as u8 - b'A') as u32, 15-$y)
    };
}

/// Enum for `Stone`,
#[derive(Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Stone {
    Empty,
    White,
    Black,
}

impl Stone {
    #[must_use]
    pub fn opposite(self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Black => Self::White,
            Self::White => Self::Black,
            //_ => unreachable!(),
        }
    }

    /// Returns `true` if the stone is [`Empty`].
    ///
    /// [`Empty`]: Stone::Empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Returns `true` if the stone is [`White`].
    ///
    /// [`White`]: Stone::White
    #[must_use]
    pub fn is_white(&self) -> bool {
        matches!(self, Self::White)
    }

    /// Returns `true` if the stone is [`Black`].
    ///
    /// [`Black`]: Stone::Black
    #[must_use]
    pub fn is_black(&self) -> bool {
        matches!(self, Self::Black)
    }

    /// Create a stone from a boolean, true = black, false = white
    pub fn from_bool(b: bool) -> Self {
        if b {
            Stone::Black
        } else {
            Stone::White
        }
    }
}
impl Default for Stone {
    fn default() -> Self {
        Self::Empty
    }
}

impl fmt::Display for Stone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Self::Empty => ".",
                Self::White => "O",
                Self::Black => "X",
            }
        )
    }
}
/// A coordinate located at (`x`, `y`)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Point {
    /// Whether the point is outside the board, ie a null point.
    pub is_null: bool,
    pub x: u32,
    pub y: u32,
}

impl Point {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        let Point { x, y, .. } = *self;
        // FIXME: Assumes grid size 15x15
        x == 0 && y == 0 || (0..=14).contains(&x) && (0..=14).contains(&y)
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (x, y) = (((self.x as u8 + 65u8) as char), 15 - self.y);
        if f.alternate() {
            return write!(
                f,
                "Point {{ x: {}, y: {}, is_null: {}, repr: \"[{:>1}, {:>2}]\" }}",
                self.x, self.y, self.is_null, x, y
            );
        }
        if !self.is_null {
            write!(f, "[{:>1}, {:>2}]", x, y)
        } else {
            write!(f, "None")
        }
    }
}

/// Holds info about the marker at `Point` or a move.
///
/// # Notes
/// This will hopefully have more fields in the future, planned for is support for comments on each
/// marker.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BoardMarker {
    pub point: Point,
    pub color: Stone,
    pub oneline_comment: Option<String>,
    pub multiline_comment: Option<String>,
    pub board_text: Option<String>,
    pub command: Command, // TODO: Frank, UINT doesn't have enough bits for 0xffff00
    pub index_in_file: Option<usize>,
}

impl BoardMarker {
    #[must_use]
    #[track_caller]
    pub fn new(point: Point, color: Stone) -> Self {
        // FIXME: Why is null point not allowed?
        // assert!(!point.is_null);
        Self {
            point,
            color,
            oneline_comment: None,
            multiline_comment: None,
            board_text: None,
            command: Command::new(0).unwrap(),
            index_in_file: None,
        }
    }

    fn _new(point: Point, color: Stone) -> Self {
        Self {
            point,
            color,
            oneline_comment: None,
            multiline_comment: None,
            board_text: None,
            command: Command::new(0).unwrap(),
            index_in_file: None,
        }
    }

    #[must_use]
    pub fn null() -> Self {
        let mut m = Self::_new(Point::null(), Stone::Empty);
        *m.command = CommandVariant::NOMOVE;
        m
    }

    pub fn from_pos_info(pos: u8, info: u32) -> Result<Self, color_eyre::eyre::Error> {
        Ok(Self {
            point: Point::from_byte(pos)?,
            color: Stone::Empty,
            oneline_comment: None,
            multiline_comment: None,
            board_text: None,
            command: Command::new(info)?,
            index_in_file: None,
        })
    }
    // Are the following functions needed?
    pub fn set_pos(&mut self, point: &Point) {
        self.point = *point;
    }

    pub fn set_oneline_comment(&mut self, comment: String) {
        self.oneline_comment = if !comment.is_empty() {
            Some(comment)
        } else {
            None
        };
    }
    pub fn set_multiline_comment(&mut self, comment: String) {
        self.multiline_comment = if !comment.is_empty() {
            Some(comment)
        } else {
            None
        };
    }
}

impl fmt::Debug for BoardMarker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !f.alternate() {
            if !self.point.is_null {
                write!(
                    f,
                    "|[{:>1},{:>2}]{}|",
                    ((self.point.x as u8 + 65u8) as char),
                    15 - self.point.y,
                    match self.color {
                        Stone::Empty => ".",
                        Stone::White => "O",
                        Stone::Black => "X",
                    }
                )
            } else {
                write!(f, "| None |")
            }
        } else {
            f.debug_struct("BoardMarker")
                .field("point", &self.point)
                .field("color", &self.color)
                .field("oneline_comment", &self.oneline_comment)
                .field("multiline_comment", &self.multiline_comment)
                .field("board_text", &self.board_text)
                .field("command", &self.command)
                .field(
                    "0xindex_in_file",
                    &format!("0x{:X}", self.index_in_file.unwrap_or_default()),
                )
                .field("index_in_file", &self.index_in_file)
                .finish()
        }
    }
}

impl fmt::Display for BoardMarker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            if self.point.is_null {
                "."
            } else {
                match self.color {
                    Stone::Empty if self.oneline_comment.is_some() => {
                        &self.oneline_comment.as_deref().unwrap()[0..1]
                    }
                    Stone::Empty => ".",
                    Stone::White => "O",
                    Stone::Black => "X",
                }
            }
        )
    }
}

impl Point {
    /// Get point from byte
    ///
    /// This should be similar to `RenLib/Utils.cpp:633` `CPoint Utils::PosToPoint(int pos)`, but with bitwise logic. Not sure what the check in `RenLib/RenLibDoc.cpp:2119` is
    pub fn from_byte(byte: u8) -> Result<Self, ParseError> {
        Ok(Self::new(
            u32::from(
                match byte.checked_sub(1) {
                    Some(value) => value,
                    None => return Err(ParseError::Other("Underflowed position".to_string())),
                } & 0x0f,
            ),
            u32::from(byte >> 4),
        ))
    }
    /// Makes a `Point` at (`x`, `y`)
    #[must_use]
    pub const fn new(x: u32, y: u32) -> Self {
        Self {
            is_null: false,
            x,
            y,
        }
    }

    #[must_use]
    pub fn null() -> Self {
        Self {
            is_null: true,
            x: 0,
            y: 0,
        }
    }
    /// Converts a 1D coord to a `Point`
    #[must_use]
    pub fn from_1d(idx: u32, width: u32) -> Self {
        Self {
            is_null: idx >/*=*/ width*width,
            x: idx % width,
            y: idx / width,
        }
    }
    /// Convert back a `Point` to a 1D coord
    #[must_use]
    pub fn to_1d(self, width: u32) -> u32 {
        self.x + self.y * width
    }
}

/// Holds all `BoardMarker`'s in a `Board`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BoardArr(Vec<BoardMarker>, u32);

impl BoardArr {
    pub fn new(size: u32) -> Self {
        let mut b = Self(vec![BoardMarker::null(); (size * size) as usize], size);
        for idx in 0..(size * size) {
            b.get_mut(idx as usize).unwrap().point = Point::from_1d(idx, size);
        }
        b
    }

    pub fn size(&self) -> u32 {
        self.1
    }

    pub fn set(&mut self, marker: BoardMarker) -> Result<(), ParseError> {
        let idx = marker.point.to_1d(self.1) as usize;
        let mut_marker = self.0.get_mut(idx).ok_or_else(|| {
            ParseError::Other(format!("Couldn't get index {} in board array", idx))
        })?;
        *mut_marker = marker;
        Ok(())
    }

    /// Internal function to add to array, use [Self::set_point] or [Self::set] to actually modify board
    fn add(&mut self, elem: BoardMarker) {
        self.0.push(elem);
    }

    /// Sets all `BoardMarker`'s to `Stone::Empty`
    pub fn clear(&mut self) {
        self.0 = (0..self.1 * self.1)
            .map(|idx| BoardMarker::new(Point::from_1d(idx, self.1), Stone::Empty))
            .collect();
    }
    /// Returns a immutable reference to the `BoardMarker` at `pos`
    #[must_use]
    #[track_caller]
    pub fn get_point(&self, pos: Point) -> Option<&BoardMarker> {
        let marker = self.0.get(pos.to_1d(self.1) as usize);
        if let Some(marker) = marker {
            assert_eq!(marker.point, pos);
        }
        marker
    }
    /// Returns a immutable reference to the `BoardMarker` at (`x`,`y`)
    #[must_use]
    pub fn get_xy(&self, x: u32, y: u32) -> Option<&BoardMarker> {
        self.0.get((x + y * self.1) as usize)
    }
    /// Returns a mutable reference to the `BoardMarker` at (`x`,`y`)
    #[must_use]
    pub fn get_xy_mut(&mut self, x: u32, y: u32) -> Option<&mut BoardMarker> {
        self.0.get_mut((x + y * self.1) as usize)
    }
    #[must_use]
    pub fn get_i32xy(&self, x: i32, y: i32) -> Option<&BoardMarker> {
        if (x + 1).is_positive() && (y + 1).is_positive() {
            // O is also valid
            self.get_xy(x as u32, y as u32)
        } else {
            None
        }
    }
    /// Returns a mutable reference to the `BoardMarker` at `pos`
    pub fn get_point_mut(&mut self, pos: Point) -> Option<&mut BoardMarker> {
        self.0.get_mut(pos.to_1d(self.1) as usize)
    }

    /// Returns a mutable reference to the `BoardMarker` at `pos`
    pub fn get_mut(&mut self, pos: usize) -> Option<&mut BoardMarker> {
        self.0.get_mut(pos)
    }
    /// Sets the `BoardMarker` at `pos` to `color`
    pub fn set_point(&mut self, pos: Point, color: Stone) {
        self.0[pos.to_1d(self.1) as usize].color = color;
    }
}

impl Deref for BoardArr {
    type Target = Vec<BoardMarker>;

    fn deref(&self) -> &Vec<BoardMarker> {
        &self.0
    }
}

#[derive(Debug)]
/// Board type. Holds the data for a game.
pub struct DisplayBoard {
    pub boardsize: u32,
    pub last_move: Option<Point>,
    pub inner: BoardArr,
}

impl fmt::Display for BoardArr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Not sure if needed - let vec: Vec<BoardMarker> = *self;
        let mut dy: u32 = 0;
        let width: u32 = self.last().unwrap().point.y + 1;
        write!(f, "15:")?;
        for marker in self.iter() {
            if marker.point.y == dy {
                if marker.point.x != width {
                    write!(f, "{} ", marker)?;
                } else {
                    write!(f, "{}", marker)?;
                }
            } else {
                dy += 1;
                write!(f, "\n{:2}:{} ", 15 - dy, marker)?;
            }
        }
        write!(
            f,
            "\n   {}",
            (b'A'..b'A' + 15)
                .map(|d| (d as char).to_string())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

impl DisplayBoard {
    /// Makes a new `DisplayBoard` of size `boardsize`.
    ///
    /// # Examples
    /// ```
    /// # use renju_board::board_logic:Display:Board;
    /// let mut board = DisplayBoard::new(15);
    /// ```
    #[must_use]
    pub fn new(boardsize: u32) -> Self {
        let board: BoardArr = (0..boardsize * boardsize)
            .map(|idx| BoardMarker::new(Point::from_1d(idx, boardsize), Stone::Empty))
            .collect();

        Self {
            boardsize,
            last_move: None,
            inner: board,
        }
    }
}

impl FromIterator<BoardMarker> for BoardArr {
    fn from_iter<I: IntoIterator<Item = BoardMarker>>(iterator: I) -> Self {
        let mut c = Self::new(15); // TODO: default size

        for i in iterator {
            c.add(i);
        }
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn check_if_board_works() {
        let mut board = BoardArr::new(15);
        assert_eq!(board.len(), 15 * 15);
        let p = Point::new(0, 0);
        board.set_point(p, Stone::White);
        assert_eq!(board.get_point(p).unwrap().color, Stone::White);
        let p = Point::new(3, 2);
        board.set_point(p, Stone::Black);
        assert_eq!(board.get_point(p).unwrap().color, Stone::Black);
        // tracing::info!("{:?}", board);
        tracing::info!("Board\n{}", board);
    }

    #[test]
    fn clear_board() {
        let mut board = BoardArr::new(15);
        let p = Point::new(7, 7);
        board.set_point(p, Stone::White);
        tracing::info!("Board:\n{}", board);
        board.clear();
        tracing::info!("Board - Cleared:\n{}", board);
        assert_eq!(board.get_point(p).unwrap().color, Stone::Empty);
    }
}
