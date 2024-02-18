//! Used for reading files.
//!
//! Currently only supports _.pos_ and _.lib_ (`RenLib`) files of version 3.04+.

use std::fs::File;
use std::path::Path;

use crate::board::{Board, BoardMarker, MoveIndex, Point, Stone};
use crate::errors::ParseError;

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
    fn new(path: &Path) -> Option<Self> {
        match path.extension() {
            Some(pos) if (pos == "pos") => Some(Self::Pos),
            Some(lib) if (lib == "lib") => Some(Self::Lib),
            Some(_) => None,
            None => None,
        }
    }
}

pub enum FileErr {
    ParseError,
}

#[tracing::instrument(fields(filetype))]
pub fn open_file_path(path: &Path) -> Result<Board, color_eyre::Report> {
    let mut board = Board::new();

    let _display = path.display();
    let filetype = FileType::new(path);
    tracing::Span::current().record("filetype", tracing::field::debug(&filetype));
    let file: File = File::open(path)?;
    // XXX: This gives a massive speedup.
    let buffered = std::io::BufReader::new(file);
    tracing::trace!("file opened");
    read_bytes(buffered, filetype.as_ref(), &mut board)?;
    Ok(board)
}

#[tracing::instrument(skip(bytes, board))]
pub fn read_bytes(
    bytes: impl std::io::Read,
    filetype: Option<&FileType>,
    board: &mut Board,
) -> Result<(), color_eyre::Report> {
    match filetype {
        Some(FileType::Pos) => {
            let mut sequence: Vec<BoardMarker> = Vec::new();
            for (index, pos) in bytes.bytes().skip(1).enumerate() {
                // First value should always be the number of moves.
                sequence.push(BoardMarker::new(
                    Point::from_1d(u32::from(pos?), 15),
                    if index % 2 == 0 {
                        Stone::Black
                    } else {
                        Stone::White
                    },
                ));
            }
            let root = board.get_root();
            let mut latest: MoveIndex = board.insert_move(root, sequence[0].clone());
            for marker_move in sequence.into_iter().skip(1) {
                latest = board.insert_move(latest, marker_move)
            }
        }
        Some(FileType::Lib) => renlib::parse_lib(bytes, board)?,
        _ => return Err(ParseError::NotSupported.into()),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    use crate::board as mn;

    #[test]
    fn open_pos_file() {
        let file = Path::new("examplefiles/example.pos");
        let graph: mn::Board = match open_file_path(file) {
            Ok(gr) => gr,
            Err(desc) => panic!("{:?}", desc),
        };
        tracing::info!("\n{:?}", graph);
    }
    #[test]
    fn open_lib_file() {
        let file = Path::new("examplefiles/lib_documented.lib");
        let graph: mn::Board = match open_file_path(file) {
            Ok(gr) => gr,
            Err(desc) => panic!("err, {:?}", desc),
        };
        tracing::info!("\n{:?}", graph);
        // panic!("Intended!");
    }
}
