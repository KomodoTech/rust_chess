use chess_engine::gamestate::GamestateBuilder;

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

fn read_lines<P>(path: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    Ok(io::BufReader::new(file).lines())
}

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
fn perft() {
    let perft_expected_path = Path::new(PERFT_EXPECTED_PATH);
    let expected = init_expected(perft_expected_path).unwrap();

    println!("{:#?}\n Testing {} FENs", expected, expected.len());
}
