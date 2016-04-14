pub enum Stone {
    Empty,
    P1,
    P2,
}
pub struct Point { x: u32, y: u32 }

pub struct BoardMarker {
    pub point: Point,
    pub marker: Stone, // Make this mutable so that we only change this when updating, instead of replacing the whole BoardMarker
    // pub id_mov: String,
}

impl Point {
    pub fn from_1d(idx: u32, width: u32) -> Point {
        Point {x: idx % width, y: idx / width}
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
    pub fn new(&self, rows: u32, cols: u32) -> Board {
        assert_eq!(rows, cols); // TODO: FIXME, update Point::*_1d to work on generic boards.
        let mut board: Vec<BoardMarker> = (0..rows*cols).map(|idx| BoardMarker { point: Point::from_1d(idx, rows), marker: Stone::Empty}).collect();
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
}
