//! This test is used to check that renlib files are opened correctly and
//! that everything is stored as it should.
//! The file that should be opened is the basic multiple_nodes.lib and the
//! sample I7.lib file that is included in RenLib
//!

extern crate renju;
use renju::*;
use std::path::Path;

#[test]
#[ignore]
/// This crashes on current implementation of `move_node`.
fn large_file() {
    let graph: move_node::MoveGraph = match file_reader::open_file(Path::new(
        "tests/norelease_all_games.lib",
    )) {
        Ok(val) => val,
        Err(err) => {
            panic!("Couldn't parse file! Error: {:?}.\nPlease download all games from renju.net/media/games.php and then convert it to a .lib file in renlib and place in RenLib/norelease_all_games.lib",
                   err)
        }
    };

    tracing::info!("\n{:?}", graph);
    panic!("intended!");
}

#[ignore]
#[test]
fn null_move() {
    let graph: move_node::MoveGraph =
        match file_reader::open_file(Path::new("tests/null_move2.lib")) {
            Ok(val) => val,
            Err(err) => panic!("Couldn't parse file! Error: {:?}.", err),
        };

    tracing::info!("\n{:?}", graph);
    panic!("intended!");
}
