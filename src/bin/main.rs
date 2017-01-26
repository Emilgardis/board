extern crate renju;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
use clap::{App, Arg};
use renju::errors::{Result, ResultExt};

use std::path::Path;
use std::fs::OpenOptions;
use std::io::Write;
use std::env;
use renju::file_reader::open_file;

quick_main!(run);

fn run() -> Result<()>{
    let matches = App::new("renju-open")
        .version(crate_version!())
        .arg(Arg::with_name("file")
             .index(1)
             .help("File to read from")
             .required(true)
             )
        .arg(Arg::with_name("output")
             .short("o")
             .help("File to output to")
             )
        .get_matches();

    let path = Path::new(matches.value_of("file").unwrap());
    println!("File: {:?}\n", path);
    let graph = open_file(&path).chain_err(|| format!("while parsing file {:?}", path))?;
    println!("{:?}", graph);
    //let mut file = OpenOptions::new().write(true).create(true).open(format!("{}.dot",path.file_stem().unwrap().to_str().unwrap())).expect("Couldn't create .dot file");
    //write!(file, "{:?}", graph).chain_err(|| "while writing to file");
    Ok(())
}
