#![allow(dead_code)]

#[derive(PartialEq,Eq, Debug)]
pub enum Stone {
    Empty,
    P1,
    P2,
}

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

pub struct BoardMarker {
    pub point: Point,
    pub marker: Stone,
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

pub struct Board {
    rows: u32,
    cols: u32,
    last_move: Option<Point>,
    board: Vec<BoardMarker>,
}


impl Board {
    pub fn new(rows: u32, cols: u32) -> Board {
        assert_eq!(rows, cols); // TODO: FIXME, update Point::*_1d to work on generic boards.
        let board: Vec<BoardMarker> = (0..rows * cols).map(|idx| { BoardMarker { point: Point::from_1d(idx, rows), marker: Stone::Empty }}).collect();
        Board {
            rows: rows,
            cols: cols,
            last_move: None,
            board: board,
        }
    }
    
    pub fn get(&self, pos: Point) -> Option<&BoardMarker> {
        self.board.get(pos.to_1d(self.rows) as usize)
    }
    pub fn get_mut(&mut self, pos: Point) -> Option<&mut BoardMarker> {
        self.board.get_mut(pos.to_1d(self.rows) as usize)
    }
    
    pub fn set(&mut self, pos: Point, marker: Stone) {
        self.board[pos.to_1d(self.rows) as usize].marker = marker;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn check_if_board_works(){
        let mut board = Board::new(15, 15);
        assert_eq!(board.board.len(), 15*15);
        let p = Point {x:0, y: 0};
        board.set(p, Stone::P1);
        assert_eq!(board.get(p).unwrap().marker, Stone::P1);
        let p = Point {x:3, y: 2};
        board.set(p, Stone::P2);
        assert_eq!(board.get(p).unwrap().marker, Stone::P2);
    }
    
    #[test]
    fn check_if_invalid_move(){
        let mut board = Board::new(15, 15);
        let p = Point {x:16, y:16};
        assert!(board.get(p).is_none());
    }
    
    #[test]
    fn check_if_illegal_move(){
        let positions: Vec<Point> = (7*15 + 7,7*15 +6,6*15 +6, 5*15 +7).
        .iter().map(|d| Point::from_1d(d, 15)).collect();
        let illegal: Point = Point::from_1d(7*15 + 5, 15);
        println!("{:?}", illegal);
    }
}
