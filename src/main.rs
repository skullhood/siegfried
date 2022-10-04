
use siegfried::game::Game;
use siegfried::types::{Side, SideConstants};

fn get_player_side() -> Option<Side>{
    let mut input = String::new();
    let side;

    println!("Choose side (w/b/n):");

    loop{
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();

        //parse input
        let input = input.trim();
        let input = input.to_lowercase();

        if input == "w" || input == "white"{
            side = Some(Side::WHITE);
            break;
        }
        else if input == "b" || input == "black"{
            side = Some(Side::BLACK);
            break;
        }
        else if input == "n" || input == "none"{
            side = None;
            break;
        }
        else{
            println!("Invalid side: '{}'!, Try again: ", input);
        }
    }
    side
}

fn  main() {

    let player_side: Option<Side> = get_player_side();
    
    let mut game = Game::new();

    game.play(player_side);

    println!("Game over! Thanks for playing!");

    //wait for input to keep console open
    let mut input = String::new();
    input.clear();
    std::io::stdin().read_line(&mut input).unwrap();
}


