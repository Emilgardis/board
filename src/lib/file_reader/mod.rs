//! Used for reading files.
//!
//! Currently only supports _.pos_ and _.lib_ (RenLib) files of version 3.04+.

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use crate::board_logic::{BoardMarker, Point, Stone};
use crate::errors::*;
use crate::move_node::{MoveGraph, MoveIndex};

pub mod renlib;

/// Describes the file
#[derive(Debug)]
pub enum FileType {
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
    /// * Libraries are stored as such: HEADER n * [POS:FLAGS:STRINGS:EXTENDEDINFO]. Since **.lib** supports
    /// trees, we had to implement it [in rust too](#move_node::MoveGraph)
    /// Positions are stored in one byte. This means that 0x78 is the middle.
    ///
    ///     This is the layout for X, Y:
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
    /// * The header consists of 20 bytes:
    ///         0xFF,  'R',  'e',  'n',  'L',  'i',  'b', 0xFF, MAJV, MINV,
    ///         0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    ///
    /// This can be shown with the command `xxd -g 1 -c8 <.lib file>`
    ///
    /// Note that the minor version doesn't reflect the software version, it stands for the
    /// files "protocol" version, i.e it is updated only on a breaking change.
    /// `RenLib v37` has major 3, minor 0. (It has support for 3.4 but files seem to never get
    /// stored as such.)
    ///
    ///
    // * FIXME!
    /// These are set in RenLib/MoveNode.cpp
    ///
    ///         const DOWN        = 0x000080;
    ///         const RIGHT       = 0x000040;
    ///         const OLD_COMMENT = 0x000020;
    ///         const MARK        = 0x000010;
    ///         const COMMENT     = 0x000008;
    ///         const START       = 0x000004;
    ///         const NO_MOVE     = 0x000002;
    ///         const EXTENSION   = 0x000001;
    ///         const MASK        = 0xFFFF3F;
    ///
    ///
    ///
    /// *    See RenLib/RenLibDoc.cpp for implementation.
    Lib,
    /// Renju Database File
    ///
    /// These are generally quite large. They include multiple games, so these will really test my
    /// implementation of trees. They need support for findig comments as this is the way games are
    /// found.
    Rif,
}

impl FileType {
    fn new(path: &Path) -> Option<FileType> {
        match path.extension() {
            Some(pos) if (pos == "pos") => Some(FileType::Pos),
            Some(lib) if (lib == "lib") => Some(FileType::Lib),
            Some(_) => None,
            None => None,
        }
    }
}

pub enum FileErr {
    ParseError,
}

#[tracing::instrument]
pub fn open_file(path: &Path) -> Result<MoveGraph, color_eyre::Report> {
    let _display = path.display();
    let filetype = FileType::new(path);
    let file: File = File::open(&path)?;
    tracing::trace!(filetype = ?filetype, "file opened");

    match filetype {
        Some(FileType::Pos) => {
            tracing::info!("Opening pos file. {:?}", path);
            let mut sequence: Vec<BoardMarker> = Vec::new();
            for (index, pos) in file.bytes().skip(1).enumerate() {
                // First value should always be the number of moves.
                sequence.push(BoardMarker::new(
                    Point::from_1d(pos? as u32, 15),
                    if index % 2 == 0 {
                        Stone::Black
                    } else {
                        Stone::White
                    },
                ));
            }
            let mut root = MoveGraph::new();
            let mut latest: MoveIndex = root.new_root(sequence[0].clone());
            for marker_move in sequence.into_iter().skip(1) {
                latest = root.add_move(latest, marker_move)
            }
            Ok(root)
        }
        Some(FileType::Lib) => renlib::parse_lib(std::io::BufReader::new(file)),
        _ => Err(ParseError::NotSupported.into()),
    }
}

pub fn open_file_legacy(path: &Path) -> Result<MoveGraph, ParseError> {
    let _display = path.display();
    let filetype = FileType::new(path);
    let file: File = File::open(&path)?;

    match filetype {
        Some(FileType::Lib) => {
            let mut file_u8: Vec<u8> = Vec::new();
            for byte in file.bytes() {
                file_u8.push(byte?)
            }
            self::renlib::old::parse_lib_legacy(file_u8)
        }
        _ => Err(ParseError::NotSupported),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    use crate::move_node as mn;

    #[test]
    fn open_pos_file() {
        let file = Path::new("examplefiles/example.pos");
        let graph: mn::MoveGraph = match open_file(file) {
            Ok(gr) => gr,
            Err(desc) => panic!("{:?}", desc),
        };
        tracing::info!("\n{:?}", graph);
    }
    #[test]
    fn open_lib_file() {
        let file = Path::new("examplefiles/lib_documented.lib");
        let graph: mn::MoveGraph = match open_file(file) {
            Ok(gr) => gr,
            Err(desc) => panic!("err, {:?}", desc),
        };
        tracing::info!("\n{:?}", graph);
        // panic!("Intended!");
    }
}
