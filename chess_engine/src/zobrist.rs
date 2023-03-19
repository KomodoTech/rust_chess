use crate::{
    board::NUM_EXTERNAL_BOARD_SQUARES, castle_perm::NUM_CASTLE_PERM, file::File, piece::Piece,
};
use strum::EnumCount;

use rand::prelude::*;
use rand_pcg::Lcg128Xsl64;

use once_cell::sync::Lazy;
use std::sync::Mutex;

/// Lazily initialize ZOBRIST values using OnceCell to create it only once
/// and share it between Gamestates
pub static ZOBRIST: Lazy<Mutex<Zobrist>> = Lazy::new(|| {
    let zobrist = Zobrist::default();
    Mutex::new(zobrist)
});

// TODO: test to make sure seed is a good choice
/// Seed used for Zobrist Hashing. Note that many PRNG implementations will behave poorly
/// if the seed is poorly distributed (it should have roughly equal number of 0s and 1s)
// NOTE: the seed data comes from this article: https://www.pcg-random.org/posts/simple-portable-cpp-seed-entropy.html
pub const ZOBRIST_SEED: [u8; 32] = [
    0x67, 0x0e, 0x5a, 0x45, 0x9a, 0xc9, 0xea, 0x9c, 0x88, 0x85, 0x36, 0x20, 0xc4, 0xc8, 0x36, 0xf9,
    0x07, 0xab, 0x56, 0x40, 0xb2, 0x0b, 0x31, 0x3e, 0x7b, 0x94, 0x50, 0x51, 0x37, 0xf5, 0x0e, 0x84,
];

// TODO: look into adding extra fields for the pocket
#[derive(Debug, PartialEq, Eq)]
pub struct Zobrist {
    pub color_key: u64,
    pub piece_keys: [[u64; NUM_EXTERNAL_BOARD_SQUARES]; Piece::COUNT],
    pub en_passant_keys: [u64; File::COUNT],
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
/// NOTE: That when taking into account permutations, there are too many
/// possible chess positions to hold in 64 bits.
impl Zobrist {
    /// Generates 781 (12*64 + 1 + 4 + 8) pseudo random numbers to be used for
    /// generation of a non-unique hash key to represent a board position.
    fn new() -> Self {
        // declare seed deterministically from const we declared
        // TODO: remove mut
        let seed: <Lcg128Xsl64 as SeedableRng>::Seed = ZOBRIST_SEED;
        // build Permuted Congruential Generator to do pseudo random number generation
        let mut rng: Lcg128Xsl64 = Lcg128Xsl64::from_seed(seed);
        // initialize Zobrist keys we want to fill with pseudo random values
        // TODO: remove mut
        let color_key: u64 = rng.gen();
        let mut piece_keys = [[0u64; NUM_EXTERNAL_BOARD_SQUARES]; Piece::COUNT];
        for square_array in &mut piece_keys {
            rng.fill(square_array)
        }
        // We only need FILE:COUNT (8) en_passant_keys because there is only
        // one row per active_color (determined by color_key) that allows for
        // en passant captures (Row 6 and Row 2)
        let mut en_passant_keys = [0u64; File::COUNT];
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
    #[test]
    fn test_zobrist_visual() {
        let color_key = ZOBRIST.lock().unwrap().color_key;
        let piece_keys = ZOBRIST.lock().unwrap().piece_keys;
        let en_passant_keys = ZOBRIST.lock().unwrap().en_passant_keys;
        let castle_keys = ZOBRIST.lock().unwrap().castle_keys;

        println!("color_key:\n{:#?}", color_key);
        println!("piece_keys:\n{:#?}", piece_keys);
        println!("en_passant_keys:\n{:#?}", en_passant_keys);
        println!("castle_keys:\n{:#?}", castle_keys);
    }
}
