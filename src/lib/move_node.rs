use daggy::{Dag};

#[test]
fn does_it_work() {
    use board_logic::*;
    let mut dag = Dag::<BoardMarker, u32, u32>::new();
    let first_move = BoardMarker{pos: Point{x:7, y:7}};
}
