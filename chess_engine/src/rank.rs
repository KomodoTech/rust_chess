use crate::{board::NUM_BOARD_SQUARES, error::RankConversionError};
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{Display, EnumCount as EnumCountMacro, EnumIter, EnumString};

#[rustfmt::skip]
pub const RANKS_BOARD: [Option<Rank>; NUM_BOARD_SQUARES] = [
    None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
    None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
    None,  Some(Rank::Rank1),  Some(Rank::Rank1),  Some(Rank::Rank1),  Some(Rank::Rank1),  Some(Rank::Rank1),  Some(Rank::Rank1),  Some(Rank::Rank1),  Some(Rank::Rank1), None,
    None,  Some(Rank::Rank2),  Some(Rank::Rank2),  Some(Rank::Rank2),  Some(Rank::Rank2),  Some(Rank::Rank2),  Some(Rank::Rank2),  Some(Rank::Rank2),  Some(Rank::Rank2), None,
    None,  Some(Rank::Rank3),  Some(Rank::Rank3),  Some(Rank::Rank3),  Some(Rank::Rank3),  Some(Rank::Rank3),  Some(Rank::Rank3),  Some(Rank::Rank3),  Some(Rank::Rank3), None,
    None,  Some(Rank::Rank4),  Some(Rank::Rank4),  Some(Rank::Rank4),  Some(Rank::Rank4),  Some(Rank::Rank4),  Some(Rank::Rank4),  Some(Rank::Rank4),  Some(Rank::Rank4), None,
    None,  Some(Rank::Rank5),  Some(Rank::Rank5),  Some(Rank::Rank5),  Some(Rank::Rank5),  Some(Rank::Rank5),  Some(Rank::Rank5),  Some(Rank::Rank5),  Some(Rank::Rank5), None,
    None,  Some(Rank::Rank6),  Some(Rank::Rank6),  Some(Rank::Rank6),  Some(Rank::Rank6),  Some(Rank::Rank6),  Some(Rank::Rank6),  Some(Rank::Rank6),  Some(Rank::Rank6), None,
    None,  Some(Rank::Rank7),  Some(Rank::Rank7),  Some(Rank::Rank7),  Some(Rank::Rank7),  Some(Rank::Rank7),  Some(Rank::Rank7),  Some(Rank::Rank7),  Some(Rank::Rank7), None,
    None,  Some(Rank::Rank8),  Some(Rank::Rank8),  Some(Rank::Rank8),  Some(Rank::Rank8),  Some(Rank::Rank8),  Some(Rank::Rank8),  Some(Rank::Rank8),  Some(Rank::Rank8), None,
    None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
    None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
];

#[derive(EnumIter, Debug, Copy, Clone, PartialEq, Eq, Display, EnumCountMacro)]
pub enum Rank {
    Rank1,
    Rank2,
    Rank3,
    Rank4,
    Rank5,
    Rank6,
    Rank7,
    Rank8,
}

impl TryFrom<usize> for Rank {
    type Error = RankConversionError;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::iter()
            .find(|r| *r as usize == value)
            .ok_or(RankConversionError::FromUsize(value))
    }
}

// impl Add<usize> for Rank {
//     type Output = Result<Self, ConversionError>;
//     fn add(self, rhs: usize) -> Self::Output {
//         let result: Result<Self, ConversionError> = (self as usize + rhs).try_into();
//         result
//     }
// }
