use std::ops::Shr;

use crate::{types::*, display::print_bitboard};

//SQUARES
pub const DARK_SQUARES: Bitboard = 0xAA55AA55AA55AA55;
pub const LIGHT_SQUARES: Bitboard = 0x55AA55AA55AA55AA;

pub const WHITE_KINGSIDE_CASTLE: Bitboard = 0x60;
pub const WHITE_QUEENSIDE_CASTLE: Bitboard = 0xE;
pub const BLACK_KINGSIDE_CASTLE: Bitboard = 0x6000000000000000;
pub const BLACK_QUEENSIDE_CASTLE: Bitboard = 0xE00000000000000;


//FILES - or - COLUMNS
pub const FILE_ABB: Bitboard = 0x0101010101010101;
pub const FILE_BBB: Bitboard = FILE_ABB << 1;
pub const FILE_CBB: Bitboard = FILE_ABB << 2;
pub const FILE_DBB: Bitboard = FILE_ABB << 3;
pub const FILE_EBB: Bitboard = FILE_ABB << 4;
pub const FILE_FBB: Bitboard = FILE_ABB << 5;
pub const FILE_GBB: Bitboard = FILE_ABB << 6;
pub const FILE_HBB: Bitboard = FILE_ABB << 7;

pub const NOT_FILE_ABB: Bitboard = FILE_BBB|FILE_CBB|FILE_DBB|FILE_EBB|FILE_FBB|FILE_GBB|FILE_HBB;
pub const NOT_FILE_HBB: Bitboard = FILE_ABB|FILE_BBB|FILE_CBB|FILE_DBB|FILE_EBB|FILE_FBB|FILE_GBB;

//RANKS - or - ROWS
pub const RANK_1BB: Bitboard = 0xFF;
pub const RANK_2BB: Bitboard = RANK_1BB << (8 * 1);
pub const RANK_3BB: Bitboard = RANK_1BB << (8 * 2);
pub const RANK_4BB: Bitboard = RANK_1BB << (8 * 3);
pub const RANK_5BB: Bitboard = RANK_1BB << (8 * 4);
pub const RANK_6BB: Bitboard = RANK_1BB << (8 * 5);
pub const RANK_7BB: Bitboard = RANK_1BB << (8 * 6);
pub const RANK_8BB: Bitboard = RANK_1BB << (8 * 7);

pub const NOT_RANK_1BB: Bitboard = RANK_2BB|RANK_3BB|RANK_4BB|RANK_5BB|RANK_6BB|RANK_7BB|RANK_8BB;
pub const NOT_RANK_8BB: Bitboard = RANK_1BB|RANK_2BB|RANK_3BB|RANK_4BB|RANK_5BB|RANK_6BB|RANK_7BB;

pub const NOT_OUTER: Bitboard = NOT_FILE_ABB&NOT_FILE_HBB&NOT_RANK_1BB&NOT_RANK_8BB;

pub type Bitboard = u64;

pub trait BitboardMethods {
    fn set_bit(&self, square: Square) -> Bitboard;
    fn unset_bit(&self, square: Square) -> Bitboard;
    fn pop_lsb(&mut self) -> Bitboard;
    fn to_square(&self) -> Square;
    fn from_square(square: Square) -> Bitboard;
    fn get_squares(&self) -> Vec<Square>;
}

pub trait BitboardConstants {
    const EMPTY: Bitboard;
}

impl BitboardConstants for Bitboard {
    const EMPTY: Bitboard = 0;
}

impl BitboardMethods for Bitboard{
    fn set_bit(&self, square: Square) -> Bitboard {
        return self | Bitboard::from_square(square);
    }

    fn unset_bit(&self, square: Square) -> Bitboard {
        return self & !Bitboard::from_square(square);
    }

    fn pop_lsb(&mut self) -> Bitboard {
        //mutate self
        let lsb = *self&(!*self+1);
        *self ^= lsb;
        return lsb;
    }

    fn to_square(&self) -> Square {
        return self.trailing_zeros() as Square;
    }

    fn from_square(square: Square) -> Bitboard {
        return 1_u64 << square as u8;
    }

    fn get_squares(&self) -> Vec<Square> {
        let mut squares: Vec<Square> = Vec::new();
        let mut board = *self;
        while board != 0 {
            squares.push(board.pop_lsb().trailing_zeros() as u8);
        }
        return squares;
    }
    
}





