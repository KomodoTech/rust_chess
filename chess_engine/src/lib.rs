#![forbid(unsafe_code)]
#![allow(clippy::module_inception)]
#![allow(unused)]

#[macro_use]
mod macros;

pub mod board;
pub mod castle_perm;
pub mod color;
pub mod error;
pub mod file;
pub mod gamestate;
pub mod moves;
pub mod piece;
pub mod rank;
pub mod square;
pub mod zobrist;
