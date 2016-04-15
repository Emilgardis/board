use board_logic::{Board, BoardMarker, Point, Stone};


#[derive(Debug, PartialEq)]
pub enum Direction {
	Horizontal,
	Vertical,
	Diagonal, // ´\´
	AntiDiagonal, // ´/´
}

pub fn is_five(board: Board, pos: Point, color: Stone) -> Option<Direction> {
	// OK this will be hard, let's do it.
	// First, check if horizontal.
	let mut nLine = 1;
	for i in pos.x .. board.boardsize+1 {
		if board.getxy(i,pos.y).unwrap_or(&BoardMarker {point: Point::new(0,0), color: Stone::Empty}).color == color {
			nLine += 1;
		} else {
			break;
		}
	}
	for i in pos.x-1 .. 0 {
		if board.getxy(i,pos.y).unwrap_or(&BoardMarker {point: Point::new(0,0), color: Stone::Empty}).color == color {
			nLine += 1;
		} else {
			break;
		}
	}
	if nLine >= 5 {
		return Some(Direction::Horizontal);
	}
	Option::None
}

#[cfg(test)]
mod tests {
    use super::*;
    use board_logic::{Board, BoardMarker, Stone, Point};

    #[test]
    #[ignore]
    fn check_if_illegal_move(){
        let positions: Vec<Point> = [7*15 + 7,7*15 +6,6*15 +6, 5*15 +7].iter().map(|d| Point::from_1d(*d, 15)).collect();
        let illegal = Point::from_1d(7*15 + 5, 15);
        println!("{:?}", illegal);
    }

    #[test]
    fn is_horizontal_five_in_a_row() {
        let mut board = Board::new(15);
        let y = 7u32;
        let p = Point::new(6, y);
        for x in (0..6) {
            board.set(Point::new(x, y), Stone::Black);
        }
        assert_eq!(is_five(board,p, Stone::Black), Some(Direction::Horizontal));
        
    }
 
}
