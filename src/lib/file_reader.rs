#![feature(io)]
use std::io;
use std::io::prelude::*;
use std::path::{Path, Prefix};
use std::fs::{File};
use std::error::Error;

use board_logic::{Board, BoardMarker, Stone, Point};

/// Describes the file
pub enum FileType{
    /// Generic Renju _.pos_ file.
    ///
    /// These files seems to always assume a field of size 15*15
    /// # Layout in binary
    /// * 0: N moves
    /// * 1: Black move #1
    /// * 2: White move #2
    /// * N: Last move
    /// 
    Pos,
    /// RenLib, _.lib_ file.
    ///
    /// This is not supported yet as I have not fully reversed the protocol yet.
    /// 
    /// # About
    ///
    /// _.lib_ files are most used for it's excellent support for saving multiple games of one play. This enables 
    /// a way to do analysis. 
    ///
    /// #### Example
    /// The move sequence is H8 I9 G7 H7 D11 etc. We want to analyse and document what would
    /// have happened if we played H6 instead of H7 on move #4. So we would go back to move #3 and continue from there. Now,
    /// when walking through the game from the beginning, we will see that there are 2 choices 
    /// highlighted on move #4, we have created a _sub game_.
    /// Clicking on either will continue this pattern of finding sub-games like before, repeating itself if needed. If
    /// you don't choose any of those choices it makes a new sub-game.
    /// 
    /// This is why RenLib's _.lib_ format is so praised in the renju community.
    ///
    /// ...* _.lib_ files come in two variations, either as Libraries or as Positions. I have yet
    /// to identify what the signature is for both.
    ///
    /// ## Known:
    /// * Libraries seem to be saved with the signature 0x40 ("@") some where at the end.
    /// Positions seems to be stored as two bytes. With 0x78 being the middle.
    /// With what I know so far, after 10 "0xFF", the first move is stored. Then if the next byte is "0x00",
    /// continue and repeat. How sub-games are stored will soon be cracked.
    ///     
    ///     This is the layout:
    ///     
    ///          0: . . . . . . . . . . . . . . .  
    ///          1: . . . . . . . . . . . . . . . 
    ///          2: . . . . . . . . . . . . . . . 
    ///          3: . . . . . . . . . . . . . . . 
    ///          4: . . . O . . . . . . . . . . . 
    ///          5: . . . . . . . . . . . . . . . 
    ///          6: . . . . . . . . . . . . . . . 
    ///          7: . . . . . . . X . . . . . . . 
    ///          8: . . . . . . . . . . . . . . . 
    ///          9: . . . . . . . . . . . . . . . 
    ///          A: . . . . . . . . . . . . . . . 
    ///          B: . . . . . . . . . . . . . . . 
    ///          C: . . . . . . . . . . . . . . . 
    ///          D: . . . . . . . . . . . . . . . 
    ///          E: . . . . . . . . . . . . . . . 
    ///             1 2 3 4 5 6 7 8 9 A B C D E F  
    ///
    /// The _O_ is on `0x44`, the _X_ is on `0x78` (the middle)
    /// 
    /// *    Libraries usually have "@" at the end, this
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
