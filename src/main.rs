
use siegfried::{maps::*, display::print_bitboard, bitboard::FILE_FBB};
use siegfried::bitboard::SquareBitboardMethods;
use siegfried::types::*;


fn main() {

    let magics = get_rook_magics();

    let mut occupancy = 0.set_bit(Square::A7).set_bit(Square::F8).set_bit(Square::E3);
    occupancy |= FILE_FBB;

    println!("Occupancy");
    print_bitboard(occupancy);

    let bishop_position = Square::D4;
    let bishop_board = 0.set_bit(bishop_position);

    println!("Bishop");
    print_bitboard(bishop_board);

    println!("Attack Map");
    let magic_for_bishop = &magics[bishop_position.0 as usize];
    let idx = magic_for_bishop.get_index(occupancy);
    print_bitboard(magic_for_bishop.attacks[idx]);
    
}


