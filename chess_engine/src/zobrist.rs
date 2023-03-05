use rand::prelude::*;
use rand_pcg::Lcg128Xsl64;
use strum::EnumCount;

use crate::board::NUM_BOARD_SQUARES;
use crate::castle_perm::NUM_CASTLE_PERM;
use crate::piece::Piece;

// TODO: make Zobrist generate at compile time with proc macro

// TODO: test to make sure seed is a good choice
/// Seed used for Zobrist Hashing. Note that many PRNG implementations will behave poorly
/// if the seed is poorly distributed (it should have roughly equal number of 0s and 1s)
// NOTE: the seed data comes from this article: https://www.pcg-random.org/posts/simple-portable-cpp-seed-entropy.html
const ZOBRIST_SEED: [u8; 32] = [
    0x67, 0x0e, 0x5a, 0x45, 0x9a, 0xc9, 0xea, 0x9c, 0x88, 0x85, 0x36, 0x20, 0xc4, 0xc8, 0x36, 0xf9,
    0x07, 0xab, 0x56, 0x40, 0xb2, 0x0b, 0x31, 0x3e, 0x7b, 0x94, 0x50, 0x51, 0x37, 0xf5, 0x0e, 0x84,
];

#[derive(Debug, PartialEq, Eq)]
pub struct Zobrist {
    pub color_key: u64,
    pub piece_keys: [[u64; NUM_BOARD_SQUARES]; Piece::COUNT],
    pub en_passant_keys: [u64; NUM_BOARD_SQUARES],
    pub castle_keys: [u64; NUM_CASTLE_PERM],
}

// NOTE: https://craftychess.com/hyatt/collisions.html
// NOTE: https://www.unf.edu/~cwinton/html/cop4300/s09/class.notes/LCGinfo.pdf
// NOTE: https://www.stmintz.com/ccc/index.php?id=29863
// NOTE: https://www.pcg-random.org/index.html
// NOTE: https://rust-random.github.io/book/portability.html
// NOTE: https://rust-random.github.io/book/guide-rngs.html
// NOTE: https://www.pcg-random.org/posts/cpp-seeding-surprises.html
/// Zobrist hashing using rand_pcg variant that should work decently well on 32bit and 64bit machines
/// We don't require cryptographically secure PRNG's, but there have historically been
/// many truly terribly implemented random number generators, so we're doing our best to choose
/// a decent one, even though the effect of collisions seems to be fairly minimal for Zobrist
/// hashing.
impl Zobrist {
    pub fn new() -> Self {
        // declare seed deterministically from const we declared
        let mut seed: <Lcg128Xsl64 as SeedableRng>::Seed = ZOBRIST_SEED;
        // build Permuted Congruential Generator to do pseudo random number generation
        let mut rng: Lcg128Xsl64 = Lcg128Xsl64::from_seed(seed);
        // initialize Zobrist keys we want to fill with pseudo random values
        let mut color_key: u64 = rng.gen();
        let mut piece_keys = [[0u64; NUM_BOARD_SQUARES]; Piece::COUNT];
        for square_array in &mut piece_keys {
            rng.fill(square_array)
        }
        let mut en_passant_keys = [0u64; NUM_BOARD_SQUARES];
        rng.fill(&mut en_passant_keys);
        let mut castle_keys = [0u64; NUM_CASTLE_PERM];
        rng.fill(&mut castle_keys);

        Zobrist {
            color_key,
            piece_keys,
            en_passant_keys,
            castle_keys,
        }
    }
}

impl Default for Zobrist {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    // TODO: properly seed and test Zobrist key gen to check for collision rate in norm
    // #[test]
    // fn test_gen_position_key_deterministic() {
    //     let gamestate = Gamestate::default();
    //     let output = gamestate.gen_position_key();
    //     let expected = gamestate.gen_position_key();
    //     println!("output: {}, expected: {}", output, expected);
    //     assert_eq!(output, expected);
    // }
}
