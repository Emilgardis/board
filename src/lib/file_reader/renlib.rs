//! Functions for handling renlib files.
use std::str;
use errors::*;

use board_logic::{BoardMarker, Stone, Point};
use move_node::{MoveGraph, MoveIndex};


pub fn parse_lib(file_u8: Vec<u8>) -> Result<MoveGraph> {
    unimplemented!()
}

pub fn validate_lib(header: &[u8;20]) -> bool {
   unimplemented!() 
}
