use core::panic;
use std::{collections::HashMap, fmt::{Display, Formatter, Result}};

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
        }, 
    masks::{
        get_file_mask, 
        get_rank_mask
    }, display::print_bitboard
    };

pub struct PositionEvaluation{
    pub moves: Vec<Move>,
    pub game_state: GameState,
    pub score: Option<i32>
}

const SCORE_WHITE_WINS: i32 = 1000000;
const SCORE_BLACK_WINS: i32 = -1000000;
const SCORE_DRAW: i32 = 0;

const PIECE_VALUES: [i32; 6] = [
    100,
    300,
    300,
    500,
    900,
    0
];

pub type SidePieces = [Bitboard; 6];

pub trait SidePiecesMethods{
    fn occupancy(&self) -> Bitboard;
    fn get_piece_type_at_square(&self, square: Square) -> Option<Piece>;
}

impl SidePiecesMethods for SidePieces{
    fn occupancy(&self) -> Bitboard{
        let mut occupancy = Bitboard::EMPTY;
        for pieces in self.iter(){
            occupancy |= *pieces;
        }
        return occupancy;
    }

    fn get_piece_type_at_square(&self, square: Square) -> Option<Piece>{
        for x in 0..6{
            if self[x] & Bitboard::from_square(square) != 0{
                return Some(x);
            }
        }
        return None;
    }
}

fn new_game_pieces(side: Side) -> SidePieces{
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
        let mut side_to_move_hash: u64 = 0;

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
                    if position.pieces[side][piece] & Bitboard::from_square(square) != 0{
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
        let mut repetitions: usize = 0;

        //traverse the array backwards
        for i in (0..MAX_ZOBRIST_ARRAY_SIZE).rev(){
            if self.zobrist_array[i] == zobrist_hash{
                repetitions += 1;
                if repetitions >= 3{
                    return repetitions;
                }
            }
        }

        return repetitions;
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

#[derive(Copy)]
#[derive(Clone)]
pub struct Translation {
    pub from: Square,
    pub to: Square,
}

#[derive(Copy)]
#[derive(Clone)]
pub struct Move{
    pub translation: Option<Translation>,
    pub promotion: Option<Piece>,
    pub capture: Option<Piece>,
    pub castling: Option<CastlingDirection>,
    pub en_passant: Option<Square>,
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
            write!(f, "{}{}{}", from_square.get_string(), capture_string, to_square.get_string())?;
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
            write!(f, "^^{}", self.promotion.unwrap())?;
        }

        return Ok(());
    }
}

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
        let pieces = [new_game_pieces(Side::WHITE), new_game_pieces(Side::BLACK)];
        let halfmove_clock = 0;
        let fullmove_number = 1;
        let side_to_move = Side::WHITE;
        let castling_rights = Castling::new();
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

    fn get_occupancy(self, side: Option<Side>) -> Bitboard{
        let mut occupancy: Bitboard = 0;

        if side.is_none() {
            for i in 0..2{
                for j in 0..6{
                    occupancy |= self.pieces[i][j];
                }
            }
        }
        else{
            for i in 0..6{
                occupancy |= self.pieces[side.as_ref().unwrap().0][i as usize];
            }
        }
        
        return occupancy;
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
                    rays_dd = bishop_attacks & DIRECTIONAL_MAP_DD[square as usize];
                    rays_da = bishop_attacks & DIRECTIONAL_MAP_DA[square as usize];
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
                    rays_h = rook_attacks & DIRECTIONAL_MAP_FILE[square as usize];
                    rays_v = rook_attacks & DIRECTIONAL_MAP_RANK[square as usize];
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
                    rays_h = queen_attacks & DIRECTIONAL_MAP_FILE[square as usize];
                    rays_v = queen_attacks & DIRECTIONAL_MAP_RANK[square as usize];
                    rays_dd = queen_attacks & DIRECTIONAL_MAP_DD[square as usize];
                    rays_da = queen_attacks & DIRECTIONAL_MAP_DA[square as usize];
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

    fn get_absolute_pins_for_side(self, enemy_attacks: SideAttacks, defender_occupancy: Bitboard, defender_king_square: Square) -> AbsolutePins{
        let mut pins_h: Bitboard = 0;
        let mut pins_v: Bitboard = 0;
        let mut pins_dd: Bitboard = 0;
        let mut pins_da: Bitboard = 0;

        //check attacks horizontal
        let relevant_rank = DIRECTIONAL_MAP_RANK[defender_king_square as usize];
        let king_sees = get_rook_attacks(defender_king_square, defender_occupancy) & relevant_rank & defender_occupancy;
        let enemy_sees = enemy_attacks.rays_h & relevant_rank & defender_occupancy;
        if king_sees & enemy_sees != 0{
            pins_h |= king_sees & enemy_sees;
        }

        //check attacks vertical
        let relevant_file = DIRECTIONAL_MAP_FILE[defender_king_square as usize];
        let king_sees = get_rook_attacks(defender_king_square, defender_occupancy) & relevant_file & defender_occupancy;
        let enemy_sees = enemy_attacks.rays_v & relevant_file & defender_occupancy;
        if king_sees & enemy_sees != 0{
            pins_v |= king_sees & enemy_sees;
        }

        //check attacks diagonal down
        let relevant_dd = DIRECTIONAL_MAP_DD[defender_king_square as usize];
        let king_sees = get_bishop_attacks(defender_king_square, defender_occupancy) & relevant_dd & defender_occupancy;
        let enemy_sees = enemy_attacks.rays_dd & relevant_dd & defender_occupancy;
        if king_sees & enemy_sees != 0{
            pins_dd |= king_sees & enemy_sees;
        }

        //check attacks diagonal up
        let relevant_da = DIRECTIONAL_MAP_DA[defender_king_square as usize];
        let king_sees = get_bishop_attacks(defender_king_square, defender_occupancy) & relevant_da & defender_occupancy;
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

    fn get_raw_score(self) -> i32{
        //iterate through all pieces and add their values
        let mut white_score = 0;
        let mut black_score = 0;

        let white_pieces = self.pieces[Side::WHITE.0];
        let black_pieces = self.pieces[Side::BLACK.0];

        for i in 0..6{
            let white_piece = white_pieces[i];
            let black_piece = black_pieces[i];

            white_score += white_piece.count_ones() as i32 * PIECE_VALUES[i];
            black_score += black_piece.count_ones() as i32 * PIECE_VALUES[i];
        }


        return white_score - black_score;
    }

    fn check_draw(&mut self) -> bool{

        //check for 3-fold repetition
        let current_position_hash = self.hasher.hash_position(self);
        self.zobrist_stack.add(current_position_hash);
        let repetitions = self.zobrist_stack.get_repetitions(current_position_hash);
        if repetitions >= 3{
            return true;
        }

        //check for 50 move rule
        if self.halfmove_clock >= 100{
            return true;
        }

        //check for insufficient material
        let mut white_insufficient_material = true;
        let mut black_insufficient_material = true;

        for side in 0..1{
            for piece in 0..6{
                if piece == PAWN{
                    let pawn_bb = self.pieces[side][piece];
                    let num_pawns = pawn_bb.count_ones();
                    if num_pawns > 0{
                        if side == 0{
                            white_insufficient_material = false;
                        }
                        else{
                            black_insufficient_material = false;
                        }
                    }
                }
                else if piece == KNIGHT{
                    let knight_bb = self.pieces[side][piece];
                    let num_knights = knight_bb.count_ones();
                    if num_knights > 1{
                        if side == 0{
                            white_insufficient_material = false;
                        }
                        else{
                            black_insufficient_material = false;
                        }
                    }
                }
                else if piece == BISHOP{
                    let bishop_bb = self.pieces[side][piece];
                    let num_bishops = bishop_bb.count_ones();
                    if num_bishops > 1{
                        if side == 0{
                            white_insufficient_material = false;
                        }
                        else{
                            black_insufficient_material = false;
                        }
                    }
                }
                else if piece == ROOK{
                    let rook_bb = self.pieces[side][piece];
                    let num_rooks = rook_bb.count_ones();
                    if num_rooks > 0{
                        if side == 0{
                            white_insufficient_material = false;
                        }
                        else{
                            black_insufficient_material = false;
                        }
                    }
                }
                else if piece == QUEEN{
                    let queen_bb = self.pieces[side][piece];
                    let num_queens = queen_bb.count_ones();
                    if num_queens > 0{
                        if side == 0{
                            white_insufficient_material = false;
                        }
                        else{
                            black_insufficient_material = false;
                        }
                    }
                }
            }

        }

        if white_insufficient_material && black_insufficient_material{
            return true;
        }

        return false;
    }

    pub fn evaluate(mut self, debug: Option<bool>) -> PositionEvaluation{
        let mut moves: Vec<Move> = Vec::new();

        //just return if it's a draw
        if self.check_draw(){
            return PositionEvaluation{
                moves,
                game_state: GameState::DRAW,
                score: Some(SCORE_DRAW)
            }
        }

        let mut game_state: GameState = GameState::IN_PROGRESS;

        let us = self.side_to_move;
        let them = !us;

        let our_occupancy = self.pieces[us.0].occupancy();
        let their_occupancy = self.pieces[them.0].occupancy();
        let occupancy = our_occupancy | their_occupancy;

        let our_king: Bitboard = self.pieces[us.0][KING];
        let our_king_square = our_king.to_square();
        let their_attacks = self.get_side_attacks(them, occupancy);
        let our_pins = self.get_absolute_pins_for_side(their_attacks, our_occupancy, our_king.to_square());
        
        let mut score = Some(self.get_raw_score());

        //make sure king is not in check
        if their_attacks.check.is_none(){
            if(debug == Some(true)){
                println!("No check");
            }
            //generate castling moves
            if us == Side::WHITE{
                if self.castling_rights.white_king_side{
                    //check that the squares between the king and the rook are empty
                    if our_occupancy & WHITE_KINGSIDE_CASTLE == 0{
                        //check that the squares between the king and the rook are not attacked
                        if their_attacks.all() & WHITE_KINGSIDE_CASTLE == 0{
                            moves.push(Move{
                                translation: None,
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
                    if our_occupancy & WHITE_QUEENSIDE_CASTLE == 0{
                        //check that the squares between the king and the rook are not attacked
                        if their_attacks.all() & WHITE_QUEENSIDE_CASTLE == 0{
                            moves.push(Move{
                                translation: None,
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
                    if our_occupancy & BLACK_KINGSIDE_CASTLE == 0{
                        //check that the squares between the king and the rook are not attacked
                        if their_attacks.all() & BLACK_KINGSIDE_CASTLE == 0{
                            moves.push(Move{
                                translation: None,
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
                    if our_occupancy & BLACK_QUEENSIDE_CASTLE == 0{
                        //check that the squares between the king and the rook are not attacked
                        if their_attacks.all() & BLACK_QUEENSIDE_CASTLE == 0{
                            moves.push(Move{
                                translation: None,
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
                let square_bb = Bitboard::from_square(square);
                //if pawn is not pinned horizontally or diagonally, generate pawn moves
                if our_pins.pins_h & square_bb == 0 && our_pins.pins_dd & square_bb == 0 && our_pins.pins_da & square_bb == 0{
                    //generate pawn moves
                    let pawn_moves = get_pawn_moves(us, square, occupancy);
                    let destination_squares = pawn_moves.get_squares();
                    for destination_square in destination_squares{
                        let destination_square_bb = Bitboard::from_square(destination_square);
                        if us == Side::WHITE && destination_square_bb & RANK_8BB != 0{
                            //generate white promotion moves
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: destination_square,
                                }),
                                promotion: Some(QUEEN),
                                capture: None,
                                castling: None,
                                en_passant: None, 
                            });
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: destination_square,
                                }),
                                promotion: Some(ROOK),
                                capture: None,
                                castling: None,
                                en_passant: None, 
                            });
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: destination_square,
                                }),
                                promotion: Some(BISHOP),
                                capture: None,
                                castling: None,
                                en_passant: None, 
                            });
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: destination_square,
                                }),
                                promotion: Some(KNIGHT),
                                capture: None,
                                castling: None,
                                en_passant: None, 
                            });
                        }
                        else if us == Side::BLACK && destination_square_bb & RANK_1BB != 0{
                            //generate black promotion moves
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: destination_square,
                                }),
                                promotion: Some(QUEEN),
                                capture: None,
                                castling: None,
                                en_passant: None, 
                            });
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: destination_square,
                                }),
                                promotion: Some(ROOK),
                                capture: None,
                                castling: None,
                                en_passant: None, 
                            });
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: destination_square,
                                }),
                                promotion: Some(BISHOP),
                                capture: None,
                                castling: None,
                                en_passant: None, 
                            });
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: destination_square,
                                }),
                                promotion: Some(KNIGHT),
                                capture: None,
                                castling: None,
                                en_passant: None, 
                            });
                        }
                        else{
                            //check if destination_square two squares back is identical to square
                            let two_sq_back: u8 = if us == Side::WHITE{
                                destination_square - 16
                            }
                            else{
                                destination_square + 16
                            };

                            let mut added_as_en_passant = false;

                            if square == two_sq_back{
                                let destination_square_bb = Bitboard::from_square(destination_square);
                                let sides_of_destination_square = destination_square_bb << 1 | destination_square_bb >> 1;
                                
                                if sides_of_destination_square & self.pieces[them.0][PAWN] != 0{
                                    moves.push(Move{
                                        translation: Some(Translation{
                                            from: square,
                                            to: destination_square,
                                        }),
                                        promotion: None,
                                        capture: None,
                                        castling: None,
                                        en_passant: Some(destination_square), 
                                    });
                                    added_as_en_passant = true;
                                }
                            }

                            //generate non-promotion moves
                            if !added_as_en_passant{
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
                }
                //if pawn is not pinned horizontally or vertically, generate pawn captures
                else if our_pins.pins_h & square_bb == 0 && our_pins.pins_v & square_bb == 0{
                    let pawn_attacks = get_pawn_attacks(us, square);
                    
                    //generate normal pawn captures first
                    let pawn_captures = pawn_attacks & their_occupancy;
                    let pawn_capture_squares = pawn_captures.get_squares();

                    for pawn_capture_square in pawn_capture_squares{
                        let pawn_capture_square_bb = Bitboard::from_square(pawn_capture_square);
                        
                        if us == Side::WHITE && pawn_capture_square_bb & RANK_8BB != 0{
                            //generate white promotion captures
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: pawn_capture_square,
                                }),
                                promotion: Some(QUEEN),
                                capture: self.pieces[them.0].get_piece_type_at_square(square),
                                castling: None,
                                en_passant: None, 
                            });
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: pawn_capture_square,
                                }),
                                promotion: Some(ROOK),
                                capture: self.pieces[them.0].get_piece_type_at_square(square),
                                castling: None,
                                en_passant: None, 
                            });
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: pawn_capture_square,
                                }),
                                promotion: Some(BISHOP),
                                capture: self.pieces[them.0].get_piece_type_at_square(square),
                                castling: None,
                                en_passant: None, 
                            });
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: pawn_capture_square,
                                }),
                                promotion: Some(KNIGHT),
                                capture: self.pieces[them.0].get_piece_type_at_square(square),
                                castling: None,
                                en_passant: None, 
                            });
                        }
                        else if us == Side::BLACK && pawn_capture_square_bb & RANK_1BB != 0{
                            //generate black promotion captures
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: pawn_capture_square,
                                }),
                                promotion: Some(QUEEN),
                                capture: self.pieces[them.0].get_piece_type_at_square(square),
                                castling: None,
                                en_passant: None, 
                            });
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: pawn_capture_square,
                                }),
                                promotion: Some(ROOK),
                                capture: self.pieces[them.0].get_piece_type_at_square(square),
                                castling: None,
                                en_passant: None, 
                            });
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: pawn_capture_square,
                                }),
                                promotion: Some(BISHOP),
                                capture: self.pieces[them.0].get_piece_type_at_square(square),
                                castling: None,
                                en_passant: None, 
                            });
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: pawn_capture_square,
                                }),
                                promotion: Some(KNIGHT),
                                capture: self.pieces[them.0].get_piece_type_at_square(square),
                                castling: None,
                                en_passant: None, 
                            });
                        }
                        else{
                            //generate non-promotion captures
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: pawn_capture_square,
                                }),
                                promotion: None,
                                capture: self.pieces[them.0].get_piece_type_at_square(square),
                                castling: None,
                                en_passant: None, 
                            });
                        }
                    }
                    //if pawn is not pinned horizontally or vertically, generate en passant captures if possible
                    if self.en_passant_square.is_some(){
                        let en_passant_capture_square = if us == Side::WHITE { self.en_passant_square.unwrap() + 8 } else { self.en_passant_square.unwrap() - 8 };
                        let en_passant_valid_bb = pawn_attacks & Bitboard::from_square(en_passant_capture_square);

                        if en_passant_valid_bb != 0{
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: square,
                                    to: en_passant_capture_square,
                                }),
                                promotion: None,
                                capture: Some(PAWN),
                                castling: None,
                                en_passant: Some(self.en_passant_square.unwrap()),
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
                let current_knight_bb = Bitboard::from_square(knight);
                let valid_knight_attacks = knight_attacks & !our_occupancy;

                //if knight is pinned at all, skip generating knight moves
                if our_pins.all() & current_knight_bb == 0{
                    for valid_knight_attack in valid_knight_attacks.get_squares(){
                        let valid_knight_attack_bb = Bitboard::from_square(valid_knight_attack);
                        if valid_knight_attack_bb & their_occupancy != 0{
                            //generate knight captures
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: knight,
                                    to: valid_knight_attack,
                                }),
                                promotion: None,
                                capture: self.pieces[them.0].get_piece_type_at_square(valid_knight_attack),
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
                let current_bishop_bb = Bitboard::from_square(bishop_square);

                //if bishop is pinned horizontally or vertically, skip generating bishop moves
                if our_pins.pins_h & current_bishop_bb == 0 && our_pins.pins_v & current_bishop_bb == 0{
                    let valid_bishop_attacks: Bitboard;
                    
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

                    for valid_bishop_attack in valid_bishop_attacks.get_squares(){
                        let valid_bishop_attack_bb = Bitboard::from_square(valid_bishop_attack);
                        if valid_bishop_attack_bb & their_occupancy != 0{
                            //generate bishop captures
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: bishop_square,
                                    to: valid_bishop_attack,
                                }),
                                promotion: None,
                                capture: self.pieces[them.0].get_piece_type_at_square(valid_bishop_attack),
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
                let current_rook_bb = Bitboard::from_square(rook_square);

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
                        let valid_rook_attack_bb = Bitboard::from_square(valid_rook_attack);
                        if valid_rook_attack_bb & their_occupancy != 0{
                            //generate rook captures
                            moves.push(Move{
                                translation: Some(Translation{
                                    from: rook_square,
                                    to: valid_rook_attack,
                                }),
                                promotion: None,
                                capture: self.pieces[them.0].get_piece_type_at_square(valid_rook_attack),
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
                    let valid_queen_attack_bb = Bitboard::from_square(valid_queen_attack);
                    if valid_queen_attack_bb & their_occupancy != 0{
                        //generate queen captures
                        moves.push(Move{
                            translation: Some(Translation{
                                from: queen_square,
                                to: valid_queen_attack,
                            }),
                            promotion: None,
                            capture: self.pieces[them.0].get_piece_type_at_square(valid_queen_attack),
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
        }
        else{
            let our_attacks_all = self.get_side_attacks(us, occupancy).all();

            //in check 
            if(debug == Some(true)){
                println!("In Check");
                println!("Side to Move: {}", us);
                println!("Their attacks: ");
                print_bitboard(their_attacks.all());
                println!("Our attacks: ");
                print_bitboard(our_attacks_all);
            }

            let occupancy_without_our_king = occupancy & !our_king;
            let their_attacks_without_our_king = self.get_side_attacks(them, occupancy_without_our_king);

            //double check, only king must move
            if their_attacks.double_check{
                if(debug == Some(true)){
                    println!("Double Check");
                }
                let available_squares: Bitboard = (get_king_attacks(our_king_square) & !our_occupancy) & !their_attacks_without_our_king.all();
                //checkmate?
                if available_squares == 0{
                    if us == Side::WHITE{
                        return PositionEvaluation{
                            game_state: GameState::CHECKMATE,
                            moves,
                            score: Some(SCORE_BLACK_WINS)
                        };
                    }
                    else{
                        return PositionEvaluation{
                            game_state: GameState::CHECKMATE,
                            moves,
                            score: Some(SCORE_WHITE_WINS)
                        };
                    }

                }
                //we can still play for one more move at least
                game_state = GameState::CHECK;
                for square in available_squares.get_squares(){
                    let square_bb = Bitboard::from(square);
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
                let checker_square_bb = Bitboard::from(checker_square);
                let checker_piece = checker.piece;

                let between_bb = get_ray_between_squares(our_king_square, checker_square);
                //Make sure we can even capture or block the checker
                if our_attacks_all & checker_square_bb != 0 || our_attacks_all & between_bb != 0 {
                    for piece in 0..6{
                        let piece_bb = self.pieces[us.0][piece];
                        for square in piece_bb.get_squares(){
                            let square_bb = Bitboard::from(square);
                            if piece == PAWN{
                                let pawn_attacks = get_pawn_attacks(us, square);
                                //make sure pawn can capture the checker
                                if pawn_attacks & checker_square_bb != 0{
                                    //if the pawn is pinned vertically or horizontally, it can't capture the checker
                                    if our_pins.pins_h & square_bb == 0 && our_pins.pins_v & square_bb == 0{
                                        //if the pawn will promote, we need to generate all promotion moves
                                        if (us == Side::WHITE && checker_square_bb & RANK_8BB == 0) || (us == Side::BLACK && checker_square_bb & RANK_1BB == 0){
                                            moves.push(Move{
                                                translation: Some(Translation { from: square, to: checker_square }),
                                                promotion: Some(QUEEN),
                                                capture: Some(checker_piece),
                                                castling:None,
                                                en_passant: None, 
                                            });
                                            moves.push(Move{
                                                translation: Some(Translation { from: square, to: checker_square }),
                                                promotion: Some(ROOK),
                                                capture: Some(checker_piece),
                                                castling:None,
                                                en_passant: None, 
                                            });
                                            moves.push(Move{
                                                translation: Some(Translation { from: square, to: checker_square }),
                                                promotion: Some(BISHOP),
                                                capture: Some(checker_piece),
                                                castling:None,
                                                en_passant: None, 
                                            });
                                            moves.push(Move{
                                                translation: Some(Translation { from: square, to: checker_square }),
                                                promotion: Some(KNIGHT),
                                                capture: Some(checker_piece),
                                                castling:None,
                                                en_passant: None, 
                                            });
                                        }
                                        else{
                                            moves.push(Move{
                                                translation: Some(Translation { from: square, to: checker_square }),
                                                promotion: None,
                                                capture: Some(checker_piece),
                                                castling:None,
                                                en_passant: None, 
                                            });
                                        }
                                    }
                                }
                                if checker_piece > KNIGHT { 
                                    //could the pawn move to the square between the king and the checker?
                                    let pawn_moves = get_pawn_moves(us, square, occupancy);
                                    let valid_pawn_moves = pawn_moves & between_bb;
                                    if valid_pawn_moves != 0{
                                        //if the pawn is pinned vertically or horizontally, it can't move to the square between the king and the checker
                                        if our_pins.pins_h & square_bb == 0 && our_pins.pins_v & square_bb == 0{
                                            //if the pawn will promote, we need to generate all promotion moves
                                            if (us == Side::WHITE && valid_pawn_moves & RANK_8BB == 0) || (us == Side::BLACK && valid_pawn_moves & RANK_1BB == 0){
                                                moves.push(Move{
                                                    translation: Some(Translation { from: square, to: valid_pawn_moves.to_square() }),
                                                    promotion: Some(QUEEN),
                                                    capture: None,
                                                    castling:None,
                                                    en_passant: None, 
                                                });
                                                moves.push(Move{
                                                    translation: Some(Translation { from: square, to: valid_pawn_moves.to_square() }),
                                                    promotion: Some(ROOK),
                                                    capture: None,
                                                    castling:None,
                                                    en_passant: None, 
                                                });
                                                moves.push(Move{
                                                    translation: Some(Translation { from: square, to: valid_pawn_moves.to_square() }),
                                                    promotion: Some(BISHOP),
                                                    capture: None,
                                                    castling:None,
                                                    en_passant: None, 
                                                });
                                                moves.push(Move{
                                                    translation: Some(Translation { from: square, to: valid_pawn_moves.to_square() }),
                                                    promotion: Some(KNIGHT),
                                                    capture: None,
                                                    castling:None,
                                                    en_passant: None, 
                                                });
                                            }
                                            else{
                                                moves.push(Move{
                                                    translation: Some(Translation { from: square, to: valid_pawn_moves.to_square() }),
                                                    promotion: None,
                                                    capture: None,
                                                    castling:None,
                                                    en_passant: None, 
                                                });
                                            }
                                        }
                                    }
                                }                                
                                //check if the pawn can capture the checker en passant
                                if checker_piece == PAWN && self.en_passant_square.is_some() && self.en_passant_square.unwrap() == checker_square{
                                    let pawn_attacks = get_pawn_attacks(us, square);

                                    //get the en passant capture square
                                    let en_passant_capture_square = if us == Side::WHITE { checker_square + 8 } else { checker_square - 8 };
                                    let en_passant_capture_bb = Bitboard::from(en_passant_capture_square);

                                    if pawn_attacks & en_passant_capture_bb != 0{
                                        moves.push(Move{
                                            translation: Some(Translation { from: square, to: checker_square}),
                                            promotion: None,
                                            capture: Some(checker_piece),
                                            castling:None,
                                            en_passant: Some(checker_square), 
                                        });
                                    }
                                }
                            }
                            else if piece == KNIGHT{
                                //can the knight capture the checker?
                                let knight_attacks = get_knight_attacks(square);
                                if knight_attacks & checker_square_bb != 0{
                                    //if the knight is pinned vertically or horizontally, it can't capture the checker
                                    if our_pins.pins_h & square_bb == 0 && our_pins.pins_v & square_bb == 0{
                                        moves.push(Move{
                                            translation: Some(Translation { from: square, to: checker_square }),
                                            promotion: None,
                                            capture: Some(checker_piece),
                                            castling:None,
                                            en_passant: None, 
                                        });
                                    }
                                }
                                if checker_piece > KNIGHT {
                                    //could the knight move to the square between the king and the checker?
                                    let valid_knight_moves = knight_attacks & between_bb;
                                    if valid_knight_moves != 0{
                                        //if the knight is pinned vertically or horizontally, it can't move to the square between the king and the checker
                                        if our_pins.pins_h & square_bb == 0 && our_pins.pins_v & square_bb == 0{
                                            moves.push(Move{
                                                translation: Some(Translation { from: square, to: valid_knight_moves.to_square() }),
                                                promotion: None,
                                                capture: None,
                                                castling:None,
                                                en_passant: None, 
                                            });
                                        }
                                    }
                                }
                            }
                            else if piece == BISHOP{
                                //can the bishop capture the checker?
                                let bishop_attacks = get_bishop_attacks(square, occupancy);
                                if bishop_attacks & checker_square_bb != 0{
                                    //if the bishop is pinned vertically or horizontally, it can't capture the checker
                                    if our_pins.pins_h & square_bb == 0 && our_pins.pins_v & square_bb == 0{
                                        moves.push(Move{
                                            translation: Some(Translation { from: square, to: checker_square }),
                                            promotion: None,
                                            capture: Some(checker_piece),
                                            castling:None,
                                            en_passant: None, 
                                        });
                                    }
                                }
                                if checker_piece > KNIGHT {
                                    //could the bishop move to the square between the king and the checker?
                                    let valid_bishop_moves = bishop_attacks & between_bb;
                                    if valid_bishop_moves != 0{
                                        //if the bishop is pinned vertically or horizontally, it can't move to the square between the king and the checker
                                        if our_pins.pins_h & square_bb == 0 && our_pins.pins_v & square_bb == 0{
                                            moves.push(Move{
                                                translation: Some(Translation { from: square, to: valid_bishop_moves.to_square() }),
                                                promotion: None,
                                                capture: None,
                                                castling:None,
                                                en_passant: None, 
                                            });
                                        }
                                    }
                                }
                            }
                            else if piece == ROOK{
                                //can the rook capture the checker?
                                let rook_attacks = get_rook_attacks(square, occupancy);
                                if rook_attacks & checker_square_bb != 0{
                                    //if the rook is pinned diagonally, it can't capture the checker
                                    if our_pins.pins_da & square_bb == 0 && our_pins.pins_dd & square_bb == 0{
                                        //is checker on the same rank as the rook
                                        if get_rank_mask(square) & checker_square_bb != 0{
                                            //if the rook is pinned vertically, it can't capture the checker
                                            if our_pins.pins_h & square_bb == 0{
                                                moves.push(Move{
                                                    translation: Some(Translation { from: square, to: checker_square }),
                                                    promotion: None,
                                                    capture: Some(checker_piece),
                                                    castling:None,
                                                    en_passant: None, 
                                                });
                                            }
                                        }
                                        else{
                                            //if the rook is pinned horizontally, it can't capture the checker
                                            if our_pins.pins_h & square_bb == 0{
                                                moves.push(Move{
                                                    translation: Some(Translation { from: square, to: checker_square }),
                                                    promotion: None,
                                                    capture: Some(checker_piece),
                                                    castling:None,
                                                    en_passant: None, 
                                                });
                                            }
                                        }
                                    }
                                }
                                if checker_piece > KNIGHT {
                                    //could the rook move to the square between the king and the checker?
                                    let valid_rook_moves = rook_attacks & between_bb;
                                    if valid_rook_moves != 0{
                                        //if the rook is pinned diagonally, it can't move to the square between the king and the checker
                                        if our_pins.pins_da & square_bb == 0 && our_pins.pins_dd & square_bb == 0{
                                            for valid_rook_move in valid_rook_moves.get_squares(){
                                                let valid_rook_move_bb = Bitboard::from(valid_rook_move);
                                                //is the valid rook move on the same rank as the rook
                                                if get_rank_mask(square) & valid_rook_move_bb != 0{
                                                    //if the rook is pinned vertically, it can't move to the square between the king and the checker
                                                    if our_pins.pins_h & square_bb == 0{
                                                        moves.push(Move{
                                                            translation: Some(Translation { from: square, to: valid_rook_move }),
                                                            promotion: None,
                                                            capture: None,
                                                            castling:None,
                                                            en_passant: None, 
                                                        });
                                                    }
                                                }
                                                else{
                                                    //if the rook is pinned horizontally, it can't move to the square between the king and the checker
                                                    if our_pins.pins_h & square_bb == 0{
                                                        moves.push(Move{
                                                            translation: Some(Translation { from: square, to: valid_rook_move }),
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
                                }
                            }
                            else if piece == QUEEN{
                                //can the queen capture the checker?
                                let queen_attacks = get_queen_attacks(square, occupancy);
                                if queen_attacks & checker_square_bb != 0{
                                    //is the checker on the same rank as the queen
                                    if get_rank_mask(square) & checker_square_bb != 0{
                                        //if the queen is pinned vertically or diagonally, it can't capture the checker
                                        if our_pins.pins_h & square_bb == 0 && our_pins.pins_da & square_bb == 0 && our_pins.pins_dd & square_bb == 0{
                                            moves.push(Move{
                                                translation: Some(Translation { from: square, to: checker_square }),
                                                promotion: None,
                                                capture: Some(checker_piece),
                                                castling:None,
                                                en_passant: None, 
                                            });
                                        }
                                    }
                                    //is the checker on the same file as the queen
                                    else if get_file_mask(square) & checker_square_bb != 0{
                                        //if the queen is pinned horizontally or diagonally, it can't capture the checker
                                        if our_pins.pins_v & square_bb == 0 && our_pins.pins_da & square_bb == 0 && our_pins.pins_dd & square_bb == 0{
                                            moves.push(Move{
                                                translation: Some(Translation { from: square, to: checker_square }),
                                                promotion: None,
                                                capture: Some(checker_piece),
                                                castling:None,
                                                en_passant: None, 
                                            });
                                        }
                                    }
                                    //is the checker on the same descending diagonal as the queen
                                    else if DIRECTIONAL_MAP_DD[square as usize] & checker_square_bb != 0{
                                        //if the queen is pinned vertically or horizontally, it can't capture the checker
                                        if our_pins.pins_h & square_bb == 0 && our_pins.pins_v & square_bb == 0{
                                            moves.push(Move{
                                                translation: Some(Translation { from: square, to: checker_square }),
                                                promotion: None,
                                                capture: Some(checker_piece),
                                                castling:None,
                                                en_passant: None, 
                                            });
                                        }
                                    }
                                    //is the checker on the same ascending diagonal as the queen
                                    else if DIRECTIONAL_MAP_DA[square as usize] & checker_square_bb != 0{
                                        //if the queen is pinned vertically or horizontally, it can't capture the checker
                                        if our_pins.pins_h & square_bb == 0 && our_pins.pins_v & square_bb == 0{
                                            moves.push(Move{
                                                translation: Some(Translation { from: square, to: checker_square }),
                                                promotion: None,
                                                capture: Some(checker_piece),
                                                castling:None,
                                                en_passant: None, 
                                            });
                                        }
                                    }
                                }
                                if checker_piece > KNIGHT {
                                    //could the queen move to the square between the king and the checker?
                                    let valid_queen_moves = queen_attacks & between_bb;
                                    for valid_queen_move in valid_queen_moves.get_squares(){
                                        let valid_queen_move_bb = Bitboard::from(valid_queen_move);
                                        //is the valid queen move on the same rank as the queen
                                        if get_rank_mask(square) & valid_queen_move_bb != 0{
                                            //if the queen is pinned vertically or diagonally, it can't move to the square between the king and the checker
                                            if our_pins.pins_h & square_bb == 0 && our_pins.pins_da & square_bb == 0 && our_pins.pins_dd & square_bb == 0{
                                                moves.push(Move{
                                                    translation: Some(Translation { from: square, to: valid_queen_move }),
                                                    promotion: None,
                                                    capture: None,
                                                    castling:None,
                                                    en_passant: None, 
                                                });
                                            }
                                        }
                                        //is the valid queen move on the same file as the queen
                                        else if get_file_mask(square) & valid_queen_move_bb != 0{
                                            //if the queen is pinned horizontally or diagonally, it can't move to the square between the king and the checker
                                            if our_pins.pins_v & square_bb == 0 && our_pins.pins_da & square_bb == 0 && our_pins.pins_dd & square_bb == 0{
                                                moves.push(Move{
                                                    translation: Some(Translation { from: square, to: valid_queen_move }),
                                                    promotion: None,
                                                    capture: None,
                                                    castling:None,
                                                    en_passant: None, 
                                                });
                                            }
                                        }
                                        else{
                                            //if the queen is pinned vertically or horizontally, it can't move to the square between the king and the checker
                                            if our_pins.pins_h & square_bb == 0 && our_pins.pins_v & square_bb == 0{
                                                moves.push(Move{
                                                    translation: Some(Translation { from: square, to: valid_queen_move }),
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
                            else if piece == KING{
                                //can the king capture the checker?
                                let mut valid_attacks = get_king_attacks(square) & !our_occupancy;

                                if(debug == Some(true)){
                                    println!("valid attacks1: {}", valid_attacks);
                                }

                                valid_attacks &= !their_attacks_without_our_king.all();

                                if(debug == Some(true)){
                                    println!("valid attacks2: {}", valid_attacks);
                                }

                                for attack in valid_attacks.get_squares(){
                                    let attack_bb = Bitboard::from(attack);
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
                                        let piece = self.pieces[them.0].get_piece_type_at_square(square);
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
                }
                
                //no moves available after check
                if moves.is_empty(){
                    score = if us == Side::WHITE { Some(SCORE_BLACK_WINS) } else { Some(SCORE_WHITE_WINS) };
                    return PositionEvaluation{
                        game_state: GameState::CHECKMATE,
                        moves,
                        score
                    }
                }
            }
        }

        return PositionEvaluation{
            game_state,
            moves,
            score
        };
    }

    pub fn make_move(&self, m: Move) -> Position{
        let mut new_position = self.clone();

        let us = self.side_to_move;

        //if the move includes a translation
        if m.translation.is_some(){
            let translation = m.translation.unwrap();
            let from_piece_wrapped = self.pieces[us.0].get_piece_type_at_square(translation.from);
            if from_piece_wrapped.is_none(){
                panic!("No piece at the from square!");
            }
            let from_piece = from_piece_wrapped.unwrap();

            if from_piece == PAWN{
                //check if en passant is involved
                if m.en_passant.is_some(){
                    //self en passant or opponent en passant?
                    let en_passant = m.en_passant.unwrap();
                    //en_passant is the square the pawn is moving to
                    if en_passant == translation.to{
                        new_position.pieces[us.0][PAWN] = new_position.pieces[us.0][PAWN].set_bit(translation.to);
                        new_position.en_passant_square = Some(translation.to);
                        //remove original pawn
                        new_position.pieces[us.0][PAWN] = new_position.pieces[us.0][PAWN].unset_bit(translation.from);
                    }
                    else{
                        //opponent en passant
                        new_position.pieces[us.0][PAWN] = new_position.pieces[us.0][PAWN].set_bit(translation.to);
                        new_position.en_passant_square = None;
                        //remove the captured pawn
                        new_position.pieces[(!us).0][PAWN] = new_position.pieces[(!us).0][PAWN].unset_bit(en_passant);
                        //remove original pawn
                        new_position.pieces[us.0][PAWN] = new_position.pieces[us.0][PAWN].unset_bit(translation.from);

                    }
                }
                else{
                    //no en passant, just a normal pawn move

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
            new_position.halfmove_clock += 1;
        }
        else{
            panic!("Unidentified move!");
        }
        if us == Side::BLACK{
            new_position.fullmove_number += 1;
        }

        new_position.side_to_move = !us;
        new_position.halfmove_clock += 1;

        return new_position;
    }
}

