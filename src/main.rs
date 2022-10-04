use std::io::BufRead;

use siegfried::game::Game;
use siegfried::types::{Side, SideConstants};

fn  main() {

    //ask user for side
    let input = std::io::stdin();
    let mut input = input.lock();
    let mut side = String::new();
    let mut player_side: Option<Side> = None;

    loop{
        side.clear();
        println!("Choose side (w/b/none): ");
        input.read_line(&mut side).unwrap();
        let side = side.trim().to_lowercase();
        if side == "w"{
            player_side = Some(Side::WHITE);
            break;
        }
        else if side == "b"{
            player_side = Some(Side::BLACK);
            break;
        }
        else if side == "none"{
            break;
        }
        else{
            println!("Invalid side: '{}', try again: ", side);
        }
    }

    let mut game = Game::new();

    game.play(player_side);

    println!("Game over! Thanks for playing!");

    //wait for input to keep console open
    let mut input = String::new();
    input.clear();
    std::io::stdin().read_line(&mut input).unwrap();
}


