use serde_json::*;

use crate::{position::Position, display::print_position, types::{GameState, GameStateConstants}};

#[test]
pub fn move_generation_test(){
    let file = std::fs::File::open("./src/../testfens.json").unwrap();
    let reader = std::io::BufReader::new(file);
    let json: Value = serde_json::from_reader(reader).unwrap();

    let keys = json.as_object().unwrap().keys();

    let mut key_count = 0;
    //iterate through all positions
    for key in keys{
        key_count += 1;
        let position = Position::from_fen(key);
        let mut position_eval = position.evaluate();

        let fen_moves = json[key].as_array().unwrap();
        //position moves as Vec<String>
        let mut position_moves: Vec<String> = Vec::new();
        for m in position_eval.moves{
            position_moves.push(m.get_tstring());
        }

        let fen_move_strings = fen_moves.iter().map(|m| m.as_str().unwrap().to_string()).collect::<Vec<String>>();

        let fen_copy = fen_move_strings.clone();

        //check if all moves are in the position moves
        for fen_move in fen_move_strings{
            if position_eval.game_state != GameState::DRAW && !position_moves.contains(&fen_move){

                position_eval = position.evaluate();

                println!("Position Moves: ");

                print_position(&position);
                println!("fen: {}", key);
                println!("gamestate: {}", position_eval.game_state);
                println!("{} to move", position.side_to_move);
                println!("Keycount: {}", key_count);
                println!("Castling: {:?}", position.castling_rights);
                panic!("{} not in position moves", fen_move);
            }
        }

        //check if all position moves are in the fen moves
        for position_move in position_moves{
            if position_eval.game_state != GameState::DRAW && !fen_copy.contains(&position_move){
                position_eval = position.evaluate();

                println!("Position Moves: ");
                for pm in position_eval.moves{
                    print!("{} ", pm.get_tstring());    
                }
                print_position(&position);
                println!("fen: {}", key);
                println!("gamestate: {}", position_eval.game_state);
                println!("{} to move", position.side_to_move);
                println!("Keycount: {}", key_count);
                println!("Castling: {:?}", position.castling_rights);
                panic!("{} not in fen moves", position_move);
            }
        }
    }
}