use chess_engine::{
    error::{GamestateValidityCheckError, MoveDeserializeError, MoveGenError, UndoMoveError},
    gamestate::{Gamestate, GamestateBuilder, ValidityCheck},
};
use num::Zero;

use std::{
    collections::HashMap,
    env,
    error::Error,
    fs::File,
    io::{self, BufRead, ErrorKind},
    num::ParseIntError,
    path::{Path, PathBuf},
};

const PARENT_DIR: &str = "chess_engine";
const PERFT_EXPECTED_PATH: &str = "tests/perft_expected.txt";
const PERFT_FEN_COUNT: usize = 126;

// TODO: look into moving this out of file
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum PerftError {
    #[error(transparent)]
    PerftInitExpected(#[from] PerftInitExpectedError),

    #[error(transparent)]
    PerftGamestate(#[from] GamestateValidityCheckError),

    #[error(transparent)]
    PerftMoveGen(#[from] MoveGenError),

    #[error(transparent)]
    UndoMove(#[from] UndoMoveError),

    #[error(transparent)]
    MoveDeserialize(#[from] MoveDeserializeError),
}

#[derive(Error, Debug, PartialEq)]
pub enum PerftInitExpectedError {
    #[error(transparent)]
    ParseNodeCount(#[from] ParseIntError),

    #[error("No FEN found in current line")]
    NoFen,

    #[error("While trying to deserialize line from perft file, encountered a line without a depth label")]
    NoDepthLabel,

    #[error("Encountered Error while trying to read line in perft file")]
    ReadLine,
}

// #[derive(Debug)]
// struct Perft {
//     fen: String,
//     node_counts: Vec<u64>,
// }

// impl Perft {
//     pub fn new(fen: String, node_counts: Vec<u64>) -> Self {
//         Perft { fen, node_counts }
//     }
// }

fn perft(gamestate: &mut Gamestate, depth: usize, leaf_count: &mut u64) -> Result<u64, PerftError> {
    gamestate.check_gamestate(ValidityCheck::Move)?;

    // Base Case
    if depth.is_zero() {
        *leaf_count += 1;
        return Ok(*leaf_count);
    } else {
        // Recursive Case
        let move_list = gamestate.gen_move_list()?;
        for move_ in move_list.moves.into_iter().flatten() {
            if gamestate.make_move(move_).is_ok() {
                perft(gamestate, depth - 1, leaf_count)?;
                gamestate.undo_move()?;
            }
        }
    }

    Ok(*leaf_count)
}

fn divided_perft(gamestate: &mut Gamestate, depth: usize) -> Result<u64, PerftError> {
    gamestate.check_gamestate(ValidityCheck::Move)?;

    println!("{}", gamestate);
    println!("PERFT TO DEPTH {}", depth);

    let mut leaf_count = 0;

    let move_list = gamestate.gen_move_list()?;
    for (move_index, move_) in move_list.moves.into_iter().flatten().enumerate() {
        if gamestate.make_move(move_).is_ok() {
            let total_count = leaf_count;

            perft(gamestate, depth - 1, &mut leaf_count)?;
            gamestate.undo_move()?;

            // TODO: rename everything as better naming conventions become clear
            // This is the count for the number of nodes visited on the last divided
            // "line". E.g. Just made initial move A2 A4 and there were 44 nodes visited
            // in that subtree
            let prev_delta_count = leaf_count - total_count;
            println!(
                "Move {}: {} {} : {}",
                move_index,
                move_.get_start()?,
                move_.get_end()?,
                prev_delta_count
            );
        }

        println!("TOTAL NODES VISITED: {}", leaf_count);
    }

    Ok(leaf_count)
}

fn get_perft_expected_path() -> io::Result<PathBuf> {
    // NOTE: the Current Working Directory is set to ${workspaceRoot} when
    // testing, but ${workspaceRoot}/metadata for debugging
    // https://github.com/rust-lang/rust-analyzer/issues/4705
    // https://github.com/rust-lang/cargo/issues/3946
    let curr_dir = std::env::current_dir()?;
    let curr_dir_last = curr_dir
        .iter()
        .last()
        .ok_or(io::Error::new(ErrorKind::NotFound, "curr_dir empty"))?
        .to_str()
        .ok_or(io::Error::new(
            ErrorKind::InvalidInput,
            "last part of curr_dir can't be parsed",
        ))?;

    let perf_expected_path = match curr_dir_last {
        _normal_path if curr_dir_last == PARENT_DIR => PERFT_EXPECTED_PATH.to_owned(),
        // Reset curr_dir for debug mode
        _debug_path => {
            format!("{}/{}", PARENT_DIR, PERFT_EXPECTED_PATH)
        }
    };

    Ok(PathBuf::from(perf_expected_path))
}

fn read_lines<P>(path: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    Ok(io::BufReader::new(file).lines())
}

/// Read in Perft results from pre-generated file. Create a Vec of Perft instances
/// that we can use to compare to our move generation
fn init_expected<P>(perft_expected_path: P) -> Result<HashMap<String, Vec<u64>>, Box<dyn Error>>
where
    P: AsRef<Path>,
{
    let mut expected =
        HashMap::<String, Vec<u64>>::with_capacity(usize::next_power_of_two(PERFT_FEN_COUNT));
    let lines = read_lines(perft_expected_path)?;

    for line in lines {
        let node_counts = line?;
        let mut sections = node_counts.split(';').collect::<Vec<&str>>().into_iter();
        let fen = sections
            .next()
            .ok_or(PerftInitExpectedError::NoFen)?
            .trim()
            .to_owned();

        let mut counts: Vec<u64> = Vec::new();

        for section in sections {
            // we don't need the depth label (e.g. D5) that will be stored via
            // the index of counts
            let mut node_count = section.split(' ').collect::<Vec<&str>>().into_iter();
            let _depth_label = node_count.next();

            counts.push(
                node_count
                    .next()
                    .ok_or(PerftInitExpectedError::NoDepthLabel)?
                    .parse::<u64>()?,
            );
        }

        expected.insert(fen, counts);
    }
    Ok(expected)
}

#[test]
fn test_perft() {
    let perft_expected_path = get_perft_expected_path().unwrap();
    let expected = init_expected(perft_expected_path).unwrap();

    let depth = 5;

    for position in expected {
        let fen = position.0;
        println!("FEN: {}", fen);

        assert!(depth > 0);
        let expected_node_count = position.1[depth - 1];

        let mut gamestate = GamestateBuilder::new_with_fen(fen.as_str())
            .unwrap()
            .build()
            .unwrap();

        let mut output_node_count = 0;
        let output_node_count = perft(&mut gamestate, depth, &mut output_node_count).unwrap();

        println!(
            "Expected: {}\nOutput: {}",
            expected_node_count, output_node_count
        );

        if output_node_count != expected_node_count {
            divided_perft(&mut gamestate, depth).unwrap();
        }

        assert_eq!(output_node_count, expected_node_count);
    }
}

//================================= DEBUGGING SCRATCH SPACE ===================
use chess_engine::moves::MoveBuilder;
use chess_engine::piece::Piece;
use chess_engine::square::Square;

#[test]
fn test_explore_kiwi_pete() {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let expected = init_expected(get_perft_expected_path().unwrap()).unwrap();
    let node_counts = expected.get(fen).cloned().unwrap();
}

// #[test]
// fn test_gamestate_make_undo_moves_depth_2_wn_e5_g6_kiwipete() {
//     let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

//     let mut gamestate = GamestateBuilder::new_with_fen(fen)
//         .unwrap()
//         .build()
//         .unwrap();

//     let move_wn_e5_g6 = MoveBuilder::new(Square::E5, Square::G6, Piece::WhiteKnight)
//         .build()
//         .unwrap();

//     // kiwipete after WN E5 to G6:
//     // r3k2r/p1ppqpb1/bn2pnN1/3P4/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1
//     gamestate.make_move(move_wn_e5_g6);

//     let move_list = gamestate.gen_move_list().unwrap().moves;

//     let mut move_errors = vec![];
//     let mut undo_errors = vec![];

//     let mut move_count: usize = 0;

//     println!("{}", gamestate);
//     for move_ in move_list.into_iter().flatten() {
//         match gamestate.make_move(move_) {
//             Ok(()) => {
//                 println!("Make Move Success:\n{}", move_);
//                 println!("{}", gamestate);

//                 move_count += 1;

//                 match gamestate.undo_move() {
//                     Ok(undo_move) => {
//                         println!("Undo Move Success:\n{}", undo_move);
//                         println!("{}", gamestate);
//                     }
//                     Err(e) => {
//                         println!("UNDO ERROR: {}", e);
//                         undo_errors.push(e);
//                     }
//                 }
//             }
//             Err(e) => {
//                 println!("MOVE ERROR: {}", e);
//                 move_errors.push(e);
//             }
//         }
//     }

//     println!("NUMBER OF MOVES: {}", move_count);
//     println!("MOVE Errors: {}\n{:#?}", move_errors.len(), move_errors);
//     println!("MOVE Errors: {}\n{:#?}", undo_errors.len(), move_errors);

//     let expected_move_count = 42;

//     assert_eq!(move_count, expected_move_count);
// }
