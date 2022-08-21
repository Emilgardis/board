use crate::board_logic::{BoardMarker, Point, Stone};

use super::Version;
pub use super::{Command, CommandVariant};

#[cfg(test)]
mod tests {
    use test_log::test;

    use crate::{board_logic::Point, p};

    use super::{
        super::{BufRead, Command, CommandVariant, Stone, Version},
        *,
    };

    fn buf(b: &'static [u8]) -> impl BufRead {
        b
    }

    fn parse_v30(bytes: &'static [u8]) -> Result<Vec<BoardMarker>, color_eyre::Report> {
        let mut bytes = buf(bytes);
        parse_v3x(&mut bytes, Version::V30)
    }

    #[test]
    fn start_move() -> Result<(), color_eyre::Report> {
        assert_eq!(
            parse_v30(&[0x78, 0x00])?,
            [BoardMarker {
                point: p![H, 8],
                color: Stone::Empty,
                oneline_comment: None,
                multiline_comment: None,
                board_text: None,
                command: Command(CommandVariant::empty()),
            },]
        );
        Ok(())
    }

    #[test]
    fn basic() -> Result<(), color_eyre::Report> {
        assert_eq!(
            parse_v30(&[
                0x78, 0x00, 0x68, 0x80, 0x66, 0x00, 0x49, 0x00, 0x58, 0x00, 0x79, 0x00, 0x69, 0x00,
                0x7A, 0x00, 0x59, 0x00, 0x4A, 0x80, 0x5A, 0x40, 0x5A, 0x40, 0x69, 0xC0, 0x8A, 0x00,
                0x69, 0x00, 0x8B, 0x00, 0x68, 0x00, 0x7B, 0x00, 0x7A, 0x00, 0x6B, 0x00, 0x58, 0x40,
            ])?,
            [
                BoardMarker {
                    point: p![H, 8],
                    command: Command(CommandVariant::empty()),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![H, 9],
                    command: Command(CommandVariant::DOWN),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![F, 9],
                    command: Command(CommandVariant::empty()),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![I, 11],
                    command: Command(CommandVariant::empty()),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![H, 10],
                    command: Command(CommandVariant::empty()),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![I, 8],
                    command: Command(CommandVariant::empty()),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![I, 9],
                    command: Command(CommandVariant::empty()),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![J, 8],
                    command: Command(CommandVariant::empty()),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![I, 10],
                    command: Command(CommandVariant::empty()),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![J, 11],
                    command: Command(CommandVariant::DOWN),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![J, 10],
                    command: Command(CommandVariant::RIGHT),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![J, 10],
                    command: Command(CommandVariant::RIGHT),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![I, 9],
                    command: Command(CommandVariant::DOWN | CommandVariant::RIGHT),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![J, 7],
                    command: Command(CommandVariant::empty()),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![I, 9],
                    command: Command(CommandVariant::empty()),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![K, 7],
                    command: Command(CommandVariant::empty()),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![H, 9],
                    command: Command(CommandVariant::empty()),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![K, 8],
                    command: Command(CommandVariant::empty()),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![J, 8],
                    command: Command(CommandVariant::empty()),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![K, 9],
                    command: Command(CommandVariant::empty()),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![H, 10],
                    command: Command(CommandVariant::RIGHT),
                    ..BoardMarker::null()
                }
            ]
        );
        Ok(())
    }

    #[test]
    fn comment() -> Result<(), color_eyre::Report> {
        assert_eq!(
            parse_v30(&[
                0x78, 0x08, 0x08, 0x54, 0x68, 0x69, 0x73, 0x20, 0x63, 0x6F, 0x6D, 0x6D, 0x65, 0x6E,
                0x74, 0x20, 0x6F, 0x6E, 0x20, 0x37, 0x38, 0x00, 0x87, 0x48, 0x08, 0x49, 0x6D, 0x20,
                0x66, 0x72, 0x6F, 0x6D, 0x20, 0x38, 0x37, 0x00, 0x0A,
            ])?,
            [
                BoardMarker {
                    point: Point::from_byte(0x78)?,
                    color: Stone::Empty,
                    oneline_comment: None,
                    multiline_comment: Some("This comment on 78".to_owned()),
                    board_text: None,
                    command: Command(CommandVariant::COMMENT)
                },
                BoardMarker {
                    point: Point::from_byte(0x87)?,
                    color: Stone::Empty,
                    oneline_comment: None,
                    multiline_comment: Some("Im from 87".to_owned()),
                    board_text: None,
                    command: Command(CommandVariant::RIGHT | CommandVariant::COMMENT)
                }
            ]
        );
        Ok(())
    }

    #[test]
    fn simple() -> Result<(), color_eyre::Report> {
        assert_eq!(
            parse_v30(&[0x78, 0x00, 0x79, 0x40])?,
            [
                BoardMarker {
                    point: p![H, 8],
                    command: Command(CommandVariant::empty()),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![I, 8],
                    command: Command(CommandVariant::RIGHT),
                    ..BoardMarker::null()
                }
            ]
        );
        Ok(())
    }

    #[test]
    fn board_marker() -> Result<(), color_eyre::Report> {
        assert_eq!(
            parse_v30(&[
                0x78, 0x00, 0x68, 0xC3, 0x00, 0x01, 0x44, 0x00, 0x77, 0xC3, 0x00, 0x01, 0x42, 0x00,
                0x79, 0xC3, 0x00, 0x01, 0x41, 0x00, 0x88, 0x43, 0x00, 0x01, 0x43, 0x00,
            ])?,
            [
                BoardMarker {
                    point: p![H, 8],
                    command: Command(CommandVariant::empty()),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![H, 9],
                    board_text: Some("D".to_owned()),
                    command: Command(
                        CommandVariant::BOARDTEXT
                            | CommandVariant::DOWN
                            | CommandVariant::RIGHT
                            | CommandVariant::NOMOVE
                            | CommandVariant::EXTENSION
                    ),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![G, 8],
                    board_text: Some("B".to_owned()),
                    command: Command(
                        CommandVariant::BOARDTEXT
                            | CommandVariant::DOWN
                            | CommandVariant::RIGHT
                            | CommandVariant::NOMOVE
                            | CommandVariant::EXTENSION
                    ),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![I, 8],
                    board_text: Some("A".to_owned()),
                    command: Command(
                        CommandVariant::BOARDTEXT
                            | CommandVariant::DOWN
                            | CommandVariant::RIGHT
                            | CommandVariant::NOMOVE
                            | CommandVariant::EXTENSION
                    ),
                    ..BoardMarker::null()
                },
                BoardMarker {
                    point: p![H, 7],
                    board_text: Some("C".to_owned()),
                    command: Command(
                        CommandVariant::BOARDTEXT
                            | CommandVariant::RIGHT
                            | CommandVariant::NOMOVE
                            | CommandVariant::EXTENSION
                    ),
                    ..BoardMarker::null()
                }
            ]
        );
        Ok(())
    }
}

pub fn parse_v3x(
    mut bytes: impl std::io::Read,
    _version: Version,
) -> Result<Vec<BoardMarker>, color_eyre::eyre::Report> {
    let mut vec = vec![];
    let mut buf: [u8; 2] = [0, 0];

    loop {
        match bytes.read_exact(&mut buf) {
            Ok(_) => (),
            Err(e) => match e.kind() {
                std::io::ErrorKind::UnexpectedEof => break,
                _ => todo!(),
            },
        }
        tracing::debug!(buf = format_args!("{:#4X?}", buf));
        let point = if buf[0] == 0x00 {
            Point::null()
        } else {
            Point::from_byte(buf[0])?
        };
        let mut mark = BoardMarker::new(point, Stone::Empty);
        let command = Command::new(u32::from(buf[1]))?;

        let command = if command.is_extension() {
            bytes.read_exact(&mut buf)?;
            tracing::debug!("extension: {:#4b}, {:#4b}", buf[0], buf[1]);
            let mut cmd = command.0.bits & 0xFF;

            cmd |= ((u32::from(buf[0]) << 8) | u32::from(buf[1])) << 8;
            Command::new(cmd)?
        } else {
            command
        };
        tracing::debug!(?mark.point, ?command, "parsed");

        if command.is_comment() {
            let (one, multi) = parse_comments(&mut bytes)?;
            mark.oneline_comment = one;
            mark.multiline_comment = multi;
            tracing::debug!(?mark.oneline_comment, ?mark.multiline_comment);
        } else if command.is_old_comment() {
            let (one, multi) = parse_old_comments(&mut bytes)?;
            mark.oneline_comment = one;
            mark.multiline_comment = multi;
            tracing::debug!(?mark.oneline_comment, ?mark.multiline_comment);
        }

        if command.is_board_text() {
            let board_text = parse_board_text(&mut bytes)?;
            mark.board_text = Some(board_text)
        }

        tracing::info!(?mark, ?command, "evaluated");
        mark.command = command;
        vec.push(mark)
    }
    Ok(vec)
}

pub fn read_text(mut bytes: impl std::io::Read) -> Result<Vec<u8>, std::io::Error> {
    // TODO: Should be moved to be initialized once
    let mut buf = vec![];

    // this cannot be a read_until, as we need to do it in chunks of two.
    let mut t_buf = [0; 2];
    loop {
        bytes.read_exact(&mut t_buf)?;
        match t_buf {
            [0, 0] => {
                buf.push(0);
            }
            s => buf.extend(s),
        }
        if t_buf.contains(&0) {
            break;
        }
    }
    assert!(buf.len() > 1);
    Ok(buf)
}

#[derive(thiserror::Error, Debug)]
pub enum ParseBoardTextError {
    #[error("read from board text buffer failed")]
    Io(#[from] std::io::Error),
}

fn parse_board_text(bytes: impl std::io::Read) -> Result<String, ParseBoardTextError> {
    // Board text is a null padded null-ending string, iff len % 2 == 1
    // so: the string "AA\0" becomes "AA\0\0"

    let buf = read_text(bytes)?;
    assert!(buf.len() > 1);
    assert!(buf.last().unwrap() == &0);

    Ok(String::from_utf8_lossy(&buf[..buf.len() - 1]).to_string())
}

#[derive(thiserror::Error, Debug)]
pub enum ParseCommentError {
    #[error("read from comment buffer failed")]
    Io(#[from] std::io::Error),
}

pub fn parse_comments(
    bytes: impl std::io::Read,
) -> Result<(Option<String>, Option<String>), ParseCommentError> {
    // The comments are either:
    //
    // oneline + 0
    // oneline + 8 + multiline + 0
    // 8 + multiline + 0
    // if the bytes are uneven, they will be padded with an extra 0, this is accounted for with out buffer read.

    let mut one = None;
    let mut multi = None;

    let buf = read_text(bytes)?;

    if &0x08 == buf.first().unwrap() {
        // FIXME: Could be empty
        multi = Some(String::from_utf8_lossy(&buf[1..buf.len() - 1]).to_string())
    } else if let Some(pos) = buf.iter().position(|b| *b == 0x08) {
        one = Some(String::from_utf8_lossy(&buf[0..pos]).to_string());
        multi = Some(String::from_utf8_lossy(&buf[(pos + 1)..buf.len() - 1]).to_string());
    } else {
        one = Some(String::from_utf8_lossy(&buf[..buf.len() - 1]).to_string());
    }

    Ok((one, multi))
}

pub fn parse_old_comments(
    bytes: impl std::io::Read,
) -> Result<(Option<String>, Option<String>), ParseCommentError> {
    let mut one = None;
    let mut multi = None;

    let buf = read_text(bytes)?
        .into_iter()
        .map(|c| match c {
            // FIXME: There has to be more like this, no?
            b'}' => 0xE5,
            b'{' => 0xE4,
            b'|' => 0xF6,
            b']' => 0xC5,
            b'[' => 0xC4,
            b'\\' => 0xD6,
            other => other,
        })
        .collect::<Vec<_>>();

    if &0x08 == buf.first().unwrap() {
        // FIXME: Could be empty
        multi = Some(String::from_utf8_lossy(&buf[1..buf.len() - 1]).to_string())
    } else if let Some(pos) = buf.iter().position(|b| *b == 0x08) {
        one = Some(String::from_utf8_lossy(&buf[0..pos]).to_string());
        multi = Some(String::from_utf8_lossy(&buf[(pos + 1)..buf.len() - 1]).to_string());
    } else {
        one = Some(String::from_utf8_lossy(&buf[..buf.len() - 1]).to_string());
    }
    Ok((one, multi))
}
