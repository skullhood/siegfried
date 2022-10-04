use crate::{position::{Position, Move}, tree::{PositionTree, ExpandStyle}, types::{Side, GameState, GameStateConstants}, display::print_position};

pub struct Game{
    position: Position,
    player_side: Option<Side>,
    move_history: Vec<Move>,
    max_depth: u8,
}

impl Game{
    pub fn new() -> Game{
        let position = Position::new_game();
        Game{
            position,
            player_side: None,
            move_history: Vec::new(),
            max_depth: 20,
        }
    }

    pub fn from_fen(fen: &str) -> Game{
        let position = Position::from_fen(fen);
        Game{
            position,
            player_side: None,
            move_history: Vec::new(),
            max_depth: 20,
        }
    }

    pub fn set_max_depth(&mut self, depth: u8){
        self.max_depth = depth;
    }

    pub fn clear(&self){
        print!("\x1B[2J\x1B[1;1H");
    }

    pub fn get_position(&self) -> &Position{
        &self.position
    }

    pub fn get_player_side(&self) -> Option<Side>{
        self.player_side
    }

    pub fn get_move_history(&self) -> &Vec<Move>{
        &self.move_history
    }

    fn make_move(&mut self, m: Move){
        println!("Move played: {} ", m);
        self.position = self.position.make_move(m);
        self.move_history.push(m);
        print_position(&self.position);
        println!("");
    }

    fn parse_move(&self, m: &str) -> Option<Move>{
        let mut moves = self.position.evaluate().moves;
        moves.sort_by(|a, b| a.get_tstring().cmp(&b.get_tstring()));
        for mov in moves{
            if mov.get_tstring() == m{
                return Some(mov);
            }
        }
        None
    }

    fn get_player_move(&self) -> Move{
        let mut input = String::new();

        loop{
            input.clear();
            std::io::stdin().read_line(&mut input).unwrap();

            //parse input
            let input = input.trim();
            let input = input.to_lowercase();
            let m = self.parse_move(&input);

            if m.is_some(){
                return m.unwrap();
            }
            else{
                println!("Invalid Move: '{}'!, Try again: ", input);
            }
        }
    }

    pub fn get_pgn(&self) -> String{
        let mut pgn = String::new();
        let mut move_count = 1;
        let mut white_plays = true;
        for m in &self.move_history{
            if white_plays{
                pgn += &format!("{}. ", move_count);
                move_count += 1;
            }
            pgn.push_str(&m.get_tstring());
            pgn.push_str(" ");
            white_plays = !white_plays;
        }
        pgn
    }

    pub fn play(&mut self, player: Option<Side>){
        self.player_side = player;

        let side_to_move = self.position.side_to_move;

        println!("New game: ");

        print_position(&self.position);

        if self.player_side.is_some(){
            let eval = self.position.evaluate();
            let game_state = eval.game_state;

            while game_state == GameState::ONGOING || game_state == GameState::CHECK{
                if self.player_side.unwrap() == self.position.side_to_move{
                    println!("Player's turn: ");
                    let m = self.get_player_move();
                    self.make_move(m);
                }
                else{
                    println!("Computer is thinking...");
                    let mut tree = PositionTree::new(self.position);
                    let best_moves = tree.expand_to_depth(self.max_depth, ExpandStyle::DEFAULT, self.position.side_to_move);
                    let best_move = best_moves[0];
                    self.make_move(best_move);
                }
            }
        }
        else{
            let eval = self.position.evaluate();
            let game_state = eval.game_state;
            while game_state == GameState::ONGOING || game_state == GameState::CHECK{
                let mut tree = PositionTree::new(self.position);
                let best_moves = tree.expand_to_depth(self.max_depth, ExpandStyle::DEFAULT, self.position.side_to_move);
                let best_move = best_moves[0];
                self.make_move(best_move);
            }
        }

        let eval = self.position.evaluate();
        let game_state = eval.game_state;
        let state_note = if eval.state_note.is_some() { eval.state_note.unwrap() } else { "None".to_string() };
        if game_state == GameState::CHECKMATE{
            println!("Checkmate! {} wins!", !side_to_move);
        }
        else{
            println!("Draw! Reason: {}", state_note);
        }

        println!("PGN: {}", self.get_pgn());

    }

}