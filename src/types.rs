use std::{fmt::Display, fmt::Formatter, fmt::Result, ops::{Not}};
use bitintr::Pext;
use crate::bitboard::*;

#[derive(PartialEq, Eq)]
pub struct GameState(pub u8);
pub type Piece = usize;
pub type CastlingDirection = usize;

//GAMESTATE CONSTANTS
pub trait GameStateConstants{
    const CHECKMATE: GameState;
    const CHECK: GameState;
    const DRAW: GameState;
    const IN_PROGRESS: GameState;
    const INVALID_STATE: GameState;
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match *self{
            GameState::CHECKMATE => write!(f, "CHECKMATE"),
            GameState::CHECK => write!(f, "CHECK"),
            GameState::DRAW => write!(f, "DRAW"),
            GameState::IN_PROGRESS => write!(f, "IN_PROGRESS"),
            GameState::INVALID_STATE => write!(f, "INVALID_STATE"),
            _ => panic!("Error: Unexpected value in Side: {}", self)
        }
    }
}

impl GameStateConstants for GameState{
    const CHECKMATE: GameState = GameState(0);
    const CHECK: GameState = GameState(1);
    const DRAW: GameState = GameState(2);
    const IN_PROGRESS: GameState = GameState(3);
    const INVALID_STATE: GameState = GameState(4);
}

//CASTLING SIDE
pub const KING_SIDE : CastlingDirection = 0;
pub const QUEEN_SIDE : CastlingDirection = 1;

//PIECES
pub const PAWN: Piece = 0;
pub const KNIGHT: Piece = 1;
pub const BISHOP: Piece = 2;
pub const ROOK: Piece = 3;
pub const QUEEN: Piece = 4;
pub const KING: Piece = 5;

#[derive(Copy)]
#[derive(Clone)]
pub struct Magic{
    pub mask: Bitboard,
    pub magic: Bitboard,
    pub attacks: [Bitboard; 4096],
    pub shift: usize
}

pub trait MagicIndex{
    fn get_index(&self, occupied: Bitboard) -> usize;
}

impl MagicIndex for Magic{
    fn get_index(&self, occupancy: Bitboard) -> usize {
        return Pext::pext(occupancy, self.mask) as usize;
    }
}

//SIDES
#[derive(PartialEq, Eq)]
#[derive(Copy)]
#[derive(Clone)]
pub struct Side(pub usize);

pub trait SideConstants{
    const WHITE: Side;
    const BLACK: Side;
}

impl SideConstants for Side{
    const WHITE: Side = Side(0);
    const BLACK: Side = Side(1);
}

impl Not for Side {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Side::WHITE => Side::BLACK,
            Side::BLACK => Side::WHITE,
            _ => panic!("Error: Unexpected value in Side: {}", self)
        }
    }
}

impl Display for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match *self{
            Side::WHITE => write!(f, "White"),
            Side::BLACK => write!(f, "Black"),
            _ => panic!("Error: Unexpected value in Side: {}", self)
        }
    }
}

//SQUARES
pub type Square = u8;

pub trait SquareConstants{
    const A1: Square; const B1: Square; const C1: Square; const D1: Square; 
    const E1: Square; const F1: Square; const G1: Square; const H1: Square;
    const A2: Square; const B2: Square; const C2: Square; const D2: Square; 
    const E2: Square; const F2: Square; const G2: Square; const H2: Square;
    const A3: Square; const B3: Square; const C3: Square; const D3: Square; 
    const E3: Square; const F3: Square; const G3: Square; const H3: Square;
    const A4: Square; const B4: Square; const C4: Square; const D4: Square; 
    const E4: Square; const F4: Square; const G4: Square; const H4: Square;
    const A5: Square; const B5: Square; const C5: Square; const D5: Square; 
    const E5: Square; const F5: Square; const G5: Square; const H5: Square;
    const A6: Square; const B6: Square; const C6: Square; const D6: Square; 
    const E6: Square; const F6: Square; const G6: Square; const H6: Square;
    const A7: Square; const B7: Square; const C7: Square; const D7: Square; 
    const E7: Square; const F7: Square; const G7: Square; const H7: Square;
    const A8: Square; const B8: Square; const C8: Square; const D8: Square; 
    const E8: Square; const F8: Square; const G8: Square; const H8: Square;
}

pub trait SquareMethods{
    fn get_string(&self) -> String;
}

impl SquareMethods for Square{
    fn get_string(&self) -> String{
        let mut string = String::new();
        string.push((self % 8 + 97) as u8 as char);
        string.push((self / 8 + 49) as u8 as char);
        return string;
    }
}


impl SquareConstants for Square{
    const A1: Square = 0;  const B1: Square = 1;
    const C1: Square = 2;  const D1: Square = 3;
    const E1: Square = 4;  const F1: Square = 5;
    const G1: Square = 6;  const H1: Square = 7;
    const A2: Square = 8;  const B2: Square = 9;
    const C2: Square = 10; const D2: Square = 11;
    const E2: Square = 12; const F2: Square = 13;
    const G2: Square = 14; const H2: Square = 15;
    const A3: Square = 16; const B3: Square = 17;
    const C3: Square = 18; const D3: Square = 19;
    const E3: Square = 20; const F3: Square = 21;
    const G3: Square = 22; const H3: Square = 23;
    const A4: Square = 24; const B4: Square = 25;
    const C4: Square = 26; const D4: Square = 27;
    const E4: Square = 28; const F4: Square = 29;
    const G4: Square = 30; const H4: Square = 31;
    const A5: Square = 32; const B5: Square = 33;
    const C5: Square = 34; const D5: Square = 35;
    const E5: Square = 36; const F5: Square = 37;
    const G5: Square = 38; const H5: Square = 39;
    const A6: Square = 40; const B6: Square = 41;
    const C6: Square = 42; const D6: Square = 43;
    const E6: Square = 44; const F6: Square = 45;
    const G6: Square = 46; const H6: Square = 47;
    const A7: Square = 48; const B7: Square = 49;
    const C7: Square = 50; const D7: Square = 51;
    const E7: Square = 52; const F7: Square = 53;
    const G7: Square = 54; const H7: Square = 55;
    const A8: Square = 56; const B8: Square = 57;
    const C8: Square = 58; const D8: Square = 59;
    const E8: Square = 60; const F8: Square = 61;
    const G8: Square = 62; const H8: Square = 63;
}

pub struct Squares;

impl IntoIterator for Squares{
    type Item = Square;
    type IntoIter = std::array::IntoIter<Square, 64>;
    fn into_iter(self) -> Self::IntoIter {
        std::array::IntoIter::into_iter([
            Square::A1, Square::B1, Square::C1, Square::D1,
            Square::E1, Square::F1, Square::G1, Square::H1,
            Square::A2, Square::B2, Square::C2, Square::D2,
            Square::E2, Square::F2, Square::G2, Square::H2,
            Square::A3, Square::B3, Square::C3, Square::D3,
            Square::E3, Square::F3, Square::G3, Square::H3,
            Square::A4, Square::B4, Square::C4, Square::D4,
            Square::E4, Square::F4, Square::G4, Square::H4,
            Square::A5, Square::B5, Square::C5, Square::D5,
            Square::E5, Square::F5, Square::G5, Square::H5,
            Square::A6, Square::B6, Square::C6, Square::D6,
            Square::E6, Square::F6, Square::G6, Square::H6,
            Square::A7, Square::B7, Square::C7, Square::D7,
            Square::E7, Square::F7, Square::G7, Square::H7,
            Square::A8, Square::B8, Square::C8, Square::D8,
            Square::E8, Square::F8, Square::G8, Square::H8,
        ].into_iter())
    }
}

