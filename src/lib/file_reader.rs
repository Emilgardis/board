#![feature(io)]
use std::io;
use std::io::prelude::*;
use std::path::{Path, Prefix};
use std::fs::{File};
use std::error::Error;

use board_logic::{Board, BoardMarker, Stone, Point};

/// Describes the file
pub enum FileType{
    /// `.pos` file.
    ///
    /// `.pos` files seems to always assume a field of size 15*15
    /// # Layout
    /// 0: N moves
    /// 1: Black move #1
    /// 2: White move #2
    /// N: Last move
    /// 
    Pos,
    /// `.lib` file.
    ///
    /// This is not supported yet as I have not reversed the protocol yet.
    Lib,
}

impl FileType{
    fn new(path: &Path) -> Option<FileType> {
        match path.extension() {
            Some(pos) if (pos == "pos") => Some(FileType::Pos),
            Some(lib) if (lib == "lib") => Some(FileType::Lib),
            Some(_) => None,
            None => None,
        }
    }
}
pub fn open_file_as_board(path: &Path) -> Option<Board> {
    let display = path.display();
    let filetype = FileType::new(path);
    let mut file = match File::open(&path){
        // Should probably return a none. Or change from Option to Result
        Err(why) => panic!("couldn't open {}: {}", display, Error::description(&why)),
        Ok(file) => file,
    };

    match filetype { 
        Some(FileType::Pos) => {
            let mut board = Board::new(15);
            for (index, pos) in file.bytes().skip(1).enumerate() { // First value should always be the number of moves.
                board.set_point(Point::from_1d(pos.ok().unwrap() as u32, 15), if index % 2 == 0 {Stone::Black} else {Stone::White});
            }
            return Some(board);
        },
        Some(FileType::Lib) => {
            unimplemented!();
        },
        _ => unimplemented!(),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use board_logic as bl;

    #[test]
    fn open_pos_file(){
        let file = Path::new("examplefiles/example.pos");
        let mut board = open_file_as_board(file).unwrap();
        println!("\n{}", board.board);
    }
}
