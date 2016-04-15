#![allow(dead_code)]

#[derive(PartialEq,Eq, Debug)]
pub enum Stone {
    Empty,
    White,
    Black,
}

impl Default for Stone {
    fn default() -> Stone {Stone::Empty}
}
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug)]
pub struct BoardMarker {
    pub point: Point,
    pub color: Stone,
}
impl Default for BoardMarker {
    fn default() -> BoardMarker {BoardMarker {point: Point::new(0,0), color: Stone::Empty}}
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
pub struct Board {
    pub boardsize: u32,
    pub last_move: Option<Point>,
    pub board: Vec<BoardMarker>,
}


impl Board {
    pub fn new(boardsize: u32) -> Board {
        let board: Vec<BoardMarker> = (0..boardsize*boardsize).map(|idx| { 
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
