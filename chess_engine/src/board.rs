use crate::pieces::{ Piece , Color };
use std::fmt;
use std::num::NonZeroUsize;
use std::mem::size_of;


const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
// TODO: This is definitely super overkill! https://users.rust-lang.org/t/compile-time-const-unwrapping/51619/3
// Safety: 1 is a non-zero, non-negative, integer so this should be fine
const INITIAL_FULLMOVE_NUMBER: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(1) };
const BOARD_DISPLAY_STRING_SIZE: usize = 16 * size_of::<u8>();
const FEN_STRING_SLASH_NUMBER: usize = 7;

#[derive(Debug)]
pub struct Board<'a> {
    /// White or Black's turn
    turn: Color,
    /// Move pairs. Starts at 1.
    fullmove_number: NonZeroUsize,
    // TODO: implement Stack on top of Vec for history/move_stack
    // For now store FEN here, although that will probably change
    // TODO: how plausible would it be to build a struct for a FEN string created
    // on top of a Vec or String, that makes sure that the string is valid FEN?
    fen: &'a str,
}

// TODO: really understand the lifetimes here
impl Board<'_> {
    // TODO: How do we handle the case where an invalid fen is passed in?
    pub fn from_fen(fen: &str) -> Board {
        // TODO: validate fen
        Board {
            turn: Color::White,
            fullmove_number: INITIAL_FULLMOVE_NUMBER,
            fen,
        }
    }

    pub fn to_fen(&self) -> String {
        self.fen.to_string()
    }

    // TODO: figure out where to put this functionality
    pub fn is_valid_fen(fen: &str) -> bool {
        todo!()
    }
}

impl fmt::Display for Board<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: parse fen string so that it prints out one string with 8 new lines
        // including the trailing one, and spaces between each piece/char
        // How would I interact with the Formatter? Through all of the methods

        // NOTE: closure so that I can capture self from outer scope
        // TODO: go through the lifetimes here. Could we return an &str?
        let generate_display_string = || -> String {
            // TODO: is there a way to do this without heap allocation?
            // create a String with capacity that should hold all characters used to display board
            let mut output_string: String = String::with_capacity(BOARD_DISPLAY_STRING_SIZE);

            // TODO: pull out this functionality and put it somewhere more general?
            // grab the fen from self.fen
            // TODO: validate it!
            // numbers in the piece placement data are encoding the number of squares without pieces
            // we want each square without a piece on it to be displayed by a .
            // parse through the str. every time you see a non /, add it and a space to output_string
            // when you see a /, replace the last space with a \n. stop after the 7th / and and a final \n

            // TODO: how can you leverage Piece enum here? Also some of this would be double checked with validation
            // TODO: what would we have to do to make an array work here?
            // let valid_piece_chars = ['r', 'R', 'n', 'N', 'b', 'B', 'q', 'Q', 'k', 'K', 'p', 'P'];
            let valid_pieces = "rRnNbBqQkKpP";

            // NOTE: should be safe to iterate over chars as opposed to graphemes, because we know that a 
            // valid fen string should only include ascii chars.
            for (i,c) in self.fen.chars().enumerate(){
                match c {
                    c if valid_pieces.contains(c) => {
                        // TODO: is there a way to do this in one push?
                        output_string.push(c);
                        output_string.push(' ');
                    },
                    c if c == '/' => {
                        // TODO: how can I avoid popping and just use index to overwrite?
                        output_string.pop();
                        output_string.push('\n');
                    }
                    c if c.is_numeric() => {
                        // TODO: better error handling
                        let mut i = c.to_digit(10).unwrap();
                        while i > 0 {
                            output_string.push('.');
                            output_string.push(' ');
                            i -= 1;
                        } 
                    },
                    // TODO: better error handling
                    _ => {
                        // pop off last space
                        output_string.pop();
                        break;
                    }
                }
            }
            output_string
        };

        // TODO: figure out how I would pass &str
        write!(f, "{}", generate_display_string())
    }
}

impl Default for Board<'_> {
    fn default() -> Self {
        Board::from_fen(DEFAULT_FEN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fen_parsing() {
        let board: Board = Board::from_fen(DEFAULT_FEN);
        let output_fen = board.to_fen();
        assert_eq!(DEFAULT_FEN, output_fen);

        let sicilian_fen = "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2";
        let board: Board = Board::from_fen(sicilian_fen);
        let output_fen = board.to_fen();
        assert_eq!(sicilian_fen, output_fen)
    }

    #[test]
    fn test_board_to_string() {
        let board: Board = Board::from_fen(DEFAULT_FEN);
        let ref_string = "r n b q k b n r\np p p p p p p p\n. . . . . . . .\n. . . . . . . .\n. . . . . . . .\n. . . . . . . .\nP P P P P P P P\nR N B Q K B N R";
        let output_string = board.to_string(); // autoderived from impl Display
        assert_eq!(ref_string, output_string);
    }
}
