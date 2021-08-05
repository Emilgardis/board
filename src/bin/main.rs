extern crate renju;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate rustyline;

use clap::{App, Arg};
use renju::errors::*;
use error_chain::ChainedError;

use std::path::Path;

use std::io::Write;
use std::env;
use renju::file_reader::{open_file, open_file_legacy};
use renju::board_logic;
use renju::move_node::{MoveIndex, MoveGraph};
quick_main!(run);

fn run() -> Result<i32> {
    let matches = App::new("renju-open")
        .version(crate_version!())
        .arg(Arg::with_name("file")
                 .index(1)
                 .help("File to read from")
                 .required(true))
        .arg(Arg::with_name("output")
                 .short("o")
                 .help("File to output to"))
        .arg(Arg::with_name("legacy")
                 .short("l")
                 .takes_value(false)
                 .help("Parse lib with legacy code"))
        .get_matches();

    let path = Path::new(matches.value_of("file").unwrap());
    println!("File: {:?}\n", path);
    let graph = if !matches.is_present("legacy") {
        open_file(&path)
            .chain_err(|| format!("while parsing file {:?}", path))?
    } else {
        open_file_legacy(&path)
            .chain_err(|| format!("while parsing file {:?} in lagacy mode", path))?
    };
    println!("{:?}", graph);
    //let mut file = OpenOptions::new().write(true).create(true).open(format!("{}.dot",path.file_stem().unwrap().to_str().unwrap())).expect("Couldn't create .dot file");
    //write!(file, "{:?}", graph).chain_err(|| "while writing to file");
    let mut rl = rustyline::Editor::<()>::new();

    loop {
        let read = rl.readline(">> ");
        //println!("{:?}", read);
        match read {
            Ok(ref empty) if &empty == &"" => {
                println!("Exit with quit/q or ctrl+d");
            }
            Ok(ref g) if &g == &"graph" || &g == &"g" => {
                println!("{:?}", graph);
            }
            // Should be regex or match, quiz should not match
            Ok(ref quit) if quit.to_lowercase().starts_with("q") => {
                return Ok(0);
            }
            Ok(line) => {
                match line.parse() {
                    Ok(node) => {
                        match traverse(&graph, node) {
                            Ok(board) => {
                                println!("{}", board.board);
                                if let Some(last_point) = board.last_move {
                                    match board.get(last_point) {
                                        Some(&board_logic::BoardMarker {
                                                  comment: ref comment_opt, ..
                                              }) => {
                                            if let &Some(ref comment) = comment_opt {
                                                println!("{}", comment)
                                            }
                                        }
                                        None => panic!("Move not found"),
                                    }
                                }
                            }
                            Err(e) => println!("{}", e.display_chain()),
                        }
                    }
                    Err(e) => {
                        println!("{}", e.display_chain());
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Eof) => return Ok(0),
            _ => {}
        }
    }
    unreachable!()
}

fn traverse(graph: &MoveGraph, index: MoveIndex) -> Result<board_logic::Board> {
    graph.as_board(index)
}
