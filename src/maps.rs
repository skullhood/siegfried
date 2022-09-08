use crate::masks::*;
use crate::bitboard::*;
use crate::types::Magic;
use crate::types::Squares;
use bitintr::Pext;

use crate::lazy_static::lazy_static;

//Initialize these as constants 
lazy_static! {
    pub static ref KNIGHT_ATTACKS: [Bitboard; 64] = {
        let m = get_knight_attack_map();
        m
    };
    pub static ref KING_ATTACKS: [Bitboard; 64] = {
        let m = get_king_attack_map();
        m
    };
}

//KNIGHT
pub fn get_knight_attack_map() -> [Bitboard; 64]{
    let mut attack_map: [Bitboard; 64] = [0; 64];

    for square in Squares{
        attack_map[square] = mask_knight_attacks(square); 
    }

    return attack_map;
}

//BISHOP 
pub fn get_bishop_attack_rays() -> [Bitboard; 64]{
    let mut ray_map: [Bitboard; 64] = [0; 64];

    for square in Squares{
        ray_map[square] = mask_bishop_attacks(square, 0); 
    }

    return ray_map;
}

pub fn get_bishop_blockers() -> [Bitboard; 64]{
    let mut block_map: [Bitboard; 64] = [0; 64];
    let attack_rays = get_bishop_attack_rays();

    for square in Squares{
        let ray_map = attack_rays[square];
        block_map[square] = ray_map&NOT_OUTER;
    }

    return block_map;
}

pub fn get_bishop_magics() -> Vec<Magic>{
    let mut bishop_magic: Vec<Magic> = Vec::with_capacity(64);

    let bishop_blockmap = get_bishop_blockers();

    let mut occupancy: [Bitboard; 4096] = [0; 4096];
    let mut reference: [Bitboard; 4096] = [0; 4096];
    let mut b: Bitboard;

    let mut size: usize;

    for square in Squares{
        let bishop_mask = bishop_blockmap[square];

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
        bishop_magic.insert(square.0 as usize, magic);
    }

    return bishop_magic;
}

//ROOK 
pub fn get_rook_attack_rays() -> [Bitboard; 64]{
    let mut ray_map: [Bitboard; 64] = [0; 64];

    for square in Squares{
        ray_map[square] = mask_rook_attacks(square, 0); 
    }

    return ray_map;
}

pub fn get_rook_blockers() -> [Bitboard; 64]{
    let mut block_map: [Bitboard; 64] = [0; 64];

    let attack_rays = get_rook_attack_rays();

    for square in Squares{
        let mut attack_map = attack_rays[square];
        if (attack_map&FILE_ABB).count_ones() == 1 { attack_map &= NOT_FILE_ABB }
        if (attack_map&FILE_HBB).count_ones() == 1 { attack_map &= NOT_FILE_HBB }
        if (attack_map&RANK_1BB).count_ones() == 1 { attack_map &= NOT_RANK_1BB }
        if (attack_map&RANK_8BB).count_ones() == 1 { attack_map &= NOT_RANK_8BB }
        block_map[square] = attack_map;
    }

    return block_map;
}

pub fn get_rook_magics() -> Vec<Magic>{
    let mut rook_magics: Vec<Magic> = Vec::with_capacity(64);

    let bishop_blockmap = get_rook_blockers();

    let mut occupancy: [Bitboard; 4096] = [0; 4096];
    let mut reference: [Bitboard; 4096] = [0; 4096];
    let mut b: Bitboard;

    let mut size: usize;

    for square in Squares{
        let bishop_mask = bishop_blockmap[square];

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
        rook_magics.insert(square.0 as usize, magic);
    }

    return rook_magics;
}

//QUEEN
pub fn get_queen_attack_rays() -> [Bitboard; 64]{
    let mut attack_map: [Bitboard; 64] = [0; 64];

    for square in Squares{
        attack_map[square] = mask_rook_attacks(square, 0)|mask_bishop_attacks(square, 0);
    }

    return attack_map;
}

//KING 
pub fn get_king_attack_map() -> [Bitboard; 64]{
    let mut attack_map: [Bitboard; 64] = [0; 64];

    for square in Squares{
        attack_map[square] = mask_king_attacks(square); 
    }
    return attack_map;
}

