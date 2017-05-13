#![feature(question_mark, slice_patterns)]
pub mod board_logic;
pub mod evaluator;
pub mod file_reader;
pub mod move_node;
pub mod errors;
#[macro_use]
extern crate slog;
extern crate daggy;
extern crate num;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate error_chain;
