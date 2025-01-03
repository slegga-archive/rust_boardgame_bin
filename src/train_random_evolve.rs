//! This crate is handling GGP for rust.

//mod game;
//mod player;
mod game_utils;

use crate::game_utils::*;
use boardgame_game::game::game::*;
use boardgame_player::player::player::*;
use boardgame_player::player::plgnn_diamond::plgnn_diamond::PlayerNNDiamond;
use log::{debug, info, warn};
use rand::distributions::{Alphanumeric, DistString};

/// Test like this:
/// cargo run --bin train_random
fn main() -> Result<(), anyhow::Error> {
    println!("Started");
    env_logger::init();
    let mut old_wins: f32 = 0.0;

    // Start from file
    let mut gamestruct = boardgame_game::game::connect4::connect4::Connect4::default();
    let mut game: Game = Game::Connect4(&mut gamestruct);
    loop {
        game.reset();

        let game_static = game.get_game_static();
        debug!("La oss spille {}]. Jeg starter ...", game_static.name);

        let mut player_a = PlayerNNDiamond::default();
        player_a.get_ready(&game_static, "Red")?;

        let name_b = Alphanumeric.sample_string(&mut rand::thread_rng(), 8);
        if old_wins == 0.0 {
            old_wins = mass_run_match_with_random_player(&mut game, &player_a, 100);
            warn!(
                "Load old brain from disk with win chance of: {:.1}",
                old_wins
            );
        }
        debug!("Ny utfordrer: {}", name_b);
        let mut player_b = player_a.clone(); //PlayerNeuralNetwork::default();
                                             //player_b.game_name = stat.players[1].clone();
                                             //player_b.brain.from_file()?;
        player_b.get_ready(&game_static, "yellow")?;
        let stat = game.get_game_static();
        // evolve loop
        loop {
            let state_size = stat.get_state_size();
            player_b.brain.do_mutate(&state_size);
            game.reset();

            // new first
            let winner = run_match(&mut game, &player_b, &player_a);
            game.reset();

            let mut score_new = 0;
            score_new += match winner {
                0 => 50,  //draw
                1 => 100, //win
                2 => 0,   //loose

                _ => {
                    panic!("Unexpected winner {}", winner);
                }
            };
            if score_new < 50 {
                break; // new player did not played equal or win against old
            }

            //new last
            let winner = run_match(&mut game, &player_a, &player_b);
            game.reset();

            score_new += match winner {
                0 => 50,  //draw
                2 => 100, //win
                1 => 0,   //loose

                _ => {
                    panic!("Unexpected winner {}", winner);
                }
            };
            if score_new <= 100 {
                break; // new player did not played equal or win against old
            }

            let new_old_wins = mass_run_match_with_random_player(&mut game, &player_b, 100);
            if new_old_wins < old_wins {
                info!(
                    "Lav score  og ,{:.1}. Kongen er {:.1}",
                    new_old_wins, old_wins
                );
                continue; // close but no sigar. evolve
            }
            old_wins = new_old_wins;
            if let Err(e) = player_b.brain.save_to_file() {
                panic!(
                    "Error saving to file: '{}'  {:?} ",
                    player_b.brain.filepath, e
                );
            }
            warn!("NEW KING {} Score {:.1} ()", player_b.get_name(), old_wins,);
        }
    }
}
