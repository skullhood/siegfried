use crate::{bitboard::*, types::{Square, Side, SideConstants}};

//CORNER MASKS
const NE_CORNER: Bitboard = RANK_8BB|FILE_HBB;
const NW_CORNER: Bitboard = RANK_8BB|FILE_ABB;
const SW_CORNER: Bitboard = RANK_1BB|FILE_ABB;
const SE_CORNER: Bitboard = RANK_1BB|FILE_HBB;

pub fn get_file_mask(square: Square) -> Bitboard {
    let mut mask: Bitboard = 0;
    let square_bb = Bitboard::from_square(square);

    if square_bb & RANK_8BB == 0{
        for x in 1..8{
            let line = 8 * x;
            let ray = square_bb << line;
            mask |= ray;
            if ray & RANK_8BB != 0{
                break;
            }
        }
    }
    if square_bb & RANK_1BB == 0{
        for x in 1..8{
            let line = 8 * x;
            let ray = square_bb >> line;
            mask |= ray;
            if ray & RANK_1BB != 0{
                break;
            }
        }
    }
    
    return mask;
}

pub fn get_rank_mask(square: Square) -> Bitboard {
    let mut mask: Bitboard = 0;
    let square_bb = Bitboard::from_square(square);
    
    if square_bb & FILE_ABB == 0{
        for x in 1..8{
            let ray = square_bb >> x;
            mask |= ray;
            if ray & FILE_ABB != 0{
                break;
            }
        }
    }

    if square_bb & FILE_HBB == 0{
        for x in 1..8{
            let ray = square_bb << x;
            mask |= ray;
            if ray & FILE_HBB != 0{
                break;
            }
        }
    }

    return mask;
}

pub fn get_diagonal_descending_mask(square: Square) -> Bitboard {
    let mut mask: Bitboard = 0;
    let square_bb = Bitboard::from_square(square);
    
    if square_bb & NW_CORNER == 0{
        for x in 1..8{
            let diag = (6 * x) + x;
            let ray = square_bb << diag;
            mask |= ray;
            if ray & NW_CORNER != 0{
                break;
            }
        }
    }

    if square_bb & SE_CORNER == 0{
        for x in 1..8{
            let diag = (6 * x) + x;
            let ray = square_bb >> diag;
            mask |= ray;
            if ray & SE_CORNER != 0{
                break;
            }
        }
    }

    return mask;
}

pub fn get_diagonal_ascending_mask(square: Square) -> Bitboard {
    let mut mask: Bitboard = 0;    
    let square_bb = 1 << square;

    if square_bb & SW_CORNER == 0{
        for x in 1..8{
            let diag = (8 * x) + x;
            let ray = square_bb >> diag;
            mask |= ray;
            if ray & SW_CORNER != 0{
                break;
            }
        }
    }    

    if square_bb & NE_CORNER == 0{
        for x in 1..8{
            let diag = (8 * x) + x;
            let ray = square_bb << diag;
            mask |= ray;
            if ray & NE_CORNER != 0{
                break;
            }
        }
    }
    
    return mask;
}

//PAWN MASK
pub fn mask_pawn_attacks(side: Side, square: Square) -> Bitboard{

    let pawn = 1 << square;

    let mut attacks: Bitboard = 0;

    if side == Side::WHITE {
        if pawn & FILE_HBB == 0 {attacks |= pawn << 9};
        if pawn & FILE_ABB == 0 {attacks |= pawn << 7};
        return attacks;
    }
    if pawn & FILE_ABB == 0 {attacks |= pawn >> 9};
    if pawn & FILE_HBB == 0 {attacks |= pawn >> 7};
    return attacks;
}

//KNIGHT MASK
pub fn mask_knight_attacks(square: Square) -> Bitboard{
    let mut attacks: Bitboard = 0;

    let knight = 1 << square;

    if knight & (FILE_HBB) == 0 { 
        attacks |= knight << 17;
        attacks |= knight >> 15;

        if knight & (FILE_GBB) == 0 { 
            attacks |= knight << 10;
            attacks |= knight >> 6;
        }
    }

    if knight & (FILE_ABB) == 0 { 
        attacks |= knight >> 17;
        attacks |= knight << 15;

        if knight & (FILE_BBB) == 0 { 
            attacks |= knight >> 10;
            attacks |= knight << 6;
        }
    }

    return attacks;
}

pub fn mask_bishop_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    let mut attacks: Bitboard = 0;

    let bishop = 1 << square;
    
    //NE ray calculation
    if bishop & NE_CORNER == 0{
        for x in 1..8{
            let diag = (8 * x) + x;
            let ray = bishop << diag;
            attacks |= ray;
            if ray & NE_CORNER != 0 || ray & occupancy != 0{
                break;
            }
        }
    }

    //NW ray calculation
    if bishop & NW_CORNER == 0{
        for x in 1..8{
            let diag = (6 * x) + x;
            let ray = bishop << diag;
            attacks |= ray;
            if ray & NW_CORNER != 0 || ray & occupancy != 0{
                break;
            }
        }
    }

    //SW ray calculation
    if bishop & SW_CORNER == 0{
        for x in 1..8{
            let diag = (8 * x) + x;
            let ray = bishop >> diag;
            attacks |= ray;
            if ray & SW_CORNER != 0 || ray & occupancy != 0{
                break;
            }
        }
    }    

    //SE ray calculation
    if bishop & SE_CORNER == 0{
        for x in 1..8{
            let diag = (6 * x) + x;
            let ray = bishop >> diag;
            attacks |= ray;
            if ray & SE_CORNER != 0 || ray & occupancy != 0{
                break;
            }
        }
    }
    
    return attacks;
}

//ROOK MASK
pub fn mask_rook_attacks(square: Square, occupancy: Bitboard) -> Bitboard{
    let mut attacks: Bitboard = 0;

    let rook = 1 << square;

    //NORTH 
    if rook & RANK_8BB == 0{
        for x in 1..8{
            let line = 8 * x;
            let ray = rook << line;
            attacks |= ray;
            if ray & RANK_8BB != 0 || ray & occupancy != 0{
                break;
            }
        }
    }

    //EAST
    if rook & FILE_ABB == 0{
        for x in 1..8{
            let ray = rook >> x;
            attacks |= ray;
            if ray & FILE_ABB != 0 || ray & occupancy != 0{
                break;
            }
        }
    }

    //SOUTH 
    if rook & RANK_1BB == 0{
        for x in 1..8{
            let line = 8 * x;
            let ray = rook >> line;
            attacks |= ray;
            if ray & RANK_8BB != 0 || ray & occupancy != 0{
                break;
            }
        }
    }

    //WEST
    if rook & FILE_HBB == 0{
        for x in 1..8{
            let ray = rook << x;
            attacks |= ray;
            if ray & FILE_HBB != 0 || ray & occupancy != 0{
                break;
            }
        }
    }

    return attacks;
}

//KING MASK
pub fn mask_king_attacks(square: Square) -> Bitboard{
    let mut attacks: Bitboard = 0;

    let king = 1 << square;

    //Left Shift
    if king & FILE_HBB == 0{
        attacks |= king << 1;
        if king & RANK_8BB == 0{
            attacks |= king << 9;
        }
    }
    if king & RANK_8BB == 0{
        attacks |= king << 8;
        if king & FILE_ABB == 0{
            attacks |= king << 7;
        }
    }

    //Right Shift
    if king & FILE_ABB == 0{
        attacks |= king >> 1;
        if king & RANK_1BB == 0{
            attacks |= king >> 9;
        }
    }
    if king & RANK_1BB == 0{
        attacks |= king >> 8;
        if king & FILE_HBB == 0{
            attacks |= king >> 7;
        }
    }

    return attacks;
}

