use crate::{move_node::{MoveGraph, MoveIndex}, board_logic::BoardMarker};

use super::Version;
pub use super::{Command, CommandVariant};

pub fn parse_v3x(
    mut file: impl std::io::BufRead,
    _version: Version,
) -> Result<MoveGraph, color_eyre::eyre::Report> {
    let mut graph = MoveGraph::new();
    let mut buffer = Vec::with_capacity(1024);
    let _size = file.read_to_end(&mut buffer)?;
    let mut _cur_index: Option<MoveIndex> = None;
    let mut _next_index: Option<MoveIndex> = None;
    let mut _cur_root: Option<MoveIndex> = None;
    let mut index = 0;
    // No clue what this is
    let mut _number = 0;

    if _cur_root.is_none() {
        let _ind = graph.new_root(BoardMarker::null_move());
        _cur_index = Some(_ind);
        _cur_root = Some(_ind);
    } else {
        unimplemented!("creating from existing graph is not implemented")
    }

    let check_root = true;
    loop {
        let marker = BoardMarker::from_pos_info(buffer[index], buffer[index + 1] as u16)?;
        index += 2;

        if check_root && marker.point.is_null {
            // Skip root.
        } else if marker.point.is_valid() {
            color_eyre::eyre::bail!("invalid point, for some reason");
        } else {
            _number += 1;
            _next_index = graph.get_variant(_cur_index.as_ref(), &marker.point)
        }
    }
    // let mut iter = file.bytes().peekable();
    // if iter
    //     .peek()
    //     .filter(|&r| match r {
    //         Ok(y) => &0x00 == y,
    //         Err(_) => false,
    //     })
    //     .is_some()
    // {
    //     // No move start, ignore.
    //     // TODO: Is this valid?
    //     //iter.next();
    //     tracing::info!("No start move");
    // }
    // // It should just work to do this sequentially and use move_graph functions, let's try that
    // while let Some(byte) = iter.next() {
    //     let byte = byte?;
    //     let mut _cur_marker: Option<BoardMarker> = None;
    //     let span = tracing::debug_span!("moving", byte = %format!("0x{:02x}", byte));
    //     let _enter = span.enter();
    //     if iter.peek().is_none() && byte == 0x0a {
    //         // This is really wierd and shouldn't happen, will have to investigate
    //         break;
    //     }
    //     let command = Command(iter.next().ok_or_else(|| {
    //         ParseError::Other("Expected a command byte, got nothing".to_string())
    //     })??);
    //     let point = if let Ok(point) = byte_to_point(&byte) {
    //         point
    //     } else {
    //         tracing::debug!("Nope");
    //         Point::null()
    //     };
    //     tracing::info!(
    //         "Point: {:?} Command: ({:x}) {:?} Previous Index: {:?}",
    //         point,
    //         command.0,
    //         command.get_all(),
    //         cur_index
    //     );
    //     let stone = if let Some(cur_index) = cur_index {
    //         if graph.moves_to_root(cur_index) % 2 == 1 {
    //             Stone::Black
    //         } else {
    //             Stone::White
    //         }
    //     } else {
    //         Stone::Black
    //     };

    //     if command.is_extension() {
    //         let extension = (
    //             iter.next().transpose()?.unwrap(),
    //             iter.next().transpose()?.unwrap(),
    //         );
    //         tracing::info!("Extension: {:?}", extension);
    //     }

    //     _cur_marker = Some(BoardMarker::new(point, stone));
    //     if command.is_comment() {
    //         tracing::info!("Parsing comment");
    //         // Move into functon?
    //         {
    //             let mut title = Vec::new();
    //             let mut comment = Vec::new();

    //             // while !{
    //             //     let this = &(iter.peek().ok_or_else(|| {
    //             //         ParseError::Other("File ended while parsing title".to_string())
    //             //     })?);
    //             //     match this {
    //             //         Ok(y) => &0x00 == y,
    //             //         Err(_) => false,
    //             //     }
    //             // } {
    //             //     title.push(iter.next().unwrap()?)
    //             // }

    //             while let Some(byte) = iter.next().transpose()? {
    //                 let byte = byte;
    //                 if byte == 0x00 {
    //                     break;
    //                 }
    //                 title.push(byte);
    //             }

    //             while let Some(byte) = iter.next().transpose()? {
    //                 let byte = byte;
    //                 if byte == 0x00 {
    //                     break;
    //                 }
    //                 comment.push(byte);
    //             }

    //             let title = String::from_utf8_lossy(title.as_slice());
    //             let comment = String::from_utf8_lossy(comment.as_slice());
    //             tracing::debug!(%comment, %title,);
    //             // Marker has to be something
    //             if let Some(m) = _cur_marker.as_mut() {
    //                 m.set_comment(format!("Title: {}, Comment: {}", title, comment,))
    //             }
    //         }
    //         iter.next(); // Skip the 0x00
    //     }
    //     if cur_index.is_none() {
    //         prev_index = cur_index;
    //         cur_index = Some(graph.new_root(_cur_marker.clone().unwrap()));
    //         cur_root = cur_index;
    //     } else if !(command.is_down() && command.is_right()) {
    //         prev_index = cur_index;
    //         cur_index = Some(graph.add_move(cur_index.unwrap(), _cur_marker.clone().unwrap()));
    //     }
    //     if command.is_right() && command.is_down() {
    //         //tracing::info!("Popped markeds");
    //         //graph.marked_for_branch.pop();
    //         prev_index = cur_index;
    //         // This branch leaf is alone, go down immidiatly
    //         cur_index = graph.down_to_branch(cur_index.unwrap());
    //         graph.add_move(cur_index.unwrap(), _cur_marker.unwrap());
    //     } else {
    //         if command.is_right() {
    //             prev_index = None;
    //             cur_index = graph.down_to_branch(cur_index.unwrap());
    //             tracing::info!("Branching down to, res: {:?}", cur_index);
    //         }
    //         if command.is_down() {
    //             tracing::info!(
    //                 "Marking {:?} as branch.",
    //                 prev_index.unwrap_or_else(|| cur_root.unwrap())
    //             );
    //             graph.mark_for_branch(prev_index.unwrap_or_else(|| cur_root.unwrap()));
    //         }
    //     }

    //     if command.is_no_move() {
    //         if let Some(byte) = iter.next() {
    //             let byte = byte?;
    //             if byte != 0x00 {
    //                 return Err(ParseError::Other(format!(
    //                     "Expected 0x00, got 0x{:x} while skiping for no-move",
    //                     byte
    //                 )));
    //             }
    //         }
    //     }
    // }
    Ok(graph)
}
