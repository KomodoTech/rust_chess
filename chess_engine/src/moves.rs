use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct Move;

impl Move {
    pub fn from_uci(uci: &str) -> Self {
        todo!()
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!()
        // write!(f, "{}", output_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_from_uci() {
    //     let ref_string = "e2e4";
    //     let new_move: Move = Move::from_uci(ref_string);
    //     let output_string = new_move.to_string();
    //     assert_eq!(ref_string, output_string);
    // }
}
