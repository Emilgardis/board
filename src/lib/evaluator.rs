//! This is the evauluator for checking what condition a certain move creates. 
//! 
//! It is either an illegal move, (i.e) black makes a three-three, four-four or an overline. Or it
//! is an overline (win for white), five (win for black and white), four (can become a five) or an three
//! (can become a four). A four and a three can also be either in two states, open or closed. An
//! open three will always be able to become a four, an open four will always become an five.
//! These states are easy to check for white, but it becomes trickier when the move is done by
//! black. Black cannot place a stone that actively is part of any of the illegal moves, but a
//! three-four can become a four-four (e.g).
//!
//! # Implementation.
//!

use board_logic::{BoardMarker, Board, Stone};

pub enum Direction{
    Horizontal,
    Vertical,
    Diagonal,
    AntiDiagonal,

}
pub struct Line(Vec<i8>);

pub fn line(board: &Board, marker: BoardMarker, direction: Direction) -> Result<Vec<i8>, ()>{
    match direction {
        Direction::Horizontal => {
            let mut line: Vec<i8> = Vec::new();
            'right: for i in marker.point.x+1..board.boardsize {
                match board.getxy(i, marker.point.y) {
                    Some(other_marker) => {
                        debug!("\t{:?}", other_marker);
                        if other_marker.color == marker.color {
                            line.push((i-marker.point.x) as i8);
                        }
                    },
                    None => return Err(()),
                }
            }
            'left: for i in (0..marker.point.x).rev() {
                match board.getxy(i, marker.point.y) {
                    Some(other_marker) => {
                        debug!("\t{:?}", other_marker);
                        if other_marker.color == marker.color {
                            line.push(((i as i8)-marker.point.x as i8));
                        }
                    },
                    None => return Err(()),
                }
            }
            Ok(line)
        },
        Direction::Vertical => {
            let mut line: Vec<i8> = Vec::new();
            'down: for i in marker.point.y+1..board.boardsize {
                match board.getxy(marker.point.x, i) {
                    Some(other_marker) => {
                        debug!("\t{:?}", other_marker);
                        if other_marker.color == marker.color {
                            line.push((i-marker.point.y) as i8);
                        }
                    },
                    None => return Err(()),
                }
            }
            'up: for i in (0..marker.point.y).rev() {
                match board.getxy(marker.point.x, i) {
                    Some(other_marker) => {
                        debug!("\t{:?}", other_marker);
                        if other_marker.color == marker.color {
                            line.push(((i as i8)-marker.point.y as i8));
                        }
                    },
                    None => return Err(()),
                }
            }
            Ok(line)
        },
        Direction::Diagonal => {
            let mut line: Vec<i8> = Vec::new();
            'diag_down: for i in 1..board.boardsize+1 {
                match board.getxy(marker.point.x+1, marker.point.y+1) {
                    Some(other_marker) => {
                        debug!("\t{:?}", other_marker);
                        if other_marker.color == marker.color {
                            line.push(i as i8);
                        }
                    },
                    None => break 'diag_down, // We have hit the border. Don't err, this is expected.
                }
            }
            'diag_up: for i in 1..board.boardsize+1 {
                match board.get_i32xy((marker.point.x as i32) - (i as i32), (marker.point.y as i32) - (i as i32)) {
                    Some(other_marker) => {
                        debug!("\t{:?}", other_marker);
                        if other_marker.color == marker.color {
                            line.push(-(i as i8));
                        }
                    },
                    None => break 'diag_up,
                }
            }
            Ok(line)
        },
        _ => Err(()),
    }
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
        let p1 = BoardMarker::new(Point::new(4, y), Stone::Black);
        for x in 0..4 {
            board.set_point(Point::new(x, y), Stone::Black);
        }

        let p2 = BoardMarker::new(Point::new(8, y + 2), Stone::White);
        for x in (7..12).filter(|x| *x != 8) {
            board.set_point(Point::new(x, y + 2), Stone::White);
        }
        println!("\n{}\nChecks,{:?} and {:?}",
                 board.board, p1, p2);
        println!("{:?}", line(&board, p1, Direction::Horizontal));
        println!("{:?}", line(&board, p2, Direction::Horizontal));
        //assert_eq!(line(&board, p1), Ok(Direction::Horizontal));
        // assert_eq!(is_line(&board, p2).unwrap(), Direction::Horizontal);
    }

    #[test]
    fn is_vertical_five_in_a_row() {  
        let mut board = Board::new(15);
        let x = 7u32;
        let p1 = BoardMarker::new(Point::new(x, 4), Stone::Black);
        for y in 0..4 {
            board.set_point(Point::new(x, y), Stone::Black);
        }

        let p2 = BoardMarker::new(Point::new(x + 2, 8), Stone::White);
        for y in (7..12).filter(|y| *y != 8) {
            board.set_point(Point::new(x+2, y), Stone::White);
        }
        println!("\n{}\nChecks; {:?} and {:?}",
                 board.board, p1, p2);
        
        println!("{:?}", line(&board, p1, Direction::Vertical));
        println!("{:?}", line(&board, p2, Direction::Vertical));
        //assert_eq!(is_line(&board, p1), Ok(Direction::Vertical));
        //assert_eq!(is_line(&board, p2), Ok(Direction::Vertical));
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
        let p1 = BoardMarker::new(Point::from_1d(11*15+6, 15), Stone::Black);
        let p2 = BoardMarker::new(Point::from_1d(12+3*15, 15), Stone::White);

        println!("\n{}\nChecks; {:?} and {:?}",
                 board.board, p1, p2);
        
        println!("{:?}", line(&board, p1, Direction::Diagonal));
        println!("{:?}", line(&board, p2, Direction::Diagonal));
        //assert_eq!(is_line(&board, p1), Ok(Direction::Diagonal));
        //assert_eq!(is_line(&board, p2), Ok(Direction::Diagonal));
    }
    #[test]
    fn is_anti_diagonal_five_in_a_row() {
        let mut board = Board::new(15);
        for pos in [6u32+6*15,5u32+7*15, 4u32+8*15, 3u32+9*15].iter() {
            board.set_point(Point::from_1d(*pos, 15), Stone::Black);
        }

        let p1 = BoardMarker::new(Point::from_1d(2u32+10*15, 15), Stone::Black);

        println!("\n{}\nChecks; {:?}",
                 board.board, p1);

        //assert_eq!(is_line(&board, p1), Ok(Direction::AntiDiagonal));
    }
}
