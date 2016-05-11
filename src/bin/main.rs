extern crate renju;
use std::path::Path;
use renju::file_reader::open_file;
use std::env;
fn main(){
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        panic!("Please provide a file.")
    }
    for file in args {
        let graph = open_file(&Path::new(&file));
        println!("File: {}\n{:?}", file, graph);
    }
}
