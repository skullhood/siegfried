use crate::masks::*;
use crate::bitboard::*;
use crate::types::Magic;
use crate::types::MagicIndex;
use crate::types::Side;
use crate::types::SideConstants;
use crate::types::Square;
use crate::types::SquareConstants;
use crate::types::SquareMethods;
use crate::types::Squares;
use bitintr::Pext;

use crate::lazy_static::lazy_static;

lazy_static! {
    static ref WHITE_PAWN_ATTACK_MAP: [Bitboard; 64] = {
        let m = get_pawn_attack_map(Side::WHITE);
        m
    };
    static ref BLACK_PAWN_ATTACK_MAP: [Bitboard; 64] = {
        let m = get_pawn_attack_map(Side::BLACK);
        m
    };
    static ref KNIGHT_ATTACK_MAP: [Bitboard; 64] = {
        let m = get_knight_attack_map();
        m
    };
    static ref ROOK_MAGICS: Box<[Magic]> = {
        let m = get_rook_magics().into_boxed_slice();
        m
    };
    static ref BISHOP_MAGICS: Box<[Magic]> = {
        let m = get_bishop_magics().into_boxed_slice();
        m
    };
    static ref KING_ATTACK_MAP: [Bitboard; 64] = {
        let m = get_king_attack_map();
        m
    };
    pub static ref DIRECTIONAL_MAP_RANK: [Bitboard; 64] = {
        let m = get_rank_map();
        m
    };
    pub static ref DIRECTIONAL_MAP_FILE: [Bitboard; 64] = {
        let m = get_file_map();
        m
    };
    pub static ref DIRECTIONAL_MAP_DA: [Bitboard; 64] = {
        let m = get_diagonal_ascending_map();
        m
    };
    pub static ref DIRECTIONAL_MAP_DD: [Bitboard; 64] = {
        let m = get_diagonal_descending_map();
        m
    };

}

pub fn load_maps() {
    let square = Square::D5;
    let occupancy = Bitboard::EMPTY;
    //lazy load all the maps
    let _rook_magic_init = get_rook_attacks(square, occupancy);
    let _bishop_magic_init = get_bishop_attacks(square, occupancy);   
    let _knight_attack_init = get_knight_attacks(square);
    let _king_attack_init = get_king_attacks(square);

    let _file_map_init = DIRECTIONAL_MAP_FILE[square as usize];
    let _rank_map_init = DIRECTIONAL_MAP_RANK[square as usize];
    let _dd_map_init = DIRECTIONAL_MAP_DD[square as usize];
    let _da_map_init = DIRECTIONAL_MAP_DA[square as usize];
}

pub fn get_ray_between_squares(from: Square, to: Square) -> Bitboard{
    let mut squares_between: Bitboard = 0;

    if from == to {
        return Bitboard::EMPTY;
    }

    let from_file = from as usize % 8;
    let from_rank = from as usize / 8;

    let to_file = to as usize % 8;
    let to_rank = to as usize / 8;

    if from_file == to_file{
        let lower_rank = from_rank.min(to_rank);
        let upper_rank = from_rank.max(to_rank);
        for rank in lower_rank + 1..upper_rank{
            squares_between |= 1 << (rank * 8 + from_file);
        }
    }
    else if from_rank == to_rank{
        let lower_file = from_file.min(to_file);
        let upper_file = from_file.max(to_file);
        for file in lower_file + 1..upper_file{
            squares_between |= 1 << (from_rank * 8 + file);
        }
    }
    else{
        let file_diff = to_file as i8 - from_file as i8;
        let rank_diff = to_rank as i8 - from_rank as i8;
        let mut file = from_file as i8;
        let mut rank = from_rank as i8;
        let fsig =file_diff.signum();
        let rsig = rank_diff.signum();

        while file != to_file as i8 - fsig && rank != to_rank as i8 - rsig{
            file += fsig;
            rank += rsig;
            squares_between |= 1 << (rank as usize * 8 + file as usize);
        }
    }

    return squares_between;
}


//DIRECTION MAPS
fn get_diagonal_ascending_map() -> [Bitboard; 64] {
    let mut map: [Bitboard; 64] = [0; 64];
    for square in Squares {
        map[square as usize] = get_diagonal_ascending_mask(square);
    }
    return map;
}

fn get_diagonal_descending_map() -> [Bitboard; 64] {
    let mut map: [Bitboard; 64] = [0; 64];
    for square in Squares {
        map[square as usize] = get_diagonal_descending_mask(square);
    }
    return map;
}

fn get_rank_map() -> [Bitboard; 64] {
    let mut map: [Bitboard; 64] = [0; 64];
    for square in Squares {
        map[square as usize] = get_rank_mask(square);
    }
    return map;
}

fn get_file_map() -> [Bitboard; 64] {
    let mut map: [Bitboard; 64] = [0; 64];
    for square in Squares {
        map[square as usize] = get_file_mask(square);
    }
    return map;
}


//PAWN
pub fn get_pawn_moves(side: Side, square: Square, occupancy: Bitboard) -> Bitboard{
    let mut moves: Bitboard = 0;

    let square_bb = square.to_bitboard();

    if side == Side::WHITE{
        //get square in front of pawn
        let square_in_front = square_bb << 8;
        if square_in_front & occupancy == 0{
            moves |= square_in_front;
            //if pawn is on starting rank
            if square_bb & RANK_2BB != 0{
                //get square two squares in front
                let leap_square = square_in_front << 8;
                //if square two squares in front is empty
                if leap_square & occupancy == 0{
                    //add square two squares in front to moves
                    moves |= leap_square;
                }
            }
        }
    }
    else{
        //get square in front of pawn
        let square_in_front = square_bb >> 8;
        if square_in_front & occupancy == 0{
            moves |= square_in_front;
            //if pawn is on starting rank
            if square_bb & RANK_7BB != 0{
                //get square two squares in front
                let leap_square = square_in_front >> 8;
                //if square two squares in front is empty
                if leap_square & occupancy == 0{
                    //add square two squares in front to moves
                    moves |= leap_square;
                }
            }
        }

    }


    return moves;
}


fn get_pawn_attack_map(side: Side) -> [Bitboard; 64] {
    let mut attack_map: [Bitboard; 64] = [0; 64];
    for square in Squares {
        let attacks = mask_pawn_attacks(side, square);
        attack_map[square as usize] = attacks;
    }
    return attack_map;
}

pub fn get_pawn_attacks(side: Side, square: Square) -> Bitboard{
    return match side {
        Side::WHITE => WHITE_PAWN_ATTACK_MAP[square as usize],
        Side::BLACK => BLACK_PAWN_ATTACK_MAP[square as usize],
        Side(_) => panic!("Invalid side for method get_pawn_attacks! Side: {}", side),
    };
}

//KNIGHT
fn get_knight_attack_map() -> [Bitboard; 64]{
    let mut attack_map: [Bitboard; 64] = [0; 64];

    for square in Squares{
        attack_map[square as usize] = mask_knight_attacks(square); 
    }

    return attack_map;
}

pub fn get_knight_attacks(square: Square) -> Bitboard{
    return KNIGHT_ATTACK_MAP[square as usize];
}

//BISHOP 
pub fn get_bishop_attack_rays() -> [Bitboard; 64]{
    let mut ray_map: [Bitboard; 64] = [0; 64];

    for square in Squares{
        ray_map[square as usize] = mask_bishop_attacks(square, 0); 
    }

    return ray_map;
}

fn get_bishop_blockers() -> [Bitboard; 64]{
    let mut block_map: [Bitboard; 64] = [0; 64];
    let attack_rays = get_bishop_attack_rays();

    for square in Squares{
        let ray_map = attack_rays[square as usize];
        block_map[square as usize] = ray_map&NOT_OUTER;
    }

    return block_map;
}

fn get_bishop_magics() -> Vec<Magic> {
    let mut bishop_magic: Vec<Magic> = Vec::with_capacity(64);

    let bishop_blockmap = get_bishop_blockers();

    let mut occupancy: [Bitboard; 4096] = [0; 4096];
    let mut reference: [Bitboard; 4096] = [0; 4096];
    let mut b: Bitboard;

    let mut size: usize;

    for square in Squares{
        let bishop_mask = bishop_blockmap[square as usize];

        let mut magic = Magic{
            mask: bishop_mask,
            magic: 0,
            attacks: [0; 4096],
            shift: bishop_mask.count_ones() as usize,
        };

        b = 0;
        size = 0;

        occupancy[size] = b;
        reference[size] = mask_bishop_attacks(square, b);

        magic.attacks[Pext::pext(b, magic.mask) as usize] = reference[size];

        size+=1;
        b = ((b | !magic.mask).overflowing_add(1).0) & magic.mask;

        while b > 0 {
            occupancy[size] = b;
            reference[size] = mask_bishop_attacks(square, b);

            magic.attacks[Pext::pext(b, magic.mask) as usize] = reference[size];

            size+=1;
            b = ((b | !magic.mask).wrapping_add(1)) & magic.mask;
        }
        bishop_magic.insert(square as usize, magic);
    }

    return bishop_magic;

}

pub fn get_bishop_attacks(square: Square, occupancy: Bitboard) -> Bitboard{
    let magic = BISHOP_MAGICS[square as usize];
    let index = magic.get_index(occupancy);
    return magic.attacks[index];
}
//ROOK 
pub fn get_rook_attack_rays() -> [Bitboard; 64]{
    let mut ray_map: [Bitboard; 64] = [0; 64];

    for square in Squares{
        ray_map[square as usize] = mask_rook_attacks(square, 0); 
    }

    return ray_map;
}

fn get_rook_blockers() -> [Bitboard; 64]{
    let mut block_map: [Bitboard; 64] = [0; 64];

    let attack_rays = get_rook_attack_rays();

    for square in Squares{
        let mut attack_map = attack_rays[square as usize];
        if (attack_map&FILE_ABB).count_ones() == 1 { attack_map &= NOT_FILE_ABB }
        if (attack_map&FILE_HBB).count_ones() == 1 { attack_map &= NOT_FILE_HBB }
        if (attack_map&RANK_1BB).count_ones() == 1 { attack_map &= NOT_RANK_1BB }
        if (attack_map&RANK_8BB).count_ones() == 1 { attack_map &= NOT_RANK_8BB }
        block_map[square as usize] = attack_map;
    }

    return block_map;
}

fn get_rook_magics() -> Vec<Magic>{
    let mut rook_magics: Vec<Magic> = Vec::with_capacity(64);

    let bishop_blockmap = get_rook_blockers();

    let mut occupancy: [Bitboard; 4096] = [0; 4096];
    let mut reference: [Bitboard; 4096] = [0; 4096];
    let mut b: Bitboard;

    let mut size: usize;

    for square in Squares{
        let bishop_mask = bishop_blockmap[square as usize];

        let mut magic = Magic{
            mask: bishop_mask,
            magic: 0,
            attacks: [0; 4096],
            shift: bishop_mask.count_ones() as usize,
        };

        b = 0;
        size = 0;

        occupancy[size] = b;
        reference[size] = mask_rook_attacks(square, b);

        magic.attacks[Pext::pext(b, magic.mask) as usize] = reference[size];

        size+=1;
        b = ((b | !magic.mask).overflowing_add(1).0) & magic.mask;

        while b > 0 {
            occupancy[size] = b;
            reference[size] = mask_rook_attacks(square, b);

            magic.attacks[Pext::pext(b, magic.mask) as usize] = reference[size];

            size+=1;
            b = ((b | !magic.mask).wrapping_add(1)) & magic.mask;
        }
        rook_magics.insert(square as usize, magic);
    }

    return rook_magics;
}

pub fn get_rook_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    let magic = ROOK_MAGICS[square as usize];
    let index = magic.get_index(occupancy);
    return magic.attacks[index];
}

//QUEEN
pub fn get_queen_attack_rays() -> [Bitboard; 64]{
    let mut attack_map: [Bitboard; 64] = [0; 64];

    for square in Squares{
        attack_map[square as usize] = mask_rook_attacks(square, 0)|mask_bishop_attacks(square, 0);
    }

    return attack_map;
}

pub fn get_queen_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    return get_rook_attacks(square, occupancy)|get_bishop_attacks(square, occupancy);
}

//KING 
fn get_king_attack_map() -> [Bitboard; 64]{
    let mut attack_map: [Bitboard; 64] = [0; 64];
    
    for square in Squares{
        attack_map[square as usize] = mask_king_attacks(square); 
    }

    return attack_map;
}

pub fn get_king_attacks(square: Square) -> Bitboard {
    return KING_ATTACK_MAP[square as usize];
}

