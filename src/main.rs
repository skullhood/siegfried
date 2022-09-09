
use siegfried::{display::print_bitboard, bitboard::FILE_FBB};
use siegfried::bitboard::{BitboardMethods};
 
fn main() {

    /*
        let magics = get_rook_magics();
        let magic_for_bishop = &magics[bishop_position.0 as usize];
        let idx = magic_for_bishop.get_index(occupancy);
        print_bitboard(magic_for_bishop.attacks[idx]);  
    */
    let mut occupancy = FILE_FBB;

    println!("Occupancy:");

    print_bitboard(occupancy);

    //pop lsb
    let lsb = occupancy.pop_lsb();

    println!("LSB:");
    
    print_bitboard(lsb);

    //get all squares
    let squares = 0.get_squares();

    println!("Squares:");

    for square in squares{
        print!("{:#?} ", square);
    }

    println!("Occupancy after pop:");

    print_bitboard(occupancy);

    /* 
    let pawn_position: Bitboard = 0.set_bit(Square::H4);

    println!("Pawn");

    println!("Pawn Move");
    */
    
}


