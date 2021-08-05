#![allow(dead_code)]

use std::fmt;
use std::ops::{Deref, DerefMut};
use std::iter::FromIterator;
use std::char;
use crate::errors::*;
/// Enum for `Stone`,
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Stone {
    Empty,
    White,
    Black,
}

impl Stone {
    pub fn opposite(self) -> Stone {
        match self {
            Stone::Empty => Stone::Empty,
            Stone::Black => Stone::White,
            Stone::White => Stone::Black,
            //_ => unreachable!(),
        }
    }
}
impl Default for Stone {
    fn default() -> Stone {
        Stone::Empty
    }
}

impl fmt::Display for Stone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}",
               match *self {
                   Stone::Empty => ".",
                   Stone::White => "O",
                   Stone::Black => "X",
               })
    }
}
/// A coordinate located at (`x`, `y`)
#[derive(Clone, Copy)]
pub struct Point {
    /// Whether the point is outside the board, ie a null point.
    pub is_null: bool,
    pub x: u32,
    pub y: u32,
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.is_null {
            write!(f,
                   "[{:>2}, {:>2}]",
                   ((self.x as u8 + 65u8) as char),
                   15 - self.y)
        } else {
            write!(f, "None")
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BoardText;

/// Holds info about the marker at `Point`.
///
/// # Notes
/// This will hopefully have more fields in the future, planned for is support for comments on each
/// marker.
#[derive(Clone)]
pub struct BoardMarker {
    pub point: Point,
    pub color: Stone,
    pub comment: Option<String>,
    pub board_text: Option<BoardText>, // Should find a better way to do this, maybe Vec<(Point, &'static str)>,
}

impl BoardMarker {
    pub fn new(point: Point, color: Stone) -> BoardMarker {
        BoardMarker {
            point,
            color,
            comment: None,
            board_text: None,
        }
    }
    // Are the following functions needed?
    pub fn set_pos(&mut self, point: &Point) {
        self.point = *point;
    }
    pub fn set_comment(&mut self, comment: String) {
        self.comment = if !comment.is_empty() {
            Some(comment)
        } else {
            None
        };
    }
}

impl fmt::Debug for BoardMarker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.point.is_null {
            write!(f,
                   "|[{:>2}, {:>2}]{:?}|",
                   ((self.point.x as u8 + 65u8) as char),
                   15 - self.point.y,
                   self.color)
        } else {
            write!(f, "|     None    |")
        }
    }
}

impl fmt::Display for BoardMarker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}",
               if self.point.is_null {
                   "."
               } else {
                   match self.color {
                       Stone::Empty => ".",
                       Stone::White => "O",
                       Stone::Black => "X",
                   }
               })
    }
}

impl Point {
    /// Makes a `Point` at (`x`, `y`)
    pub fn new(x: u32, y: u32) -> Point {
        Point {
            is_null: false,
            x,
            y,
        }
    }

    pub fn null() -> Point {
        Point {
            is_null: true,
            x: 0,
            y: 0,
        }
    }
    /// Converts a 1D coord to a `Point`
    pub fn from_1d(idx: u32, width: u32) -> Point {
        Point {
            is_null: idx >/*=*/ width*width,
            x: idx % width,
            y: idx / width,
        }
    }
    /// Convert back a `Point` to a 1D coord
    pub fn to_1d(self, width: u32) -> u32 {
        self.x + self.y * width
    }
}

/// Holds all `BoardMarker`'s in a `Board`.
#[derive(Debug)]
pub struct BoardArr(Vec<BoardMarker>);

impl BoardArr {
    fn new() -> BoardArr {
        BoardArr(Vec::new())
    }

    fn add(&mut self, elem: BoardMarker) {
        self.0.push(elem);
    }
}

impl Deref for BoardArr {
    type Target = Vec<BoardMarker>;

    fn deref(&self) -> &Vec<BoardMarker> {
        &self.0
    }
}

impl DerefMut for BoardArr {
    fn deref_mut(&mut self) -> &mut Vec<BoardMarker> {
        &mut self.0
    }
}


#[derive(Debug)]
/// Board type. Holds the data for a game.
pub struct Board {
    pub boardsize: u32,
    pub last_move: Option<Point>,
    pub board: BoardArr,
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
        write!(f,
               "\n   {}",
               (b'A'..b'A' + 15)
                   .map(|d| (d as char).to_string())
                   .collect::<Vec<_>>()
                   .join(" "))
    }
}

impl Board {
    /// Makes a new Board of size `boardsize`.
    ///
    /// # Examples
    /// ```
    /// # use renju_board::board_logic::Board;
    /// let mut board = Board::new(15);
    /// ```
    pub fn new(boardsize: u32) -> Board {
        let board: BoardArr = (0..boardsize * boardsize)
            .map(|idx| BoardMarker::new(Point::from_1d(idx, boardsize), Stone::Empty))
            .collect();

        Board {
            boardsize,
            last_move: None,
            board,
        }
    }
    /// Sets all `BoardMarker`'s to `Stone::Empty`
    pub fn clear(&mut self) {
        self.board = (0..self.boardsize * self.boardsize)
            .map(|idx| BoardMarker::new(Point::from_1d(idx, self.boardsize), Stone::Empty))
            .collect();
    }
    /// Returns a immutable reference to the `BoardMarker` at `pos`
    pub fn get(&self, pos: Point) -> Option<&BoardMarker> {
        self.board.get(pos.to_1d(self.boardsize) as usize)
    }
    /// Returns a immutable reference to the `BoardMarker` at (`x`,`y`)
    pub fn getxy(&self, x: u32, y: u32) -> Option<&BoardMarker> {
        self.board.get((x + y * self.boardsize) as usize)
    }
    pub fn get_i32xy(&self, x: i32, y: i32) -> Option<&BoardMarker> {
        if (x + 1).is_positive() && (y + 1).is_positive() {
            // O is also valid
            self.getxy(x as u32, y as u32)
        } else {
            None
        }
    }
    /// Returns a mutable reference to the `BoardMarker` at `pos`
    pub fn get_mut(&mut self, pos: Point) -> Option<&mut BoardMarker> {
        self.board.get_mut(pos.to_1d(self.boardsize) as usize)
    }
    /// Sets the `BoardMarker` at `pos` to `color`
    pub fn set_point(&mut self, pos: Point, color: Stone) {
        self.board[pos.to_1d(self.boardsize) as usize].color = color;
    }

    pub fn set(&mut self, marker: BoardMarker) -> Result<(), ParseError> {
        let idx = marker.point.to_1d(self.boardsize) as usize;
        let mut_marker = self.board.get_mut(idx).ok_or_else(|| ParseError::Other(format!("Couldn't get index {} in board array", idx)))?;
        *mut_marker = marker;
        Ok(())
    }
}

impl FromIterator<BoardMarker> for BoardArr {
    fn from_iter<I: IntoIterator<Item = BoardMarker>>(iterator: I) -> Self {
        let mut c = BoardArr::new();

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
        let mut board = Board::new(15);
        assert_eq!(board.board.len(), 15 * 15);
        let p = Point::new(0, 0);
        board.set_point(p, Stone::White);
        assert_eq!(board.get(p).unwrap().color, Stone::White);
        let p = Point::new(3, 2);
        board.set_point(p, Stone::Black);
        assert_eq!(board.get(p).unwrap().color, Stone::Black);
        // println!("{:?}", board);
        println!("Board\n{}", board.board);
    }

    #[test]
    fn clear_board() {
        let mut board = Board::new(15);
        let p = Point::new(7, 7);
        board.set_point(p, Stone::White);
        println!("Board:\n{}", board.board);
        board.clear();
        println!("Board - Cleared:\n{}", board.board);
        assert_eq!(board.get(p).unwrap().color, Stone::Empty);
    }
}
