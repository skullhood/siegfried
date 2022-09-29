use rand::{Rng, seq::SliceRandom};
use siegfried::{
    bitboard::*, 
    types::{
        Square, 
        SquareConstants, Side, SideConstants, GameState, GameStateConstants, Squares
    }, 
    maps::{
        DIRECTIONAL_MAP_FILE, 
        DIRECTIONAL_MAP_RANK, 
        DIRECTIONAL_MAP_DD,
        DIRECTIONAL_MAP_DA,
        load_maps, get_ray_between_squares
    }, position::{Position, ZobristMoveStack, SidePiecesMethods, PositionEvaluation}, display::{print_position, print_bitboard}
};

fn main() {

    load_maps();

    let position = Position::new();

 
    let mut positions: Vec<Position> = Vec::new();
    positions.push(position);

    let mut last_game_state = GameState::IN_PROGRESS;

    let mut move_count = 0;

    while last_game_state == GameState::IN_PROGRESS || last_game_state == GameState::CHECK{
        let last_position = positions.last().unwrap();
        let eval = last_position.evaluate(None);
        last_game_state = eval.game_state;
        let mut rng = rand::thread_rng();
        let mut moves = eval.moves;
        //pick a random move
        let move_clone = moves.clone();
        if moves.len() == 0{
            println!("GameState: {}", last_game_state);
            break;
        }
        let random_move = move_clone.choose(&mut rng).clone().unwrap();
        
        let mut new_position = last_position.make_move(*random_move);
        positions.push(new_position);
        move_count += 1;
        if(move_count % 25 == 0){
            println!("Move count: {}", move_count);
            print_position(&position);
        }
    }

    println!("Game over! Move count: {}", move_count);
    let last_position = positions.last().unwrap();
    print_position(last_position);
    println!("GameState: {}", last_game_state);
    //debug eval last position
    let eval = last_position.evaluate(Some(true));
}


