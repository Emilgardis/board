extern crate renju;
extern crate env_logger;

use std::path::Path;
use renju::file_reader::open_file;
use std::env;
fn main(){
    env_logger::init().unwrap();

    let mut args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Please provide a file.")
    }
    args.remove(0);
    for file in args {
        println!("File: {}", file);
        let graph = open_file(&Path::new(&file));
        println!("{:?}",graph);
    }
}
