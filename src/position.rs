use core::panic;
use std::{fmt::{Display, Formatter, Result}};
use rayon::prelude::*;

use crate::{
    bitboard::*, 
    types::*, 
    maps::{
        get_pawn_attacks,
        get_knight_attacks, 
        get_bishop_attacks, 
        get_rook_attacks, 
        get_queen_attacks,
        get_king_attacks, 
        DIRECTIONAL_MAP_FILE,
        DIRECTIONAL_MAP_RANK,
        DIRECTIONAL_MAP_DD, 
        DIRECTIONAL_MAP_DA, get_ray_between_squares, get_pawn_moves, 
        }, display::{print_position}
    };

pub struct PositionEvaluation{
    pub moves: Vec<Move>,
    pub game_state: GameState,
    pub state_note: Option<String>,
    pub score: Option<f32>
}

const PIN_MULTIPLIER: f32 = 10.0;
const SQUARE_MULTIPLIER: f32 = 10.0;

const SCORE_WHITE_WINS: f32 = 1000000.0;
const SCORE_BLACK_WINS: f32 = -1000000.0;

const PIECE_VALUES: [f32; 6] = [
    100.0,
    300.0,
    300.0,
    500.0,
    900.0,
    0.0
];

pub type SidePieces = [Bitboard; 6];

pub trait SidePiecesMethods{
    fn new() -> SidePieces;
    fn new_game(side: Side) -> SidePieces;
    fn occupancy(&self) -> Bitboard;
    fn get_piece_type_at_square(&self, square: Bitboard) -> Option<Piece>;
}

impl SidePiecesMethods for SidePieces{
    fn new () -> SidePieces{
        return [0; 6];
    }

    fn new_game(side: Side) -> SidePieces{
        let mut pieces: [Bitboard; 6]= [0; 6];
    
        for i in 0..6{
            if i == PAWN{
                pieces[i] = match side{
                    Side::WHITE => 0xFF00,
                    Side::BLACK => 0xFF000000000000,
                    _ => panic!("Error: Unexpected value in Side: {}", side)
                }
            }
            else if i == KNIGHT{
                pieces[i] = match side{
                    Side::WHITE => 0x42,
                    Side::BLACK => 0x4200000000000000,
                    _ => panic!("Error: Unexpected value in Side: {}", side)
                }
            }
            else if i == BISHOP{
                pieces[i] = match side{
                    Side::WHITE => 0x24,
                    Side::BLACK => 0x2400000000000000,
                    _ => panic!("Error: Unexpected value in Side: {}", side)
                }
            }
            else if i == ROOK{
                pieces[i] = match side{
                    Side::WHITE => 0x81,
                    Side::BLACK => 0x8100000000000000,
                    _ => panic!("Error: Unexpected value in Side: {}", side)
                }
            }
            else if i == QUEEN{
                pieces[i] = match side{
                    Side::WHITE => 0x8,
                    Side::BLACK => 0x800000000000000,
                    _ => panic!("Error: Unexpected value in Side: {}", side)
                }
            }
            else if i == KING{
                pieces[i] = match side{
                    Side::WHITE => 0x10,
                    Side::BLACK => 0x1000000000000000,
                    _ => panic!("Error: Unexpected value in Side: {}", side)
                }
            }
        }
    
        return pieces;
    }

    fn occupancy(&self) -> Bitboard{
        let mut occupancy = Bitboard::EMPTY;
        for pieces in self.iter(){
            occupancy |= *pieces;
        }
        return occupancy;
    }

    fn get_piece_type_at_square(&self, square: Bitboard) -> Option<Piece>{
        for x in 0..6{
            if self[x] & square != 0{
                return Some(x);
            }
        }
        return None;
    }

}


#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Copy)]
pub struct ZobristHasher{
    pub piece_hashes: [[[u64; 64]; 6]; 2],
    pub castling_hashes: [u64; 16],
    pub en_passant_hashes: [u64; 64],
    pub side_to_move_hash: u64
}

impl ZobristHasher{
    pub fn new() -> ZobristHasher{
        let mut piece_hashes: [[[u64; 64]; 6]; 2] = [[[0; 64]; 6]; 2];
        let mut castling_hashes: [u64; 16] = [0; 16];
        let mut en_passant_hashes: [u64; 64] = [0; 64];
        let side_to_move_hash: u64;

        for side in 0..2{
            for piece in 0..6{
                for square in 0..64{
                    piece_hashes[side][piece][square] = rand::random::<u64>();
                }
            }
        }

        for i in 0..16{
            castling_hashes[i] = rand::random::<u64>();
        }

        for i in 0..64{
            en_passant_hashes[i] = rand::random::<u64>();
        }

        side_to_move_hash = rand::random::<u64>();

        return ZobristHasher{
            piece_hashes,
            castling_hashes,
            en_passant_hashes,
            side_to_move_hash
        }
    }

    pub fn hash_position(&self, position: &Position) -> u64{
        let mut hash: u64 = 0;

        for side in 0..2{
            for piece in 0..6{
                for square in 0..64{
                    if position.pieces[side][piece] & square.to_bitboard() != 0{
                        hash ^= self.piece_hashes[side][piece][square as usize];
                    }
                }
            }
        }

        hash ^= self.castling_hashes[position.castling_rights.get_zobrist_index()];

        if position.en_passant_square != None{
            hash ^= self.en_passant_hashes[position.en_passant_square.unwrap() as usize];
        }

        if position.side_to_move == Side::BLACK{
            hash ^= self.side_to_move_hash;
        }

        return hash;
    }

}

const MAX_ZOBRIST_ARRAY_SIZE: usize = 100;

#[derive(PartialEq)]
#[derive(Copy)]
#[derive(Clone)]
pub struct ZobristMoveStack{
    pub zobrist_array: [u64; MAX_ZOBRIST_ARRAY_SIZE],
    pub zobrist_array_index: usize
}

impl ZobristMoveStack{
    pub fn new() -> ZobristMoveStack{
        return ZobristMoveStack{
            zobrist_array: [0; MAX_ZOBRIST_ARRAY_SIZE],
            zobrist_array_index: 0
        }
    }

    pub fn get_repetitions(&self, zobrist_hash: u64) -> usize{
        return self.zobrist_array.par_iter().filter(|&&x| x == zobrist_hash).count();
    }

    pub fn add(&mut self, zobrist_hash: u64){
        //if we are at the end of the array, we need to shift everything down
        if self.zobrist_array_index == MAX_ZOBRIST_ARRAY_SIZE - 1{
            for i in 0..MAX_ZOBRIST_ARRAY_SIZE - 1{
                self.zobrist_array[i] = self.zobrist_array[i + 1];
            }
            self.zobrist_array[MAX_ZOBRIST_ARRAY_SIZE - 1] = zobrist_hash;
        }
        else{
            self.zobrist_array[self.zobrist_array_index] = zobrist_hash;
            self.zobrist_array_index += 1;
        }
    }
}

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Copy)]
#[derive(Clone)]
pub struct Castling {
    pub white_king_side: bool,
    pub white_queen_side: bool,
    pub black_king_side: bool,
    pub black_queen_side: bool,
}

#[derive(Copy)]
#[derive(Clone)]
pub struct PieceInfo{
    pub piece: Piece,
    pub square: Square,
}

#[derive(Copy)]
#[derive(Clone)]
pub struct SideAttacks{
    pub check: Option<PieceInfo>,
    pub double_check: bool,
    pub nonrays: Bitboard,
    pub rays_h: Bitboard,
    pub rays_v: Bitboard,
    pub rays_dd: Bitboard,
    pub rays_da: Bitboard,
}

pub trait SideAttackMethods{
    fn all(self) -> Bitboard;
}

impl SideAttackMethods for SideAttacks{
    fn all(self) -> Bitboard{
        return self.nonrays | self.rays_h | self.rays_v | self.rays_dd | self.rays_da;
    }
}

#[derive(Copy)]
#[derive(Clone)]
pub struct AbsolutePins{
    pub pins_h: Bitboard,
    pub pins_v: Bitboard,
    pub pins_dd: Bitboard,
    pub pins_da: Bitboard,
}

pub trait AbsolutePinMethods{
    fn all(self) -> Bitboard;
}

impl AbsolutePinMethods for AbsolutePins{
    fn all(self) -> Bitboard{
        return self.pins_h | self.pins_v | self.pins_dd | self.pins_da;
    }
}

impl Castling {
    pub fn new() -> Castling {
        Castling {
            white_king_side: false,
            white_queen_side: false,
            black_king_side: false,
            black_queen_side: false,
        }
    }

    pub fn new_game() -> Castling {
        Castling {
            white_king_side: true,
            white_queen_side: true,
            black_king_side: true,
            black_queen_side: true,
        }
    }

    pub fn get_zobrist_index(self) -> usize{
        let mut index: usize = 0;

        if self.white_king_side{
            index += 1;
        }
        if self.white_queen_side{
            index += 2;
        }
        if self.black_king_side{
            index += 4;
        }
        if self.black_queen_side{
            index += 8;
        }

        return index;
    }
}

#[derive(PartialEq)]
#[derive(Copy)]
#[derive(Clone)]
pub struct Translation {
    pub from: Square,
    pub to: Square,
}

#[derive(PartialEq)]
#[derive(Copy)]
#[derive(Clone)]
pub struct Move{
    pub translation: Option<Translation>,
    pub promotion: Option<Piece>,
    pub capture: Option<Piece>,
    pub castling: Option<CastlingDirection>,
    pub en_passant: Option<Square>,
}

impl Move{
    pub fn get_tstring(&self) -> String{
        let mut tstring: String = String::new();

        if self.translation.is_some(){
            let promotion = if self.promotion.is_some(){
                match self.promotion.unwrap(){
                    KNIGHT => "n",
                    BISHOP => "b",
                    ROOK => "r",
                    QUEEN => "q",
                    _ => panic!("Invalid promotion piece: {}", self.promotion.unwrap())
                }
            }
            else{
                ""
            };
            let from_square: Square = self.translation.as_ref().unwrap().from;
            let to_square: Square = self.translation.as_ref().unwrap().to;
            tstring = format!("{}{}{}", from_square.as_string(), to_square.as_string(), promotion);
        }

        return tstring;
    }

}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.translation.is_some(){
            let mut capture_string: String = String::new();
            if self.capture.is_some(){
                capture_string = "x".to_string();
            }
            let from_square: Square = self.translation.as_ref().unwrap().from;
            let to_square: Square = self.translation.as_ref().unwrap().to;
            write!(f, "{}{}{}", from_square.as_string(), capture_string, to_square.as_string())?;
        }
        else if self.castling.is_some(){
            if self.castling.unwrap() == KING_SIDE{
                write!(f, "O-O")?;
            }
            else{
                write!(f, "O-O-O")?;
            }
        }
        else if self.en_passant.is_some(){
            write!(f, "{}{}", self.en_passant.unwrap() - 8, self.en_passant.unwrap())?;
        }

        if self.promotion.is_some(){
            write!(f, "={}", self.promotion.unwrap().to_notation())?;
        }

        return Ok(());
    }
}

#[derive(PartialEq)]
#[derive(Copy)]
#[derive(Clone)]
pub struct Position{
    pub pieces: [SidePieces; 2],
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub side_to_move: Side,
    pub castling_rights: Castling,
    pub en_passant_square: Option<Square>,
    pub hasher : ZobristHasher,
    pub zobrist_stack: ZobristMoveStack
}

impl Position{

    pub fn new() -> Position{
        Position{
            pieces: [SidePieces::new(), SidePieces::new()],
            halfmove_clock: 0,
            fullmove_number: 1,
            side_to_move: Side::WHITE,
            castling_rights: Castling::new(),
            en_passant_square: None,
            hasher: ZobristHasher::new(),
            zobrist_stack: ZobristMoveStack::new(),
        }
    }

    pub fn new_game() -> Position{
        let pieces = [SidePieces::new_game(Side::WHITE), SidePieces::new_game(Side::BLACK)];
        let halfmove_clock = 0;
        let fullmove_number = 1;
        let side_to_move = Side::WHITE;
        let castling_rights = Castling::new_game();
        let en_passant_square: Option<Square> = None;
        let hasher = ZobristHasher::new();
        let zobrist_stack = ZobristMoveStack::new();

        Position{
            pieces,
            halfmove_clock,
            fullmove_number,
            side_to_move,
            castling_rights,
            en_passant_square,
            hasher,
            zobrist_stack
        }
    }

    pub fn piece_at(&self, square: Square) -> Option<(Piece, Side)>{
        let square_bb = square.to_bitboard();
        let white_pieces = self.pieces[Side::WHITE.0].occupancy();
        let black_pieces = self.pieces[Side::BLACK.0].occupancy();

        if square_bb & white_pieces != 0{
            for piece in 0..6{
                if square_bb & self.pieces[Side::WHITE.0][piece] != 0{
                    return Some((piece, Side::WHITE));
                }
            }
        }
        else if square_bb & black_pieces != 0{
            for piece in 0..6{
                if square_bb & self.pieces[Side::WHITE.0][piece] != 0{
                    return Some((piece, Side::BLACK));
                }
            }
        }
        else{
            return None;
        }

        return None;
    }

    //parse a FEN string into a position
    pub fn from_fen(fen: &str) -> Position{
        let mut position = Position::new();

        //split the FEN string into its components
        let fen_split: Vec<&str> = fen.split(" ").collect();
        
        //get the piece placement
        let piece_placement: Vec<&str> = fen_split[0].split("/").collect();

        for (rank, rank_string) in piece_placement.iter().enumerate(){
            let mut file: usize = 0;
            for c in rank_string.chars(){
                if c.is_digit(10){
                    file += c.to_digit(10).unwrap() as usize;
                }
                else{
                    let piece_and_side = Piece::from_char_board(c);
                    if piece_and_side != None{
                        let piece = piece_and_side.unwrap().0;

                        let side = piece_and_side.unwrap().1;
                        let square = Square::from_rank_and_file(7-rank, file);

                        position.pieces[side.0][piece as usize] |= square.to_bitboard();
                        file += 1;
                    }
                }
            }
        }

        //get the side to move
        position.side_to_move = match fen_split[1]{
            "w" => Side::WHITE,
            "b" => Side::BLACK,
            _ => panic!("Invalid side to move in FEN string")
        };

        //match the castling rights string
        for c in fen_split[2].chars(){
            match c{
                'K' => position.castling_rights.white_king_side = true,
                'Q' => position.castling_rights.white_queen_side = true,
                'k' => position.castling_rights.black_king_side = true,
                'q' => position.castling_rights.black_queen_side = true,
                '-' => break,
                _ => panic!("Invalid castling rights in FEN string")
            }
        }

        //get the en passant square
        position.en_passant_square = match fen_split[3]{
            "-" => None,
            _ => Some(Square::from_string(fen_split[3]))
        };
        
        //get the halfmove clock
        position.halfmove_clock = fen_split[4].parse::<u32>().unwrap();

        //get the fullmove number
        position.fullmove_number = fen_split[5].parse::<u32>().unwrap();     


        return position
    }

    //get fen string of the position
    pub fn to_fen(&self) -> String{
        let mut fen_string: String = String::new();

        //get the piece placement
        for rank in (0..8).rev(){
            let mut empty_squares: u32 = 0;
            for file in 0..8{
                let square = Square::from_rank_and_file(rank, file);
                let piece_info = self.piece_at(square);
                if piece_info.is_some(){
                    if empty_squares > 0{
                        fen_string.push_str(&empty_squares.to_string());
                        empty_squares = 0;
                    }
                    let piece = piece_info.unwrap().0;
                    let side = piece_info.unwrap().1;
                    fen_string.push(piece.to_char_board(side));
                }
                else{
                    empty_squares += 1;
                }
            }
            if empty_squares > 0{
                fen_string.push_str(&empty_squares.to_string());
            }
            if rank > 0{
                fen_string.push('/');
            }
        }

        //get the side to move
        fen_string.push(' ');
        fen_string.push(self.side_to_move.to_char());

        //get the castling rights
        fen_string.push(' ');
        if self.castling_rights.white_king_side{
            fen_string.push('K');
        }
        if self.castling_rights.white_queen_side{
            fen_string.push('Q');
        }
        if self.castling_rights.black_king_side{
            fen_string.push('k');
        }
        if self.castling_rights.black_queen_side{
            fen_string.push('q');
        }
        if !self.castling_rights.white_king_side && !self.castling_rights.white_queen_side && !self.castling_rights.black_king_side && !self.castling_rights.black_queen_side{
            fen_string.push('-');
        }

        //get the en passant square
        fen_string.push(' ');
        if self.en_passant_square.is_some(){
            fen_string.push_str(&self.en_passant_square.unwrap().as_string());
        }
        else{
            fen_string.push('-');
        }

        //get the halfmove clock
        fen_string.push(' ');
        fen_string.push_str(&self.halfmove_clock.to_string());

        //get the fullmove number
        fen_string.push(' ');
        fen_string.push_str(&self.fullmove_number.to_string());

        return fen_string;
    }


    fn get_side_attacks(self, side: Side, occupancy: Bitboard) -> SideAttacks{
        let mut check: Option<PieceInfo> = None;
        let mut double_check: bool = false;
        let mut nonrays: Bitboard = 0;
        let mut rays_h: Bitboard = 0;
        let mut rays_v: Bitboard = 0;
        let mut rays_dd: Bitboard = 0;
        let mut rays_da: Bitboard = 0;

        let enemy_side: Side = !side;
        let enemy_king_square_bb = self.pieces[enemy_side.0][KING];

        //iterate over all pieces
        for i in 0..6{
            let piece_bb = self.pieces[side.0][i];
            for square in piece_bb.get_squares(){
                if i == PAWN{
                    let pawn_attacks = get_pawn_attacks(side, square);
                    if enemy_king_square_bb & pawn_attacks != 0{
                        if check.is_some(){
                            double_check = true;
                        }
                        else{
                            check = Some(PieceInfo{
                                piece: PAWN,
                                square: square,
                            });
                        }
                    }
                    nonrays |= pawn_attacks;
                }
                else if i == KNIGHT{
                    let knight_attacks = get_knight_attacks(square);
                    if enemy_king_square_bb & knight_attacks != 0{
                        if check.is_some(){
                            double_check = true;
                        }
                        else{
                            check = Some(PieceInfo{
                                piece: KNIGHT,
                                square: square,
                            });
                        }
                    }
                    nonrays |= knight_attacks;
                }
                else if i == BISHOP{
                    let bishop_attacks = get_bishop_attacks(square, occupancy);
                    if enemy_king_square_bb & bishop_attacks != 0{
                        if check.is_some(){
                            double_check = true;
                        }
                        else{
                            check = Some(PieceInfo{
                                piece: BISHOP,
                                square: square,
                            });
                        }
                    }
                    rays_dd |= bishop_attacks & DIRECTIONAL_MAP_DD[square as usize];
                    rays_da |= bishop_attacks & DIRECTIONAL_MAP_DA[square as usize];
                }
                else if i == ROOK{
                    let rook_attacks = get_rook_attacks(square, occupancy);
                    if enemy_king_square_bb & rook_attacks != 0{
                        if check.is_some(){
                            double_check = true;
                        }
                        else{
                            check = Some(PieceInfo{
                                piece: ROOK,
                                square: square,
                            });
                        }
                    }
                    rays_h |= rook_attacks & DIRECTIONAL_MAP_RANK[square as usize];
                    rays_v |= rook_attacks & DIRECTIONAL_MAP_FILE[square as usize];
                }
                else if i == QUEEN{
                    let queen_attacks = get_queen_attacks(square, occupancy);
                    if enemy_king_square_bb & queen_attacks != 0{
                        if check.is_some(){
                            double_check = true;
                        }
                        else{
                            check = Some(PieceInfo{
                                piece: QUEEN,
                                square: square,
                            });
                        }
                    }
                    rays_h |= queen_attacks & DIRECTIONAL_MAP_RANK[square as usize];
                    rays_v |= queen_attacks & DIRECTIONAL_MAP_FILE[square as usize];
                    rays_dd |= queen_attacks & DIRECTIONAL_MAP_DD[square as usize];
                    rays_da |= queen_attacks & DIRECTIONAL_MAP_DA[square as usize];
                }
                else if i == KING{
                    let king_attacks = get_king_attacks(square);
                    nonrays |= king_attacks;
                }
            }
        }

        return SideAttacks{
            check,
            double_check,
            nonrays,
            rays_h,
            rays_v,
            rays_dd,
            rays_da
        };
    }
    
    fn get_absolute_pins_for_side(self, enemy_attacks: SideAttacks, occupancy: Bitboard, defender_occupancy: Bitboard, defender_king_square: Square) -> AbsolutePins{
        let mut pins_h: Bitboard = 0;
        let mut pins_v: Bitboard = 0;
        let mut pins_dd: Bitboard = 0;
        let mut pins_da: Bitboard = 0;

        if defender_king_square == 64{
            print_position(&self);

            self.print_position_pieces();

            panic!("defender king square is 64");
        }

        //check attacks horizontal
        let relevant_rank = DIRECTIONAL_MAP_RANK[defender_king_square as usize];
        let king_sees = get_rook_attacks(defender_king_square, occupancy) & relevant_rank & defender_occupancy;
        let enemy_sees = enemy_attacks.rays_h & relevant_rank & defender_occupancy;

        if king_sees & enemy_sees != 0{
            pins_h |= king_sees & enemy_sees;
        }

        //check attacks vertical
        let relevant_file = DIRECTIONAL_MAP_FILE[defender_king_square as usize];
        let king_sees = get_rook_attacks(defender_king_square, occupancy) & relevant_file & defender_occupancy;
        let enemy_sees = enemy_attacks.rays_v & relevant_file & defender_occupancy;
        if king_sees & enemy_sees != 0{
            pins_v |= king_sees & enemy_sees;
        }

        //check attacks diagonal down
        let relevant_dd = DIRECTIONAL_MAP_DD[defender_king_square as usize];
        let king_sees = get_bishop_attacks(defender_king_square, occupancy) & relevant_dd & defender_occupancy;
        let enemy_sees = enemy_attacks.rays_dd & relevant_dd & defender_occupancy;
        if king_sees & enemy_sees != 0{
            pins_dd |= king_sees & enemy_sees;
        }

        //check attacks diagonal up
        let relevant_da = DIRECTIONAL_MAP_DA[defender_king_square as usize];
        let king_sees = get_bishop_attacks(defender_king_square, occupancy) & relevant_da & defender_occupancy;
        let enemy_sees = enemy_attacks.rays_da & relevant_da & defender_occupancy;
        if king_sees & enemy_sees != 0{
            pins_da |= king_sees & enemy_sees;
        }

        //return pins
        return AbsolutePins{
            pins_h,
            pins_v,
            pins_dd,
            pins_da
        };

    }

    fn get_score(self) -> f32{
        return (PIECE_VALUES[PAWN] * (self.pieces[Side::WHITE.0][PAWN].count_ones() as f32 - self.pieces[Side::BLACK.0][PAWN].count_ones() as f32))
               + (PIECE_VALUES[KNIGHT] * (self.pieces[Side::WHITE.0][KNIGHT].count_ones() as f32 - self.pieces[Side::BLACK.0][KNIGHT].count_ones() as f32))
               + (PIECE_VALUES[BISHOP] * (self.pieces[Side::WHITE.0][BISHOP].count_ones() as f32 - self.pieces[Side::BLACK.0][BISHOP].count_ones() as f32))
               + (PIECE_VALUES[ROOK] * (self.pieces[Side::WHITE.0][ROOK].count_ones() as f32 - self.pieces[Side::BLACK.0][ROOK].count_ones() as f32))
               + (PIECE_VALUES[QUEEN] * (self.pieces[Side::WHITE.0][QUEEN].count_ones() as f32 - self.pieces[Side::BLACK.0][QUEEN].count_ones() as f32));
    }

    fn check_draw(&mut self) -> (bool, String){

        //check for 3-fold repetition

        let current_position_hash = self.hasher.hash_position(self);
        self.zobrist_stack.add(current_position_hash);
        let repetitions = self.zobrist_stack.get_repetitions(current_position_hash);
        if repetitions >= 3{
            return (true, "Three-fold, repetition.".to_string());
        }

        //check for 50 move rule
        if self.halfmove_clock >= 100{
            return (true, "Fifty-move rule.".to_string());
        }

        //check for insufficient material
        let mut white_insufficient_material = true;
        let mut black_insufficient_material = true;

            for piece in 0..6{
                if piece != KING{
                    //check pawns
                    if piece == PAWN{
                        if self.pieces[Side::WHITE.0][PAWN] != 0{
                            white_insufficient_material = false;
                        }
                        if self.pieces[Side::BLACK.0][PAWN] != 0{
                            black_insufficient_material = false;
                        }
                    }
                    //check knights
                    else if piece == KNIGHT{
                        if self.pieces[Side::WHITE.0][KNIGHT].count_ones() >= 2{
                            white_insufficient_material = false;
                        }
                        if self.pieces[Side::BLACK.0][KNIGHT].count_ones() >= 2{
                            black_insufficient_material = false;
                        }
                    }
                    //check bishops
                    else if piece == BISHOP{
                        if self.pieces[Side::WHITE.0][BISHOP].count_ones() >= 2{
                            white_insufficient_material = false;
                        }
                        if self.pieces[Side::BLACK.0][BISHOP].count_ones() >= 2{
                            black_insufficient_material = false;
                        }
                    }
                    //check rooks
                    else if piece == ROOK{
                        if self.pieces[Side::WHITE.0][ROOK].count_ones() >= 1{
                            white_insufficient_material = false;
                        }
                        if self.pieces[Side::BLACK.0][ROOK].count_ones() >= 1{
                            black_insufficient_material = false;
                        }
                    }
                    //check queens
                    else if piece == QUEEN{
                        if self.pieces[Side::WHITE.0][QUEEN].count_ones() >= 1{
                            white_insufficient_material = false;
                        }
                        if self.pieces[Side::BLACK.0][QUEEN].count_ones() >= 1{
                            black_insufficient_material = false;
                        }
                    }
                }
            }

        

        if white_insufficient_material && black_insufficient_material{
            return (true, "Insufficient material.".to_string());
        }

        return (false, "".to_string());
    }

    pub fn evaluate(mut self) -> PositionEvaluation{
        let mut moves: Vec<Move> = Vec::new();

        //just return if it's a draw
        let draw_check = self.check_draw();
        if draw_check.0{
            return PositionEvaluation{
                moves,
                game_state: GameState::DRAW,
                state_note: Some(draw_check.1),
                score: Some(0.0)
            }
        }

        let mut game_state: GameState = GameState::ONGOING;

        let us = self.side_to_move;
        let them = !us;

        let our_occupancy = self.pieces[us.0].occupancy();
        let their_occupancy = self.pieces[them.0].occupancy();
        let occupancy = our_occupancy | their_occupancy;

        let our_king: Bitboard = self.pieces[us.0][KING];
        let our_king_square = our_king.to_square();

        let their_king = self.pieces[them.0][KING];
        let their_king_square = their_king.to_square();

        let occupancy_without_our_king = occupancy & !our_king;

        let their_attacks = self.get_side_attacks(them, occupancy);
        let their_attacks_without_our_king = self.get_side_attacks(them, occupancy_without_our_king);

        let our_attacks = self.get_side_attacks(us, occupancy);

        let our_pins = self.get_absolute_pins_for_side(their_attacks, occupancy, our_occupancy, our_king_square);
        let their_pins = self.get_absolute_pins_for_side(our_attacks, occupancy, their_occupancy, their_king_square);

        let pinscore = (our_pins.all().count_ones() as f32 - their_pins.all().count_ones() as f32) * PIN_MULTIPLIER;
        let movescore = (their_attacks.all().count_ones() as f32 - our_attacks.all().count_ones() as f32) * SQUARE_MULTIPLIER;

        let mut score = Some(self.get_score() + pinscore + movescore);

        //make sure king is not in check
        if their_attacks.check.is_none(){
            //generate castling moves
            if us == Side::WHITE{
                if self.castling_rights.white_king_side{
                    //check that the squares between the king and the rook are empty
                    if occupancy & WHITE_KINGSIDE_CASTLE == 0{
                        //check that the squares between the king and the rook are not attacked
                        if their_attacks.all() & WHITE_KINGSIDE_CASTLE == 0{
                            let destination_square = Square::G1;
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: our_king_square,
                                    to: destination_square,
                                }),
                                promotion: None,
                                capture: None,
                                castling: Some(KING_SIDE),
                                en_passant: None, 
                            });
                        }
                    }
                }
                if self.castling_rights.white_queen_side{
                    //check that the squares between the king and the rook are empty
                    if occupancy & WHITE_QUEENSIDE_CASTLE == 0{
                        let white_queenside_squares = Square::C1.to_bitboard() | Square::D1.to_bitboard();
                        //check that the squares between the king and the rook are not attacked
                        if their_attacks.all() & white_queenside_squares == 0{
                            let destination_square = Square::C1;
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: our_king_square,
                                    to: destination_square,
                                }),
                                promotion: None,
                                capture: None,
                                castling: Some(QUEEN_SIDE),
                                en_passant: None, 
                            });
                        }
                    }
                }
            }
            else{
                if self.castling_rights.black_king_side{
                    //check that the squares between the king and the rook are empty
                    if occupancy & BLACK_KINGSIDE_CASTLE == 0{

                        //check that the squares between the king and the rook are not attacked
                        if their_attacks.all() & BLACK_KINGSIDE_CASTLE == 0{
                            let destination_square = Square::G8;
                            
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: our_king_square,
                                    to: destination_square,
                                }),
                                promotion: None,
                                capture: None,
                                castling: Some(KING_SIDE),
                                en_passant: None, 
                            });
                        }
                    }
                }
                if self.castling_rights.black_queen_side{
                    //check that the squares between the king and the rook are empty

                    if occupancy & BLACK_QUEENSIDE_CASTLE == 0{
                        let black_queenside_squares = Square::C8.to_bitboard() | Square::D8.to_bitboard();
                        //check that the squares between the king and the rook are not attacked
                        if their_attacks.all() & black_queenside_squares == 0{
                            let destination_square = Square::C8;
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: our_king_square,
                                    to: destination_square,
                                }),
                                promotion: None,
                                capture: None,
                                castling: Some(QUEEN_SIDE),
                                en_passant: None, 
                            });
                        }
                    }
                }
            }

            //generate pawn moves and captures
            let pawn_bb = self.pieces[us.0][PAWN];
            let pawn_squares = pawn_bb.get_squares();
            for square in pawn_squares{
                let square_bb = square.to_bitboard();
                //if pawn is not pinned horizontally or diagonally, generate pawn moves
                if our_pins.pins_h & square_bb == 0 && our_pins.pins_dd & square_bb == 0 && our_pins.pins_da & square_bb == 0{
                    //generate pawn moves
                    let pawn_moves = get_pawn_moves(us, square, occupancy);
                    let destination_squares = pawn_moves.get_squares();

                    for destination_square in destination_squares{
                        let destination_square_bb = destination_square.to_bitboard();
                        if us == Side::WHITE && destination_square_bb & RANK_8BB != 0 || us == Side::BLACK && destination_square_bb & RANK_1BB != 0{
                            //generate promotion moves
                            for promotion_piece in [QUEEN, ROOK, BISHOP, KNIGHT].iter(){
                                moves.push(Move{
                                    translation: Some(Translation{
                                        from: square,
                                        to: destination_square,
                                    }),
                                    promotion: Some(*promotion_piece),
                                    capture: None,
                                    castling: None,
                                    en_passant: None, 
                                });
                            }
                        }
                        else{
                            //generate non-promotion moves
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: destination_square,
                                }),
                                promotion: None,
                                capture: None,
                                castling: None,
                                en_passant: None, 
                            });
                        }
                    }
                }
                //if pawn is not pinned horizontally or vertically, generate pawn captures
                if our_pins.pins_h & square_bb == 0 && our_pins.pins_v & square_bb == 0{
                    let mut valid_capture_path = Bitboard::FULL;

                    if our_pins.pins_da & square_bb != 0{
                        valid_capture_path = valid_capture_path & DIRECTIONAL_MAP_DA[square as usize];
                    }
                    if our_pins.pins_dd & square_bb != 0{
                        valid_capture_path = valid_capture_path & DIRECTIONAL_MAP_DD[square as usize];
                    }

                    let pawn_attacks = get_pawn_attacks(us, square) & valid_capture_path;
                    
                    //generate normal pawn captures first
                    let pawn_captures = pawn_attacks & their_occupancy;
                    let pawn_capture_squares = pawn_captures.get_squares();

                    for pawn_capture_square in pawn_capture_squares{
                        let pawn_capture_square_bb = pawn_capture_square.to_bitboard();
                        
                        if us == Side::WHITE && pawn_capture_square_bb & RANK_8BB != 0 || us == Side::BLACK && pawn_capture_square_bb & RANK_1BB != 0{
                            //generate promotion captures
                            for promotion_piece in [QUEEN, ROOK, BISHOP, KNIGHT].iter(){
                                moves.push(Move{
                                    translation: Some(Translation{
                                        from: square,
                                        to: pawn_capture_square,
                                    }),
                                    promotion: Some(*promotion_piece),
                                    capture: self.pieces[them.0].get_piece_type_at_square(pawn_capture_square_bb),
                                    castling: None,
                                    en_passant: None, 
                                });
                            }
                        }
                        else{
                            //generate non-promotion captures
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: pawn_capture_square,
                                }),
                                promotion: None,
                                capture: self.pieces[them.0].get_piece_type_at_square(pawn_capture_square_bb),
                                castling: None,
                                en_passant: None, 
                            });
                        }
                    }
                    if self.en_passant_square.is_some(){
                        //generate en passant captures
                        let en_passant_square = self.en_passant_square.unwrap();
                        let en_passant_valid_bb = pawn_attacks & en_passant_square.to_bitboard();

                        if en_passant_valid_bb != 0{
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: en_passant_square,
                                }),
                                promotion: None,
                                capture: Some(PAWN),
                                castling: None,
                                en_passant: Some(en_passant_square),
                            });
                        }
                    }
                }
            }
            
            //generate knight moves
            let knight_bb = self.pieces[us.0][KNIGHT];
            let knight_squares = knight_bb.get_squares();

            for knight in knight_squares{
                let knight_attacks = get_knight_attacks(knight);
                let current_knight_bb = knight.to_bitboard();
                let valid_knight_attacks = knight_attacks & !our_occupancy;

                //if knight is pinned at all, skip generating knight moves
                if our_pins.all() & current_knight_bb == 0{
                    for valid_knight_attack in valid_knight_attacks.get_squares(){
                        let valid_knight_attack_bb = valid_knight_attack.to_bitboard();
                        if valid_knight_attack_bb & their_occupancy != 0{
                            //generate knight captures
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: knight,
                                    to: valid_knight_attack,
                                }),
                                promotion: None,
                                capture: self.pieces[them.0].get_piece_type_at_square(valid_knight_attack_bb),
                                castling: None,
                                en_passant: None, 
                            });
                        }
                        else{
                            //generate knight moves
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: knight,
                                    to: valid_knight_attack,
                                }),
                                promotion: None,
                                capture: None,
                                castling: None,
                                en_passant: None, 
                            });
                        }
                    }
                }
            }

            //generate bishop moves
            let bishop_bb = self.pieces[us.0][BISHOP];
            let bishop_squares = bishop_bb.get_squares();

            for bishop_square in bishop_squares{
                let bishop_attacks = get_bishop_attacks(bishop_square, occupancy) & !our_occupancy;
                let current_bishop_bb = bishop_square.to_bitboard();

                //if bishop is pinned horizontally or vertically, skip generating bishop moves
                if our_pins.pins_h & current_bishop_bb == 0 && our_pins.pins_v & current_bishop_bb == 0{
                    let mut valid_bishop_attacks: Bitboard;
                    
                    //if bishop is pinned diagonally, filter out moves that are not along the pin
                    if our_pins.pins_dd & current_bishop_bb != 0{
                        let bishop_path = DIRECTIONAL_MAP_DD[bishop_square as usize];
                        valid_bishop_attacks = bishop_attacks & bishop_path;
                    }
                    else if our_pins.pins_da & current_bishop_bb != 0{
                        let bishop_path = DIRECTIONAL_MAP_DA[bishop_square as usize];
                        valid_bishop_attacks = bishop_attacks & bishop_path;
                    }
                    //bishop is not pinned
                    else{
                        valid_bishop_attacks = bishop_attacks;
                    }

                    valid_bishop_attacks &= !our_occupancy;

                    for valid_bishop_attack in valid_bishop_attacks.get_squares(){
                        let valid_bishop_attack_bb = valid_bishop_attack.to_bitboard();
                        if valid_bishop_attack_bb & their_occupancy != 0{
                            //generate bishop captures
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: bishop_square,
                                    to: valid_bishop_attack,
                                }),
                                promotion: None,
                                capture: self.pieces[them.0].get_piece_type_at_square(valid_bishop_attack_bb),
                                castling: None,
                                en_passant: None, 
                            });
                        }
                        else{
                            //generate bishop moves
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: bishop_square,
                                    to: valid_bishop_attack,
                                }),
                                promotion: None,
                                capture: None,
                                castling: None,
                                en_passant: None, 
                            });
                        }
                    }
                }
            }

            //generate rook moves
            let rook_bb = self.pieces[us.0][ROOK];

            let rook_squares = rook_bb.get_squares();

            for rook_square in rook_squares{
                let rook_attacks = get_rook_attacks(rook_square, occupancy) & !our_occupancy;

                let current_rook_bb = rook_square.to_bitboard();

                //if rook is pinned diagonally, skip generating rook moves
                if our_pins.pins_dd & current_rook_bb == 0 && our_pins.pins_da & current_rook_bb == 0{
                    let valid_rook_attacks: Bitboard;
                    
                    //if rook is pinned horizontally or vertically, filter out moves that are not along the pin
                    if our_pins.pins_h & current_rook_bb != 0{
                        let rook_path = DIRECTIONAL_MAP_RANK[rook_square as usize];
                        valid_rook_attacks = rook_attacks & rook_path;
                    }
                    else if our_pins.pins_v & current_rook_bb != 0{
                        let rook_path = DIRECTIONAL_MAP_FILE[rook_square as usize];
                        valid_rook_attacks = rook_attacks & rook_path;
                    }
                    //rook is not pinned
                    else{
                        valid_rook_attacks = rook_attacks;
                    }

                    for valid_rook_attack in valid_rook_attacks.get_squares(){
                        let valid_rook_attack_bb = valid_rook_attack.to_bitboard();

                        if valid_rook_attack_bb & their_occupancy != 0{
                            //generate rook captures
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: rook_square,
                                    to: valid_rook_attack,
                                }),
                                promotion: None,
                                capture: self.pieces[them.0].get_piece_type_at_square(valid_rook_attack_bb),
                                castling: None,
                                en_passant: None, 
                            });
                        }
                        else{
                            //generate rook moves
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: rook_square,
                                    to: valid_rook_attack,
                                }),
                                promotion: None,
                                capture: None,
                                castling: None,
                                en_passant: None, 
                            });
                        }
                    }
                }
            }

            //generate queen moves
            let queen_bb = self.pieces[us.0][QUEEN];
            let queen_squares = queen_bb.get_squares();

            for queen_square in queen_squares{
                let queen_attacks = get_queen_attacks(queen_square, occupancy) & !our_occupancy;
                let valid_queen_attacks: Bitboard;
                
                //if queen is pinned in any direction, filter out moves that are not along the pin
                if our_pins.pins_h & queen_bb != 0{
                    let queen_path = DIRECTIONAL_MAP_RANK[queen_square as usize];
                    valid_queen_attacks = queen_attacks & queen_path;
                }
                else if our_pins.pins_v & queen_bb != 0{
                    let queen_path = DIRECTIONAL_MAP_FILE[queen_square as usize];
                    valid_queen_attacks = queen_attacks & queen_path;
                }
                else if our_pins.pins_dd & queen_bb != 0{
                    let queen_path = DIRECTIONAL_MAP_DD[queen_square as usize];
                    valid_queen_attacks = queen_attacks & queen_path;
                }
                else if our_pins.pins_da & queen_bb != 0{
                    let queen_path = DIRECTIONAL_MAP_DA[queen_square as usize];
                    valid_queen_attacks = queen_attacks & queen_path;
                }
                else{
                    valid_queen_attacks = queen_attacks;
                }

                for valid_queen_attack in valid_queen_attacks.get_squares(){
                    let valid_queen_attack_bb = valid_queen_attack.to_bitboard();

                    if valid_queen_attack_bb & their_occupancy != 0{
                        //generate queen captures
                        moves.push(Move{
                            translation: Some(Translation{
                                from: queen_square,
                                to: valid_queen_attack,
                            }),
                            promotion: None,
                            capture: self.pieces[them.0].get_piece_type_at_square(valid_queen_attack_bb),
                            castling: None,
                            en_passant: None, 
                        });
                    }
                    else{
                        //generate queen moves
                        moves.push(Move{
                            translation: Some(Translation{
                                from: queen_square,
                                to: valid_queen_attack,
                            }),
                            promotion: None,
                            capture: None,
                            castling: None,
                            en_passant: None, 
                        });
                    }
                }
            }
            
            //generate king moves
            let king_bb = self.pieces[us.0][KING];
            let king_square = king_bb.get_squares()[0];

            let king_attacks = get_king_attacks(king_square) & !our_occupancy;
            let valid_king_attacks: Bitboard;
            valid_king_attacks = king_attacks & !their_attacks_without_our_king.all();

            for valid_king_attack in valid_king_attacks.get_squares(){
                let valid_king_attack_bb = valid_king_attack.to_bitboard();
                if valid_king_attack_bb & their_occupancy != 0{
                    //generate king captures
                    moves.push(Move{
                        translation: Some(Translation{
                            from: king_square,
                            to: valid_king_attack,
                        }),
                        promotion: None,
                        capture: self.pieces[them.0].get_piece_type_at_square(valid_king_attack_bb),
                        castling: None,
                        en_passant: None, 
                    });
                }
                else{
                    //generate king moves
                    moves.push(Move{
                        translation: Some(Translation{
                            from: king_square,
                            to: valid_king_attack,
                        }),
                        promotion: None,
                        capture: None,
                        castling: None,
                        en_passant: None, 
                    });
                }
            }
            if moves.len() == 0{
                let note = format!("No moves found for {}", us);
                return PositionEvaluation{
                    game_state: GameState::DRAW,
                    state_note: Some(note),
                    moves,
                    score
                }
            }
        }
        else{
            game_state = GameState::CHECK;

            //double check, only king must move
            if their_attacks.double_check{
                let available_squares: Bitboard = (get_king_attacks(our_king_square) & !our_occupancy) & !their_attacks_without_our_king.all();
                //checkmate?
                if available_squares == 0{
                    score = if us == Side::WHITE { Some(SCORE_BLACK_WINS) } else { Some(SCORE_WHITE_WINS) };
                    return PositionEvaluation{
                        game_state: GameState::CHECKMATE,
                        state_note: Some("No moves after check.".to_string()),
                        moves,
                        score
                    }
                }
                //we can still play for one more move at least
                for square in available_squares.get_squares(){
                    let square_bb = square.to_bitboard();
                    if square_bb & their_occupancy != 0{
                        //find which piece the king is attacking
                        let mut piece = 0;
                        for i in 0..6{
                            let pieces_bb = self.pieces[them.0][i];
                            if pieces_bb & square_bb != 0{
                                piece = i;
                                break;
                            }
                        }
                        //add capture move
                        moves.push(Move{
                            translation: Some(Translation { from: our_king_square, to: square }),
                            promotion: None,
                            capture: Some(piece),
                            castling:None,
                            en_passant: None, 
                        });
                    }
                    else{
                        moves.push(Move{
                            translation: Some(Translation { from: our_king_square, to: square }),
                            promotion: None,
                            capture: None,
                            castling:None,
                            en_passant: None, 
                        });
                    }

                }
            }   
            //single checker
            else{
                let checker = their_attacks.check.unwrap();
                let checker_square = checker.square;
                let checker_square_bb = checker_square.to_bitboard();
                let checker_piece = checker.piece;

                let mut slider_squares: Bitboard = Bitboard::EMPTY;

                if checker_piece == BISHOP || checker_piece == ROOK || checker_piece == QUEEN{
                    //find the squares between the king and the checker
                    slider_squares = get_ray_between_squares(our_king_square, checker_square);
                }

                let mut pin_path: Bitboard;

                for piece in 0..6{
                    let piece_bb = self.pieces[us.0][piece];

                    for square in piece_bb.get_squares(){

                        pin_path = Bitboard::FULL;

                        if our_pins.all() & square.to_bitboard() != 0{
                            if piece_bb & our_pins.pins_h != 0{
                                pin_path = DIRECTIONAL_MAP_RANK[square as usize];
                            }
                            else if piece_bb & our_pins.pins_v != 0{
                                pin_path = DIRECTIONAL_MAP_RANK[square as usize];
                            }
                            else if piece_bb & our_pins.pins_da != 0{
                                pin_path = DIRECTIONAL_MAP_DA[square as usize];
                            }
                            else if piece_bb & our_pins.pins_dd != 0{
                                pin_path = DIRECTIONAL_MAP_DD[square as usize];
                            }    
                        }

                        if piece == PAWN{
                            let pawn_attacks = (get_pawn_attacks(us, square) & !our_occupancy) & pin_path;
                            let pawn_move_bb = (get_pawn_moves(us, square, occupancy) & !our_occupancy) & pin_path;
                            let pawn_move = (pawn_move_bb & slider_squares).to_square();

                            if pawn_attacks & checker_square_bb != 0{
                                //pawn capture
                                
                                //generate promotion captures
                                if (pawn_attacks & RANK_1BB != 0) || (pawn_attacks & RANK_8BB != 0){
                                    for promotion in [QUEEN, ROOK, BISHOP, KNIGHT]{
                                        moves.push(Move{
                                            translation: Some(Translation{
                                                from: square,
                                                to: checker_square,
                                            }),
                                            promotion: Some(promotion),
                                            capture: Some(checker_piece),
                                            castling: None,
                                            en_passant: None, 
                                        });
                                    }
                                }
                                else{
                                    moves.push(Move{
                                        translation: Some(Translation{
                                            from: square,
                                            to: checker_square,
                                        }),
                                        promotion: None,
                                        capture: Some(checker_piece),
                                        castling: None,
                                        en_passant: None, 
                                    });
                                }
                            }
                            if pawn_move != Square::NONE{
                                //generate promotion moves
                                if (pawn_move_bb & RANK_1BB != 0) || (pawn_move_bb & RANK_8BB != 0){
                                    for promotion in [QUEEN, ROOK, BISHOP, KNIGHT]{
                                        moves.push(Move{
                                            translation: Some(Translation{
                                                from: square,
                                                to: pawn_move,
                                            }),
                                            promotion: Some(promotion),
                                            capture: None,
                                            castling: None,
                                            en_passant: None, 
                                        });
                                    }
                                }
                                else{
                                    moves.push(Move{
                                        translation: Some(Translation{
                                            from: square,
                                            to: pawn_move,
                                        }),
                                        promotion: None,
                                        capture: None,
                                        castling: None,
                                        en_passant: None, 
                                    });
                                }
                            }
                            if self.en_passant_square.is_some(){
                                //en passant
                                let en_passant_square = self.en_passant_square.unwrap();
                                let en_passant_square_bb = en_passant_square.to_bitboard();
                                let enemy_pawn_square = if us == Side::WHITE { en_passant_square - 8 } else { en_passant_square + 8 };
                                let enemy_pawn_square_bb = enemy_pawn_square.to_bitboard();

                                if pawn_attacks & en_passant_square_bb != 0{
                                    let en_passant_eats_checker = enemy_pawn_square_bb & checker_square_bb != 0;
                                    let en_passant_blocks_checker = en_passant_square_bb & slider_squares != 0;
                                    if en_passant_eats_checker || en_passant_blocks_checker{
                                        moves.push(Move{
                                            translation: Some(Translation { from: square, to: en_passant_square }),
                                            promotion: None,
                                            capture: Some(PAWN),
                                            castling:None,
                                            en_passant: Some(en_passant_square), 
                                        });
                                    }
                                }
                            }   
                        }
                        else if piece == KNIGHT{
                            let knight_attacks = (get_knight_attacks(square) & !our_occupancy) & pin_path;


                            if knight_attacks & checker_square_bb != 0{
                                //knight captures checker
                                moves.push(Move{
                                    translation: Some(Translation { from: square, to: checker_square }),
                                    promotion: None,
                                    capture: Some(checker_piece),
                                    castling:None,
                                    en_passant: None, 
                                });
                            }
                            //check if knight can move to block the check
                            let valid_moves = (knight_attacks & slider_squares) & pin_path;

                            if valid_moves != 0{
                                for valid_move in valid_moves.get_squares(){
                                    moves.push(Move{
                                        translation: Some(Translation { from: square, to: valid_move }),
                                        promotion: None,
                                        capture: None,
                                        castling:None,
                                        en_passant: None, 
                                    });
                                }
                            }
                        }
                        else if piece == BISHOP{
                            let bishop_attacks = (get_bishop_attacks(square, occupancy) & !our_occupancy) & pin_path;

                            if bishop_attacks & checker_square_bb != 0{
                                //bishop captures checker
                                moves.push(Move{
                                    translation: Some(Translation { from: square, to: checker_square }),
                                    promotion: None,
                                    capture: Some(checker_piece),
                                    castling:None,
                                    en_passant: None, 
                                });
                            }
                            let bishop_moves = (bishop_attacks & slider_squares) & pin_path;

                            if bishop_moves != 0{
                                for bishop_move in bishop_moves.get_squares(){
                                    moves.push(Move{
                                        translation: Some(Translation { from: square, to: bishop_move }),
                                        promotion: None,
                                        capture: None,
                                        castling:None,
                                        en_passant: None, 
                                    });
                                }
                            }
                        }
                        else if piece == ROOK{

                            let rook_attacks = (get_rook_attacks(square, occupancy) & !our_occupancy) & pin_path;
                            
                            if rook_attacks & checker_square_bb != 0{
                                //rook captures checker
                                moves.push(Move{
                                    translation: Some(Translation { from: square, to: checker_square }),
                                    promotion: None,
                                    capture: Some(checker_piece),
                                    castling:None,
                                    en_passant: None, 
                                });
                            }
                            let rook_moves = (rook_attacks & slider_squares) & pin_path;

                            if rook_moves != 0{
                                for rook_move in rook_moves.get_squares(){
                                    moves.push(Move{
                                        translation: Some(Translation { from: square, to: rook_move }),
                                        promotion: None,
                                        capture: None,
                                        castling:None,
                                        en_passant: None, 
                                    });
                                }
                            }
                        }
                        else if piece == QUEEN{
                            let queen_attacks = (get_queen_attacks(square, occupancy) & !our_occupancy) & pin_path;

                            if queen_attacks & checker_square_bb != 0{
                                //queen captures checker
                                moves.push(Move{
                                    translation: Some(Translation { from: square, to: checker_square }),
                                    promotion: None,
                                    capture: Some(checker_piece),
                                    castling:None,
                                    en_passant: None, 
                                });
                            }

                            let queen_moves = (queen_attacks & slider_squares) & pin_path;

                            if queen_moves != 0{
                                for queen_move in queen_moves.get_squares(){
                                    moves.push(Move{
                                        translation: Some(Translation { from: square, to: queen_move }),
                                        promotion: None,
                                        capture: None,
                                        castling:None,
                                        en_passant: None, 
                                    });
                                }
                            }
                        }
                        else if piece == KING{

                            let mut valid_attacks = get_king_attacks(square) & !our_occupancy;
                            valid_attacks &= !their_attacks_without_our_king.all();

                            for attack in valid_attacks.get_squares(){
                                let attack_bb = attack.to_bitboard();
                                if attack_bb & checker_square_bb != 0{
                                    moves.push(Move{
                                        translation: Some(Translation { from: square, to: attack }),
                                        promotion: None,
                                        capture: Some(checker_piece),
                                        castling:None,
                                        en_passant: None, 
                                    });
                                }
                                else if attack_bb & their_occupancy != 0{
                                    //find which piece the king is attacking
                                    let piece = self.pieces[them.0].get_piece_type_at_square(attack_bb);
                                    //king eats the piece
                                    moves.push(Move{
                                        translation: Some(Translation { from: square, to: attack }),
                                        promotion: None,
                                        capture: piece,
                                        castling:None,
                                        en_passant: None, 
                                    });
                                }
                                else{
                                    //normal king move
                                    moves.push(Move{
                                        translation: Some(Translation { from: square, to: attack }),
                                        promotion: None,
                                        capture: None,
                                        castling:None,
                                        en_passant: None, 
                                    });
                                }
                            }
                        }   
                    }
                }    
                //no moves available after check
                if moves.is_empty(){
                    score = if us == Side::WHITE { Some(SCORE_BLACK_WINS) } else { Some(SCORE_WHITE_WINS) };
                    return PositionEvaluation{
                        game_state: GameState::CHECKMATE,
                        state_note: Some("No moves after check.".to_string()),
                        moves,
                        score
                    }
                }
            }
        }

        return PositionEvaluation{
            game_state,
            state_note: None,
            moves,
            score
        };
    }

    pub fn print_position_pieces(&self){
        println!("White Pieces:");
        for piece in 0..6{
            let piece_type = piece;
            let piece_bb = self.pieces[0][piece_type];
            let piece_num = piece_bb.count_ones();
            println!("{}: {}", PIECES[piece_type], piece_num);
        }
        println!("Black Pieces:");
        for piece in 0..6{
            let piece_type = piece;
            let piece_bb = self.pieces[1][piece_type];
            let piece_num = piece_bb.count_ones();
            println!("{}: {}", PIECES[piece_type], piece_num);
        }
    }

    pub fn make_move(&self, m: Move) -> Position{
        let mut new_position = self.clone();
        
        let us = self.side_to_move;

        new_position.en_passant_square = None;
        new_position.side_to_move = !us;

        //if the move is not a castle and includes a translation
        if m.castling.is_none() && m.translation.is_some(){
            let translation = m.translation.unwrap();
            let from_piece_wrapped = self.pieces[us.0].get_piece_type_at_square(translation.from.to_bitboard());
            if from_piece_wrapped.is_none(){
                panic!("No piece at the from square!");
            }
            let from_piece = from_piece_wrapped.unwrap();

            if from_piece == PAWN{
                //check if en passant is involved
                if m.en_passant.is_some(){
                        new_position.pieces[us.0][PAWN] = new_position.pieces[us.0][PAWN].set_bit(translation.to);
                        //remove the captured pawn
                        let their_pawn = if us == Side::WHITE { translation.to - 8 } else { translation.to + 8 };
                        new_position.pieces[(!us).0][PAWN] = new_position.pieces[(!us).0][PAWN].unset_bit(their_pawn);
                        //remove original pawn
                        new_position.pieces[us.0][PAWN] = new_position.pieces[us.0][PAWN].unset_bit(translation.from);                        
                }
                else{
                    //check if en passant is possible

                    if translation.to > 16 && translation.to == translation.from + 16 || translation.to == translation.from.wrapping_sub(16){
                        //check if pawn has enemy pawn next on the to square
                        let to_side_bb = translation.to.to_bitboard() << 1 | translation.to.to_bitboard() >> 1;
                        if to_side_bb & self.pieces[(!us).0][PAWN] != 0{
                            new_position.en_passant_square = if us == Side::WHITE { Some(translation.to - 8) } else { Some(translation.to + 8) };
                        }
                    }

                    //check if promotion is involved
                    if m.promotion.is_some(){
                        let promotion = m.promotion.unwrap();
                        new_position.pieces[us.0][promotion] = new_position.pieces[us.0][promotion].set_bit(translation.to);
                    }
                    else{
                        new_position.pieces[us.0][PAWN] = new_position.pieces[us.0][PAWN].set_bit(translation.to);
                    }

                    //check if a capture is involved
                    if m.capture.is_some(){
                        let capture = m.capture.unwrap();
                        new_position.pieces[(!us).0][capture] = new_position.pieces[(!us).0][capture].unset_bit(translation.to);
                    }

                    new_position.en_passant_square = None;
                    new_position.pieces[us.0][PAWN] = new_position.pieces[us.0][PAWN].unset_bit(translation.from);
                }
                new_position.halfmove_clock = 0;
            }
            else{
                //check if king or rook is moving
                if from_piece == KING{
                    if us == Side::WHITE{
                        new_position.castling_rights.white_king_side = false;
                        new_position.castling_rights.white_queen_side = false;
                    }
                    else{
                        new_position.castling_rights.black_king_side = false;
                        new_position.castling_rights.black_queen_side = false;
                    }
                }
                else if from_piece == ROOK{
                    if us == Side::WHITE{
                        if translation.from == 0{
                            new_position.castling_rights.white_queen_side = false;
                        }
                        else if translation.from == 7{
                            new_position.castling_rights.white_king_side = false;
                        }
                    }
                    else{
                        if translation.from == 56{
                            new_position.castling_rights.black_queen_side = false;
                        }
                        else if translation.from == 63{
                            new_position.castling_rights.black_king_side = false;
                        }
                    }
                }
                
                new_position.pieces[us.0][from_piece] = new_position.pieces[us.0][from_piece].set_bit(translation.to);
                new_position.pieces[us.0][from_piece] = new_position.pieces[us.0][from_piece].unset_bit(translation.from);

                //non-pawn move, increment the halfmove clock
                new_position.halfmove_clock += 1;

                //check if a capture is involved
                if m.capture.is_some(){
                    let capture = m.capture.unwrap();
                    new_position.pieces[(!us).0][capture] = new_position.pieces[(!us).0][capture].unset_bit(translation.to);
                    new_position.halfmove_clock = 0;
                }

                new_position.en_passant_square = None;
            }
        }
        //castling
        else if m.castling.is_some(){
            new_position.halfmove_clock += 1;

            if us == Side::WHITE{
                let white_king = new_position.pieces[us.0][KING].to_square();

                if m.castling.unwrap() == KING_SIDE{
                    new_position.pieces[us.0][KING] = new_position.pieces[us.0][KING].unset_bit(white_king);
                    new_position.pieces[us.0][KING] = new_position.pieces[us.0][KING].set_bit(white_king + 2);
                                                                                     
                    new_position.pieces[us.0][ROOK] = new_position.pieces[us.0][ROOK].unset_bit(white_king + 3);
                    new_position.pieces[us.0][ROOK] = new_position.pieces[us.0][ROOK].set_bit(white_king + 1);
                }
                else if m.castling.unwrap() == QUEEN_SIDE{
                    new_position.pieces[us.0][KING] = new_position.pieces[us.0][KING].unset_bit(white_king);
                    new_position.pieces[us.0][KING] = new_position.pieces[us.0][KING].set_bit(white_king - 2);
                                                                                     
                    new_position.pieces[us.0][ROOK] = new_position.pieces[us.0][ROOK].unset_bit(white_king - 4);
                    new_position.pieces[us.0][ROOK] = new_position.pieces[us.0][ROOK].set_bit(white_king - 1);
                }
                else{
                    panic!("Invalid castling move!");
                }
            }
            else{
                let black_king = new_position.pieces[us.0][KING].to_square();

                if m.castling.unwrap() == KING_SIDE{
                    new_position.pieces[us.0][KING] = new_position.pieces[us.0][KING].unset_bit(black_king);
                    new_position.pieces[us.0][KING] = new_position.pieces[us.0][KING].set_bit(black_king + 2);

                    new_position.pieces[us.0][ROOK] = new_position.pieces[us.0][ROOK].unset_bit(black_king + 3);
                    new_position.pieces[us.0][ROOK] = new_position.pieces[us.0][ROOK].set_bit(black_king + 1);
                }
                else if m.castling.unwrap() == QUEEN_SIDE{
                    new_position.pieces[us.0][KING] = new_position.pieces[us.0][KING].unset_bit(black_king);
                    new_position.pieces[us.0][KING] = new_position.pieces[us.0][KING].set_bit(black_king - 2);
                                                                                     
                    new_position.pieces[us.0][ROOK] = new_position.pieces[us.0][ROOK].unset_bit(black_king - 4);
                    new_position.pieces[us.0][ROOK] = new_position.pieces[us.0][ROOK].set_bit(black_king - 1);
                }
                else{
                    panic!("Invalid castling move!");
                }
            }
        }
        else{
            panic!("Unidentified move!");
        }

        if us == Side::BLACK{
            new_position.fullmove_number += 1;
        }
        //if pawn and bishop overlap in new position print
        /* 
        if new_position.pieces[us.0].occupancy() & new_position.pieces[us.0][BISHOP] != 0{
            //get piece that is moving in from 
            let eval = self.evaluate();
            println!("MOVE: {}  ", m);
            println!("GAMESTATE: {}", eval.game_state);
            print_position(self);
            panic!("BISHOP OVERLAP!");
        }
        */

        return new_position;
    }
}

