use crate::{
    board::{NUM_EXTERNAL_BOARD_SQUARES, NUM_INTERNAL_BOARD_SQUARES},
    error::FileConversionError,
};
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{Display, EnumCount as EnumCountMacro, EnumIter, EnumString};

#[rustfmt::skip]
pub const FILES_BOARD_120: [Option<File>; NUM_INTERNAL_BOARD_SQUARES] = [
    None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
    None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
    None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
    None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
    None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
    None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
    None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
    None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
    None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
    None,  Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH), None,
    None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
    None,  None,               None,               None,               None,               None,               None,               None,               None,              None,
];

#[rustfmt::skip]
pub const FILES_BOARD_64: [Option<File>; NUM_EXTERNAL_BOARD_SQUARES] = [
    Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH),
    Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH),
    Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH),
    Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH),
    Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH),
    Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH),
    Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH),
    Some(File::FileA),  Some(File::FileB),  Some(File::FileC),  Some(File::FileD),  Some(File::FileE),  Some(File::FileF),  Some(File::FileG),  Some(File::FileH),
];

#[derive(EnumIter, Debug, Copy, Clone, PartialEq, Eq, Display, EnumCountMacro)]
pub enum File {
    FileA,
    FileB,
    FileC,
    FileD,
    FileE,
    FileF,
    FileG,
    FileH,
}

impl TryFrom<usize> for File {
    type Error = FileConversionError;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::iter()
            .find(|r| *r as usize == value)
            .ok_or(FileConversionError::FromUsize {
                invalid_usize: value,
            })
    }
}

impl From<File> for char {
    fn from(value: File) -> Self {
        match value {
            File::FileA => 'A',
            File::FileB => 'B',
            File::FileC => 'C',
            File::FileD => 'D',
            File::FileE => 'E',
            File::FileF => 'F',
            File::FileG => 'G',
            File::FileH => 'H',
        }
    }
}
