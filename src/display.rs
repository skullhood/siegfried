use std::ops::{Shr};

use crate::{bitboard::*, position::{Position, SidePiecesMethods}, types::*};

const UNICODE_WHITE_PAWN: char = '♙';
const UNICODE_WHITE_KNIGHT: char = '♘';
const UNICODE_WHITE_BISHOP: char = '♗';
const UNICODE_WHITE_ROOK: char = '♖';
const UNICODE_WHITE_QUEEN: char = '♕';
const UNICODE_WHITE_KING: char = '♔';

const UNICODE_BLACK_PAWN: char = '♟';
const UNICODE_BLACK_KNIGHT: char = '♞';
const UNICODE_BLACK_BISHOP: char = '♝';
const UNICODE_BLACK_ROOK: char = '♜';
const UNICODE_BLACK_QUEEN: char = '♛';
const UNICODE_BLACK_KING: char = '♚';

//BIT PRINTING UTILITY CONSTANTS
pub const BIT_8 : u8 = 0b10000000;
pub const BIT_7 : u8 = 0b01000000;
pub const BIT_6 : u8 = 0b00100000;
pub const BIT_5 : u8 = 0b00010000;
pub const BIT_4 : u8 = 0b00001000;
pub const BIT_3 : u8 = 0b00000100;
pub const BIT_2 : u8 = 0b00000010;
pub const BIT_1 : u8 = 0b00000001;

//PRINTING FUNCTIONS
fn get_rank_string(rank: u8) -> String{
    let mut rank_string: String = String::from("");

    if rank&BIT_1==0{rank_string+=" . "}else{rank_string+=" 1 "}
    if rank&BIT_2==0{rank_string+=" . "}else{rank_string+=" 1 "}
    if rank&BIT_3==0{rank_string+=" . "}else{rank_string+=" 1 "}
    if rank&BIT_4==0{rank_string+=" . "}else{rank_string+=" 1 "}
    if rank&BIT_5==0{rank_string+=" . "}else{rank_string+=" 1 "}
    if rank&BIT_6==0{rank_string+=" . "}else{rank_string+=" 1 "}
    if rank&BIT_7==0{rank_string+=" . "}else{rank_string+=" 1 "}
    if rank&BIT_8==0{rank_string+=" . "}else{rank_string+=" 1 "}     
    return rank_string;
}

//Lazy but good enough way to print a bitboard
pub fn print_bitboard(board: Bitboard){
    
    let rank8 = (board&RANK_8BB).shr(8*7) as u8;
    let rank7 = (board&RANK_7BB).shr(8*6) as u8;
    let rank6 = (board&RANK_6BB).shr(8*5) as u8;
    let rank5 = (board&RANK_5BB).shr(8*4) as u8;
    let rank4 = (board&RANK_4BB).shr(8*3) as u8;
    let rank3 = (board&RANK_3BB).shr(8*2) as u8;
    let rank2 = (board&RANK_2BB).shr(8*1) as u8;
    let rank1 = (board&RANK_1BB) as u8;

    println!("8   {}", get_rank_string(rank8));
    println!("7   {}", get_rank_string(rank7));
    println!("6   {}", get_rank_string(rank6));
    println!("5   {}", get_rank_string(rank5));
    println!("4   {}", get_rank_string(rank4));
    println!("3   {}", get_rank_string(rank3));
    println!("2   {}", get_rank_string(rank2));
    println!("1   {}", get_rank_string(rank1));
    println!("\n     A  B  C  D  E  F  G  H");
}

pub fn print_bitboard_alt(board: Bitboard){
    let mut board_string: String = String::from("");

    let ranks = [RANK_1BB, RANK_2BB, RANK_3BB, RANK_4BB, RANK_5BB, RANK_6BB, RANK_7BB, RANK_8BB];
    let files = [FILE_ABB, FILE_BBB, FILE_CBB, FILE_DBB, FILE_EBB, FILE_FBB, FILE_GBB, FILE_HBB];

    for rank in ranks.iter().rev(){
        for file_iterator in 0..files.len(){
            let file = files[file_iterator];
            board_string += format!(" {} ", file_iterator).as_str();
            if board&(*rank)&(file) != 0{
                board_string += "1  ";
            }else{
                board_string += ".  ";
            }
        }
        board_string += "\n";
    }

    println!("{}", board_string);
}


pub fn print_position(position: &Position){
    println!("");
    for rank in (1..9).rev(){
        println!();
        print!("{}   ", rank);
        for file in 1..9{
            //match rank and file to square
            let square: u8 = (rank-1)*8+file-1;
            let square_bb = square.to_bitboard();
            let side = if square_bb & position.pieces[Side::WHITE.0].occupancy() != 0 {Side::WHITE} else {Side::BLACK};
            let piece_type = position.pieces[side.0].get_piece_type_at_square(square);
            if piece_type.is_none(){
                print!(".  ");
            }else{
                let piece_type = piece_type.unwrap();

                if piece_type == PAWN{
                    if side == Side::WHITE{
                        print!("{}  ", UNICODE_WHITE_PAWN);
                    }else{
                        print!("{}  ", UNICODE_BLACK_PAWN);
                    }
                }
                else if piece_type == KNIGHT{
                    if side == Side::WHITE{
                        print!("{}  ", UNICODE_WHITE_KNIGHT);
                    }else{
                        print!("{}  ", UNICODE_BLACK_KNIGHT);
                    }
                }
                else if piece_type == BISHOP{
                    if side == Side::WHITE{
                        print!("{}  ", UNICODE_WHITE_BISHOP);
                    }else{
                        print!("{}  ", UNICODE_BLACK_BISHOP);
                    }
                }
                else if piece_type == ROOK{
                    if side == Side::WHITE{
                        print!("{}  ", UNICODE_WHITE_ROOK);
                    }else{
                        print!("{}  ", UNICODE_BLACK_ROOK);
                    }
                }
                else if piece_type == QUEEN{
                    if side == Side::WHITE{
                        print!("{}  ", UNICODE_WHITE_QUEEN);
                    }else{
                        print!("{}  ", UNICODE_BLACK_QUEEN);
                    }
                }
                else if piece_type == KING{
                    if side == Side::WHITE{
                        print!("{}  ", UNICODE_WHITE_KING);
                    }else{
                        print!("{}  ", UNICODE_BLACK_KING);
                    }
                }
            }
        }
    }
    println!("\n\n    A  B  C  D  E  F  G  H");
    println!("")
}