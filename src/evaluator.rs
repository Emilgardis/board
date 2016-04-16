use board_logic::{Board, BoardMarker, Point, Stone};


#[derive(Debug, PartialEq)]
pub enum Direction {
    Horizontal,
    Vertical,
    Diagonal, // ´\´
    AntiDiagonal, // ´/´
}

pub fn is_five(board: &Board, marker: BoardMarker) -> Option<Direction> {
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
            if board.getxy(i, marker.point.y).unwrap().color == marker.color {
                n_line += 1;
            } else {
                debug!("\tEnd: {}", i);
                break 'left;
            }
        }

        if n_line >= 4 {
            debug!("Horizontal Line length: {}", n_line);
            return Some(Direction::Horizontal);
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
            return Some(Direction::Vertical);
        }
        n_line = 0;
    }
    Option::None
}

#[cfg(test)]
mod tests {
    use super::*;
    use board_logic::{Board, BoardMarker, Stone, Point};

    #[test]
    #[ignore]
    fn check_if_illegal_move() {
        let positions: Vec<Point> = [7 * 15 + 7, 7 * 15 + 6, 6 * 15 + 6, 5 * 15 + 7]
                .iter().map(|d| Point::from_1d(*d, 15)).collect();
        let illegal = Point::from_1d(7 * 15 + 5, 15);
        println!("{:?}", illegal);
    }

    #[test]
    fn is_horizontal_five_in_a_row() {
        let mut board = Board::new(15);
        let y = 7u32;
        let p1 = BoardMarker { point: Point::new(4, y), color: Stone::Black };
        for x in (0..4) {
            board.set_point(Point::new(x, y), Stone::Black);
        }

        let p2 = BoardMarker { point: Point::new(8, y + 2), color: Stone::White };
        for x in (7..12).filter(|x| *x != 8) {
            board.set_point(Point::new(x, y + 2), Stone::White);
        }
        println!("test:is_horizontal_five_in_a_row:\n{}\nChecks,{:?} and {:?}",
                 board.board, p1, p2);
        
        assert_eq!(is_five(&board, p1), Some(Direction::Horizontal));
        assert_eq!(is_five(&board, p2), Some(Direction::Horizontal));
    }

    #[test]
    fn is_vertical_five_in_a_row() {  
        let mut board = Board::new(15);
        let x = 7u32;
        let p1 = BoardMarker { point: Point::new(x, 4), color: Stone::Black };
        for y in (0..4) {
            board.set_point(Point::new(x, y), Stone::Black);
        }

        let p2 = BoardMarker { point: Point::new(x + 2, 8), color: Stone::White };
        for y in (7..12).filter(|y| *y != 8) {
            board.set_point(Point::new(x+2, y), Stone::White);
        }
        println!("test:is_horizontal_five_in_a_row:\n{}\nChecks,{:?} and {:?}",
                 board.board, p1, p2);
        
        assert_eq!(is_five(&board, p1), Some(Direction::Vertical));
        assert_eq!(is_five(&board, p2), Some(Direction::Vertical));
    }

}
