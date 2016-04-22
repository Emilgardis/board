use board_logic::{Board, BoardMarker};

/// Describes direction for `is_line`, etc.
///
/// # Notes
/// Each field will probably in the future hold two integers, these will signify how many stones
/// were found around the matched `BoardMarker`.
#[derive(Debug, PartialEq)]
pub enum Direction {
    Horizontal,
    Vertical,
    Diagonal, // ´\´
    AntiDiagonal, // ´/´
}


/// Evaluation Error. Used by `is_line`, etc.
///
/// # Fields
/// - NoMatch
///     Used when the function did not find any match.
/// - OutOfBounds
///     *DEPRECATED* This should not be used.
#[derive(PartialEq, Debug)]
pub enum Error {
    NoMatch,
    OutOfBounds,
}


pub fn is_line(board: &Board, marker: BoardMarker) -> Result<Direction, Error> {
    debug!("In is_line, checking {:?}", marker);

    { // Horizontal
        let mut n_horiz_right = 0;
        let mut n_horiz_left = 0;
        debug!("Horiz right:");
        debug!("\tStart: {}", marker.point.x+1);
        'right: for i in marker.point.x+1..board.boardsize {
            match board.getxy(i, marker.point.y).ok_or(0) {
                Ok(other_marker) => {
                    debug!("\t{:?}", other_marker);
                    if other_marker.color == marker.color {
                        n_horiz_right += 1;
                    } else {
                        debug!("\tEnd: {}", i);
                        break 'right;
                    }
                },
                Err(_) => return Err(Error::OutOfBounds)
            }
        }
        debug!("Horiz left:");
        debug!("\tStart: {}", marker.point.x.checked_sub(1).unwrap_or(marker.point.x));
        'left: for i in (0..marker.point.x).rev() {
            match board.getxy(i, marker.point.y).ok_or(0) {
                Ok(other_marker) => {
                    debug!("\t{:?}", other_marker);
                    if other_marker.color == marker.color {
                        n_horiz_left += 1;
                    } else {
                        break 'left;
                    }
                },
                Err(_) => return Err(Error::OutOfBounds)
            }
        }

        debug!("Horizontal Line length: {}", n_horiz_right + n_horiz_left);
        if n_horiz_right + n_horiz_left >= 4 {
            return Ok(Direction::Horizontal);
        }
    }
    { // Vertical
        let mut n_vert_down = 0;
        let mut n_vert_up = 0;
        debug!("Vert down:");
        debug!("\tStart: {}", marker.point.y+1);
        'down: for i in marker.point.y+1..board.boardsize {
            match board.getxy(marker.point.x, i).ok_or(0) {
                Ok(other_marker) => {
                    debug!("\t{:?}", other_marker);
                    if other_marker.color == marker.color {
                        n_vert_down += 1;
                    } else {
                        break 'down;
                    }
                },
                Err(_) => return Err(Error::OutOfBounds)
            }
        }
        debug!("Vert up: ");
        debug!("\tStart: {}", marker.point.y.checked_sub(1).unwrap_or(marker.point.y));
        'up: for i in (0..marker.point.y).rev() { // if it is suppossed to be 0..y+1 or 0..y is not clear
            match board.getxy(marker.point.x, i).ok_or(0) {
                Ok(other_marker) => {
                    debug!("\t{:?}", other_marker);
                    if other_marker.color == marker.color {
                        n_vert_up += 1;
                    } else {
                        break 'up;
                    }
                },
                Err(_) => return Err(Error::OutOfBounds) // Shouldn't happen
            }
        }

        debug!("Vertical line length: {}", n_vert_down + n_vert_up);
        if n_vert_down + n_vert_up >= 4 {
            return Ok(Direction::Vertical);
        }
    }
    { // Diagonal '\'
        let mut n_diag_down = 0;
        let mut n_diag_up = 0;
        debug!("Diagonal down:");
        debug!("\tStart: {}_{}", marker.point.x+1, marker.point.y+1); 
        'diag_down: for i in 1..board.boardsize+1 {
            match board.getxy(marker.point.x+i, marker.point.y+i).ok_or(0) {
                Ok(other_marker) => {
                    debug!("\t{:?}", other_marker);
                    if other_marker.color == marker.color {
                        n_diag_down += 1;
                    } else {
                        break 'diag_down;
                    }
                },
                Err(_) => {
                    debug!("\tEnd: {}_{}", marker.point.x+i, marker.point.y+i);
                    break 'diag_down; 
                }
            }
        }
        debug!("Diagonal up:");
        debug!("\tStart: {}_{}", marker.point.x-1, marker.point.y-1); 
        'diag_up: for i in 1..board.boardsize+1 {
            match board.getxy(marker.point.x.checked_sub(i).unwrap_or(1024), marker.point.y.checked_sub(i).unwrap_or(1024)).ok_or(0) {
                Ok(other_marker) => {
                    debug!("\t{:?}", other_marker);
                    if other_marker.color == marker.color {
                        n_diag_up += 1;
                    } else {
                        break 'diag_up;
                    }
                },
                Err(_) => {
                    debug!("\tEnd: {}_{}", marker.point.x+i, marker.point.y+i);
                    break 'diag_up;
                }
            }
        }
           
        debug!("Diagonal line length: {}", n_diag_down + n_diag_up);
        if n_diag_down + n_diag_up >= 4 {
            return Ok(Direction::Diagonal);
        }
    }
    { // AntiDiagonal '/'
        let mut n_anti_diag_down = 0;
        let mut n_anti_diag_up = 0;
        debug!("Anti-Diagonal down:");
        debug!("\tStart: {}_{}", marker.point.x+1, marker.point.y+1); 
        'anti_diag_down: for i in 1..board.boardsize+1 {
            match board.getxy(marker.point.x.checked_sub(i).unwrap_or(1024), marker.point.y+i).ok_or(0) {
                Ok(other_marker) => {
                    debug!("\t{:?}", other_marker);
                    if other_marker.color == marker.color {
                        n_anti_diag_down += 1;
                    } else {
                        debug!("\tEnd: {}_{}", marker.point.x+i, marker.point.y+i);
                        break 'anti_diag_down;
                    }
                },
                Err(_) => {
                    debug!("\tEnd: {}_{}", marker.point.x+i, marker.point.y+i);
                    break 'anti_diag_down; 
                }
            }
        }
        debug!("Anti-Diagonal up:");
        debug!("\tStart: {}_{}", marker.point.x+1, marker.point.y.checked_sub(1).unwrap_or(marker.point.y)); 
        'anti_diag_up: for i in 1..board.boardsize+1 {
            match board.getxy(marker.point.x + i, marker.point.y.checked_sub(i).unwrap_or(1024)).ok_or(0) {
                Ok(other_marker) => {
                    debug!("\t{:?}", other_marker);
                    if other_marker.color == marker.color {
                        n_anti_diag_up += 1;
                    } else {
                        break 'anti_diag_up;
                    }
                },
                Err(_) => {
                    break 'anti_diag_up;
                }
            }
        }

        debug!("Anti-Diagonal line length: {}", n_anti_diag_down + n_anti_diag_up);
        if n_anti_diag_down + n_anti_diag_up >= 4 {
            return Ok(Direction::AntiDiagonal);
        }
    }
    Err(Error::NoMatch)
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
        let p1 = BoardMarker { point: Point::new(4, y), color: Stone::Black, comment: None };
        for x in 0..4 {
            board.set_point(Point::new(x, y), Stone::Black);
        }

        let p2 = BoardMarker { point: Point::new(8, y + 2), color: Stone::White, comment: None };
        for x in (7..12).filter(|x| *x != 8) {
            board.set_point(Point::new(x, y + 2), Stone::White);
        }
        println!("\n{}\nChecks,{:?} and {:?}",
                 board.board, p1, p2);
        
        assert_eq!(is_line(&board, p1), Ok(Direction::Horizontal));
        // assert_eq!(is_line(&board, p2).unwrap(), Direction::Horizontal);
    }

    #[test]
    fn is_vertical_five_in_a_row() {  
        let mut board = Board::new(15);
        let x = 7u32;
        let p1 = BoardMarker { point: Point::new(x, 4), color: Stone::Black, comment: None};
        for y in 0..4 {
            board.set_point(Point::new(x, y), Stone::Black);
        }

        let p2 = BoardMarker { point: Point::new(x + 2, 8), color: Stone::White, comment: None };
        for y in (7..12).filter(|y| *y != 8) {
            board.set_point(Point::new(x+2, y), Stone::White);
        }
        println!("\n{}\nChecks; {:?} and {:?}",
                 board.board, p1, p2);
        
        assert_eq!(is_line(&board, p1), Ok(Direction::Vertical));
        assert_eq!(is_line(&board, p2), Ok(Direction::Vertical));
    }
    #[test]
    fn is_diagonal_five_in_a_row() {
        let mut board = Board::new(15);
        // A diagonal is '\'
        for pos in [2u32 + 7*15, 3u32 + 8*15, 4u32 + 9*15, 5u32 + 10*15].iter() {
            board.set_point(Point::from_1d(*pos, 15), Stone::Black);
        }

        for pos in [9u32 + 0*15, 10u32 + 1*15, 11u32 + 2*15, 13u32 + 4*15].iter() {
            board.set_point(Point::from_1d(*pos, 15), Stone::White);
        }
        let p1 = BoardMarker { point: Point::from_1d(11*15+6, 15), color: Stone::Black, comment: None };
        let p2 = BoardMarker { point: Point::from_1d(12+3*15, 15), color: Stone::White, comment: None };

        println!("\n{}\nChecks; {:?} and {:?}",
                 board.board, p1, p2);

        assert_eq!(is_line(&board, p1), Ok(Direction::Diagonal));
        assert_eq!(is_line(&board, p2), Ok(Direction::Diagonal));
    }
    #[test]
    fn is_anti_diagonal_five_in_a_row() {
        let mut board = Board::new(15);
        for pos in [6u32+6*15,5u32+7*15, 4u32+8*15, 3u32+9*15].iter() {
            board.set_point(Point::from_1d(*pos, 15), Stone::Black);
        }

        let p1 = BoardMarker { point: Point::from_1d(2u32+10*15, 15), color: Stone::Black, comment: None };

        println!("\n{}\nChecks; {:?}",
                 board.board, p1);

        assert_eq!(is_line(&board, p1), Ok(Direction::AntiDiagonal));
    }
}
