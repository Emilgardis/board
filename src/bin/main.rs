extern crate renju;
#[macro_use]
extern crate clap;
extern crate rustyline;

use clap::{App, Arg};
use renju::errors::*;

use std::path::Path;

use color_eyre::eyre::WrapErr;
use renju::board_logic;
use renju::file_reader::{open_file, open_file_legacy};
use renju::move_node::{MoveGraph, MoveIndex};
use std::env;

fn main() -> Result<(), color_eyre::Report> {
    let _ = dotenv::dotenv();
    tracing_subscriber::fmt::init();
    let matches = App::new("renju-open")
        .version(crate_version!())
        .arg(
            Arg::with_name("file")
                .index(1)
                .help("File to read from")
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .help("File to output to"),
        )
        .arg(
            Arg::with_name("legacy")
                .short("l")
                .takes_value(false)
                .help("Parse lib with legacy code"),
        )
        .get_matches();

    let path = Path::new(matches.value_of("file").unwrap());
    tracing::info!("File: {:?}\n", path);
    let graph = if !matches.is_present("legacy") {
        open_file(path).wrap_err_with(|| format!("while parsing file {:?}", path))?
    } else {
        open_file_legacy(path)
            .wrap_err_with(|| format!("while parsing file {:?} in lagacy mode", path))?
    };
    eprintln!("{:?}", graph);
    //let mut file = OpenOptions::new().write(true).create(true).open(format!("{}.dot",path.file_stem().unwrap().to_str().unwrap())).expect("Couldn't create .dot file");
    //write!(file, "{:?}", graph).chain_err(|| "while writing to file");
    let mut rl = rustyline::Editor::<()>::new();

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
                        comment: ref comment_opt,
                        ..
                    }) = board.get(last_point)
                    {
                        if let Some(ref comment) = comment_opt {
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
    graph.as_board(index)
}
