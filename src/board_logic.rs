#![allow(dead_code)]

use std::fmt;
use std::ops::Deref;
use std::iter::FromIterator;

#[derive(PartialEq,Eq, Debug)]
pub enum Stone {
    Empty,
    White,
    Black,
}

impl Default for Stone {
    fn default() -> Stone {Stone::Empty}
}

impl fmt::Display for Stone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Stone::Empty => ".",
            Stone::White => "O",
            Stone::Black => "X",
        })
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}


pub struct BoardMarker {
    pub point: Point,
    pub color: Stone,
}

impl fmt::Debug for BoardMarker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "|[{:>2},{:>2}]{:?}|", self.point.x, self.point.y, self.color)
    }
}

impl fmt::Display for BoardMarker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self.color {
            Stone::Empty => ".",
            Stone::White => "O",
            Stone::Black => "X",
        })
    }
}

impl Point {
    pub fn new(x:u32, y:u32) -> Point {
        Point { x: x, y:y, }
    }
    pub fn from_1d(idx: u32, width: u32) -> Point {
        Point {
            x: idx % width,
            y: idx / width,
        }
    }
    pub fn to_1d(self, width: u32) -> u32 {
        self.x + self.y * width
    }
}

#[derive(Debug)]
pub struct VecBoard(Vec<BoardMarker>);

impl VecBoard {
    fn new() -> VecBoard {
        VecBoard(Vec::new())
    }

    fn add(&mut self, elem: BoardMarker) {
        self.0.push(elem);
    }
}

impl Deref for VecBoard {
    type Target = Vec<BoardMarker>;

    fn deref(&self) -> &Vec<BoardMarker> {
        &self.0
    }
}

#[derive(Debug)]
pub struct Board {
    pub boardsize: u32,
    pub last_move: Option<Point>,
    pub board: VecBoard,
}

impl fmt::Display for VecBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Not sure if needed - let vec: Vec<BoardMarker> = *self;
        let mut dy: u32 = 0;
        let width: u32 = self.last().clone().unwrap().point.y;
        for marker in self.iter() {
            if marker.point.y == dy {
                if marker.point.x != width {
                    write!(f, "{} ", marker);
                } else {
                    write!(f, "{}", marker);
                }
            } else {
                dy += 1;
                writeln!(f, "{}  ", marker);
            }
        }
        write!(f, "")
    }
    
}

impl Board {
    pub fn new(boardsize: u32) -> Board {
        let board: VecBoard = (0..boardsize*boardsize).map(|idx| { 
                BoardMarker { point: Point::from_1d(idx, boardsize), color: Stone::Empty }
            }).collect();
        Board {
            boardsize: boardsize,
            last_move: None,
            board: board,
        }
    }
    pub fn clear(&mut self) {
        self.board = (0..self.boardsize*self.boardsize).map(|idx| {
            BoardMarker { point: Point::from_1d(idx, self.boardsize), color: Stone::Empty }
        }).collect();
    }
    // FIXME: Use `Result` instead of Option
    pub fn get(&self, pos: Point) -> Option<&BoardMarker> {
        self.board.get(pos.to_1d(self.boardsize) as usize)
    }
    
    pub fn getxy(&self, x: u32, y: u32) -> Option<&BoardMarker> {
        self.board.get((x + y*self.boardsize) as usize)
    }
    pub fn get_mut(&mut self, pos: Point) -> Option<&mut BoardMarker> {
        self.board.get_mut(pos.to_1d(self.boardsize) as usize)
    }
    /// Set `pos` as `color`
    pub fn set(&mut self, pos: Point, color: Stone) {
        self.board[pos.to_1d(self.boardsize) as usize].color = color;
    }
}

impl FromIterator<BoardMarker> for VecBoard {
    fn from_iter<I: IntoIterator<Item=BoardMarker>>(iterator: I) -> Self {
        let mut c = VecBoard::new();

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
    fn check_if_board_works(){
        let mut board = Board::new(15);
        assert_eq!(board.board.len(), 15*15);
        let p = Point {x:0, y: 0};
        board.set(p, Stone::White);
        assert_eq!(board.get(p).unwrap().color, Stone::White);
        let p = Point {x:3, y: 2};
        board.set(p, Stone::Black);
        assert_eq!(board.get(p).unwrap().color, Stone::Black);
        println!("{:?}", board.board);
        println!("{}", board.board);
    }

    #[test]
    fn clear_board() {
        let mut board = Board::new(15);
        let p = Point {x:7, y:7};
        board.set(p, Stone::White);
        println!("Board: {:?}", board);
        board.clear();
        println!("Board(Cleared): {:?}\nPos: {:?}", board, board.get(p));
        assert_eq!(board.get(p).unwrap().color, Stone::Empty); 
    }
}
