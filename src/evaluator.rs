use board_logic::{Board, BoardMarker};


#[derive(Debug, PartialEq)]
pub enum Direction {
    Horizontal,
    Vertical,
    Diagonal, // ´\´
    AntiDiagonal, // ´/´
}

#[derive(PartialEq, Debug)]
pub enum EvalError {
    NoMatch,
    OutOfBounds,
}


pub fn is_five(board: &Board, marker: BoardMarker) -> Result<Direction, EvalError> {
    // OK this will be hard, let's do it.

    let mut n_line = 1;
    { // Horizontal
        debug!("Horiz right:");
        debug!("\tStart: {}", marker.point.x+1);
        'right: for i in marker.point.x+1..board.boardsize {
            if board.getxy(i, marker.point.y).unwrap().color == marker.color {
                n_line += 1;
            } else {
                debug!("\tEnd: {}", i);
                break 'right;
            }
        }
        debug!("Horiz left:");
        debug!("\tStart: {}", marker.point.x-1);
        'left: for i in (0..marker.point.x).rev() {
            if board.getxy(i, marker.point.y+200).unwrap().color == marker.color {
                n_line += 1;
            } else {
                debug!("\tEnd: {}", i);
                break 'left;
            }
        }

        if n_line >= 4 {
            debug!("Horizontal Line length: {}", n_line);
            return Ok(Direction::Horizontal);
        }
        n_line=0;
    }
    { // Vertical
        debug!("Vert down:");
        debug!("\tStart: {}", marker.point.y+1);
        'down: for i in marker.point.y+1..board.boardsize {
            if board.getxy(marker.point.x, i).unwrap().color == marker.color {
                n_line += 1;
            } else {
                debug!("\tEnd: {}", i);
                break 'down;
            }
        }
        debug!("Vert up: ");
        debug!("\tStart: {}", marker.point.x-1);
        'up: for i in (0..marker.point.y).rev() { // if it is suppossed to be 0..y+1 or 0..y is not clear
            if board.getxy(marker.point.x, i).unwrap().color == marker.color {
                n_line += 1;
            } else {
                debug!("\tEnd: {}", i);
                break 'up;
            }
        }

        debug!("Vertical line length: {}", n_line);
        if n_line >= 4 {
            return Ok(Direction::Vertical);
        }
        n_line = 0;
    }
    { // Diagonal '\'
        debug!("Diagonal down:");
        debug!("\tStart: {}_{}", marker.point.x+1, marker.point.y+1); 
    }
    Err(EvalError::NoMatch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use board_logic::{Board, BoardMarker, Stone, Point};

    #[test]
    #[ignore]
    fn check_if_illegal_move() {
        let mut board = Board::new(15);
        for pos in [7 * 15 + 7, 7 * 15 + 6, 6 * 15 + 6, 5 * 15 + 7].iter() {
            board.set_point(Point::from_1d(*pos, 15), Stone::Black);
        }

        let illegal = Point::from_1d(7 * 15 + 5, 15);
        println!("{:?}", illegal);
    }

    #[test]
    fn is_horizontal_five_in_a_row() {
        let mut board = Board::new(15);
        let y = 7u32;
        let p1 = BoardMarker { point: Point::new(4, y), color: Stone::Black };
        for x in 0..4 {
            board.set_point(Point::new(x, y), Stone::Black);
        }

        let p2 = BoardMarker { point: Point::new(8, y + 2), color: Stone::White };
        for x in (7..12).filter(|x| *x != 8) {
            board.set_point(Point::new(x, y + 2), Stone::White);
        }
        println!("\n{}\nChecks,{:?} and {:?}",
                 board.board, p1, p2);
        
        assert_eq!(is_five(&board, p1), Ok(Direction::Horizontal));
        // assert_eq!(is_five(&board, p2).unwrap(), Direction::Horizontal);
    }

    #[test]
    fn is_vertical_five_in_a_row() {  
        let mut board = Board::new(15);
        let x = 7u32;
        let p1 = BoardMarker { point: Point::new(x, 4), color: Stone::Black };
        for y in 0..4 {
            board.set_point(Point::new(x, y), Stone::Black);
        }

        let p2 = BoardMarker { point: Point::new(x + 2, 8), color: Stone::White };
        for y in (7..12).filter(|y| *y != 8) {
            board.set_point(Point::new(x+2, y), Stone::White);
        }
        println!("\n{}\nChecks; {:?} and {:?}",
                 board.board, p1, p2);
        
        assert_eq!(is_five(&board, p1), Ok(Direction::Vertical));
        assert_eq!(is_five(&board, p2), Ok(Direction::Vertical));
    }
    #[test]
    fn is_diagonal_five_in_a_row() {
        let mut board = Board::new(15);
        // A diagonal is '\'
        for pos in [7*15 + 2u32, 8*15+3u32, 9*15+4u32, 10*15+5u32].iter() {
            board.set_point(Point::from_1d(*pos, 15), Stone::Black);
        }
        let p1 = BoardMarker { point: Point::from_1d(11*15+6, 15), color: Stone::Black };
        println!("\n{}\nChecks; {:?}",
                 board.board, p1);

        assert_eq!(is_five(&board, p1), Ok(Direction::Diagonal));
    }
}
