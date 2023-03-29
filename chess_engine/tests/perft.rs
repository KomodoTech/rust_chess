use chess_engine::{
    error::{GamestateValidityCheckError, MoveDeserializeError, MoveGenError, UndoMoveError},
    gamestate::{Gamestate, GamestateBuilder, ValidityCheck},
};
use num::Zero;

use std::{
    error::Error,
    fs::File,
    io::{self, BufRead},
    num::ParseIntError,
    path::Path,
};

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

#[derive(Debug)]
struct Perft {
    fen: String,
    node_counts: Vec<u64>,
}

impl Perft {
    pub fn new(fen: String, node_counts: Vec<u64>) -> Self {
        Perft { fen, node_counts }
    }
}

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

fn divided_perft(
    gamestate: &mut Gamestate,
    depth: usize,
    leaf_count: &mut u64,
) -> Result<u64, PerftError> {
    gamestate.check_gamestate(ValidityCheck::Move)?;

    println!("{}", gamestate);
    println!("PERFT TO DEPTH {}", depth);

    *leaf_count = 0;

    let move_list = gamestate.gen_move_list()?;
    for (move_idx, move_) in move_list.moves.into_iter().flatten().enumerate() {
        if gamestate.make_move(move_).is_ok() {
            let mut total_count = *leaf_count;

            perft(gamestate, depth - 1, leaf_count)?;
            gamestate.undo_move()?;

            let prev_delta_count = *leaf_count - total_count;
            println!(
                "Move {}: {}{} : {}",
                move_idx,
                move_.get_start()?,
                move_.get_end()?,
                prev_delta_count
            );
        }

        println!("TOTAL NODES VISITED: {}", *leaf_count);
    }

    Ok(*leaf_count)
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
fn init_expected<P>(perft_expected_path: P) -> Result<Vec<Perft>, Box<dyn Error>>
where
    P: AsRef<Path>,
{
    let mut expected = Vec::<Perft>::with_capacity(usize::next_power_of_two(PERFT_FEN_COUNT));
    let lines = read_lines(perft_expected_path)?;

    for line in lines {
        let node_counts = line?;
        let mut sections = node_counts.split(';').collect::<Vec<&str>>().into_iter();
        let fen = sections
            .next()
            .ok_or(PerftInitExpectedError::NoFen)?
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

        expected.push(Perft::new(fen, counts));
    }
    Ok(expected)
}

#[test]
fn test_perft() {
    let perft_expected_path = Path::new(PERFT_EXPECTED_PATH);
    let expected = init_expected(perft_expected_path).unwrap();

    // println!("{:#?}\n Testing {} FENs", expected, expected.len());

    let mut gamestate = GamestateBuilder::new_with_fen(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    )
    .unwrap()
    .build()
    .unwrap();

    let mut leaf_count: u64 = 0;
    let node_count = divided_perft(&mut gamestate, 2, &mut leaf_count).unwrap();
    println!("node_count: {}", node_count);
}
