#![allow(clippy::result_unit_err)]
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

use super::board_logic::{BoardArr, BoardMarker, Stone};

use std::collections::BTreeSet;
use std::slice::Iter;
#[derive(Debug, Copy, Clone)]
pub enum Direction {
    /// Horizontal line `|`
    Horizontal,
    /// Vertical line `-`
    Vertical,
    /// Diagonal line `/` or `\`
    Diagonal {
        /// Top or bottom. bottom = `/`, top = `\`
        bottom: bool,
    },
}
impl Direction {
    pub const fn directions() -> [Direction; 4] {
        [
            Direction::Horizontal,
            Direction::Vertical,
            Direction::Diagonal { bottom: false },
            Direction::Diagonal { bottom: true },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        board::{BoardArr, BoardMarker, Point, Stone},
        p,
    };

    fn log() {
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();
    }

    #[test]
    #[ignore]
    fn check_if_illegal_move() {
        log();
        let mut board = BoardArr::new(15);
        for pos in p![[H, 8], [G, 8], [G, 9], [H, 10]] {
            board.set_point(pos, Stone::Black);
        }

        let illegal = p![F, 8];
        tracing::info!("{:?}", illegal);
    }

    #[test]
    fn is_horizontal_five_in_a_row() {
        log();
        let mut board = BoardArr::new(15);
        let y = 7u32;
        let p1 = BoardMarker::new(Point::new(4, y), Stone::Black);
        for x in 0..4 {
            board.set_point(Point::new(x, y), Stone::Black);
        }

        let p2 = BoardMarker::new(Point::new(8, y + 2), Stone::White);
        for x in (7..12).filter(|x| *x != 8) {
            board.set_point(Point::new(x, y + 2), Stone::White);
        }
        tracing::info!("\n{}\nChecks,{:?} and {:?}", board, &p1, p2);
        //assert!(is_five_dir(&board, &p1, Direction::Horizontal).unwrap());
        //assert!(is_five_dir(&board, &p2, Direction::Horizontal).unwrap());
        //assert_eq!(line(&board, &p1), Ok(Direction::Horizontal));
        // assert_eq!(is_line(&board, &p2).unwrap(), Direction::Horizontal);
    }

    #[test]
    fn is_vertical_five_in_a_row() {
        log();
        let mut board = BoardArr::new(15);
        let x = 7u32;
        let p1 = BoardMarker::new(Point::new(x, 4), Stone::Black);
        for y in 0..4 {
            board.set_point(Point::new(x, y), Stone::Black);
        }

        let p2 = BoardMarker::new(Point::new(x + 2, 8), Stone::White);
        for y in (7..12).filter(|y| *y != 8) {
            board.set_point(Point::new(x + 2, y), Stone::White);
        }
        tracing::info!("\n{}\nChecks; {:?} and {:?}", board, &p1, p2);

        //assert!(is_five_dir(&board, &p1, Direction::Vertical).unwrap());
        //assert!(is_five_dir(&board, &p2, Direction::Vertical).unwrap());
        //assert_eq!(is_line(&board, &p1), Ok(Direction::Vertical));
        //assert_eq!(is_line(&board, &p2), Ok(Direction::Vertical));
    }

    #[test]
    fn is_diagonal_five_in_a_row() {
        log();
        let mut board = BoardArr::new(15);
        // A diagonal is '\'
        for pos in &[2u32 + 7 * 15, 3u32 + 8 * 15, 4u32 + 9 * 15, 5u32 + 10 * 15] {
            board.set_point(Point::from_1d(*pos, 15), Stone::Black);
        }
        #[allow(clippy::identity_op)]
        for pos in &[
            9u32, /*+ 0 * 15*/
            10u32 + 1 * 15,
            11u32 + 2 * 15,
            13u32 + 4 * 15,
        ] {
            board.set_point(Point::from_1d(*pos, 15), Stone::White);
        }
        let p1 = BoardMarker::new(Point::from_1d(11 * 15 + 6, 15), Stone::Black);
        let p2 = BoardMarker::new(Point::from_1d(12 + 3 * 15, 15), Stone::White);

        tracing::info!("\n{}\nChecks; {:?} and {:?}", board, &p1, p2);

        //assert!(is_five_dir(&board, &p1, Direction::Diagonal).unwrap());
        //assert!(is_five_dir(&board, &p2, Direction::Diagonal).unwrap());
        //assert_eq!(is_line(&board, &p1), Ok(Direction::Diagonal));
        //assert_eq!(is_line(&board, &p2), Ok(Direction::Diagonal));
    }
    #[test]
    fn is_anti_diagonal_five_in_a_row() {
        log();
        let mut board = BoardArr::new(15);
        for pos in &[6u32 + 6 * 15, 5u32 + 7 * 15, 4u32 + 8 * 15, 3u32 + 9 * 15] {
            board.set_point(Point::from_1d(*pos, 15), Stone::Black);
        }

        let p1 = BoardMarker::new(Point::from_1d(2u32 + 10 * 15, 15), Stone::Black);

        tracing::info!("\n{}\nChecks; {:?}", board, &p1);
        //assert!(is_five_dir(&board, &p1, Direction::AntiDiagonal).unwrap());
        //assert_eq!(is_line(&board, &p1), Ok(Direction::AntiDiagonal));
    }
}
