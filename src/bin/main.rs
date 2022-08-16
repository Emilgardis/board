use clap::{App, Arg};
use renju::errors::*;

use std::path::Path;

use color_eyre::eyre::WrapErr;
use renju::board_logic;
use renju::file_reader::open_file;
use renju::move_node::{MoveGraph, MoveIndex};

fn main() -> Result<(), color_eyre::Report> {
    let _ = dotenv::dotenv();
    color_eyre::install()?;
    renju::util::build_logger()?;
    let matches = App::new("renju-open")
        .arg(
            Arg::new("file")
                .index(1)
                .help("File to read from")
                .required(true),
        )
        .arg(Arg::new("output").short('o').help("File to output to"))
        .get_matches();

    let path = Path::new(matches.value_of("file").unwrap());
    tracing::info!("File: {:?}", path);
    let graph = open_file(path).wrap_err_with(|| format!("while parsing file {:?}", path))?;

    eprintln!("{:?}", graph);
    //let mut file = OpenOptions::new().write(true).create(true).open(format!("{}.dot",path.file_stem().unwrap().to_str().unwrap())).expect("Couldn't create .dot file");
    //write!(file, "{:?}", graph).chain_err(|| "while writing to file");
    let mut rl = rustyline::Editor::<()>::new()?;

    loop {
        let read = rl.readline(">> ");
        //tracing::info!("{:?}", read);
        match read {
            Ok(ref empty) if empty.is_empty() => {
                tracing::info!("Exit with quit/q or ctrl+d");
            }
            Ok(ref g) if g == "graph" || g == "g" => {
                tracing::info!("{:?}", graph);
            }
            // Should be regex or match, quiz should not match
            Ok(ref quit) if quit.to_lowercase().starts_with('q') => {
                return Ok(());
            }
            Ok(line) => {
                let node = line.parse()?;
                let board = traverse(&graph, node)?;
                eprintln!("{}", board.board);
                if let Some(last_point) = board.last_move {
                    if let Some(&board_logic::BoardMarker {
                        multiline_comment: ref multiline_comment_opt,
                        oneline_comment: ref oneline_comment_opt,
                        ..
                    }) = board.get(last_point)
                    {
                        if let Some(ref comment) = oneline_comment_opt {
                            tracing::info!("{}", comment)
                        }
                        if let Some(ref comment) = multiline_comment_opt {
                            tracing::info!("{}", comment)
                        }
                    } else {
                        panic!("Move not found")
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Eof) => return Ok(()),
            _ => {}
        }
    }
}

fn traverse(graph: &MoveGraph, index: MoveIndex) -> Result<board_logic::Board, ParseError> {
    graph.as_board(&index)
}
