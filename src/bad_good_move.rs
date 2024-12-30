//! This crate is handling GGP for rust.

mod game;
mod logic_gates;
mod players;

//use std::collections::HashMap;

//use crate::connect4::connect4::*;
use crate::game::game::*;
use crate::logic_gates::logic_gates::*;
//use crate::nim12::nim12::*;

use crate::players::players::*;
use env_logger;
use log::{debug, info, warn};
use rand::distributions::{Alphanumeric, DistString};
use std::io::ErrorKind;

/// Test like this:
///
/// cargo run --bin train_random

fn main() -> Result<(), anyhow::Error> {
    println!("Started");
    let mut gamestruct = crate::game::connect4::connect4::Connect4::default();
    let mut game: Game = Game::Connect4(&mut gamestruct); // Combined::InnerOne(InnerOne);
    env_logger::init();
    let mut old_wins = 40.0;
    let game_static = game.get_game_static();
    debug!("{}", game);
    let layer = get_default_layer();
    let mut brain_a = Brain {
        game_name: game_static.name.clone(),
        filepath: "".to_string(),
        layers: [layer; NO_LAYERS],
    };
    match brain_a.from_file() {
        Ok(_value) => (),
        Err(e) => match e {
            crate::LogicGatesError::Io { source: x } => {
                if x.kind() == std::io::ErrorKind::NotFound {
                    warn!("File not found loading brain. Generate a new one");
                    brain_a = generate_random_brain(&game_static);
                    brain_a.save_to_file()?;
                    ()
                };
            }
            other_error => panic!("Problem creating the file: {other_error:?}"),
        },
    }
    let stat = game.get_game_static();
    /*let pos = stat
                .players
                .iter()
                .position(|x| x.eq(&active_player) == true);
    */
    let mut player_a = PlayerNeuralNetwork {
        name: "from_disk".to_string(),
        is_loaded: true,
        brain: brain_a,
    };
    warn!("Start");
    loop {
        // let mut player_library = PlayerLibrary::new();
        //brain boilerplate

        let name_b = Alphanumeric.sample_string(&mut rand::thread_rng(), 8);
        debug!("Ny utfordrer: {}", name_b);

        let player_b = PlayerNeuralNetwork {
            name: name_b,
            is_loaded: true,
            brain: generate_random_brain(&game_static),
        };
        game.reset();
        //    todo!("Lag while under til en funksjon som tar player_a og player_b som innputt. Returnerer &str 'A' hvis 'A' vant og 'B' hvis B vant");
        let winner = run_match(&mut game, &player_a, &player_b);
        match winner {
            0 | 2 => {
                let winnera = run_match(&mut game, &player_b, &player_a);
                match winnera {
                    0 | 1 => {
                        //     let wins = mass_run_match_with_random_player(&mut game, &player_b);
                        //     if wins < old_wins || wins < 45.0 {
                        //         debug!("Wins to low {wins} < {old_wins}");
                        //         continue;
                        //     }
                        old_wins = mass_run_match_with_random_player(&mut game, &player_b);
                        if let Err(e) = player_b.brain.save_to_file() {
                            panic!("Error saving to file {}", e);
                        }
                        warn!("NEW KING {} Score {:.1}", player_b.get_name(), old_wins);
                        player_a = player_b.clone();
                    }
                    2 => {
                        info!("King stands. Vinner: {} {}", winnera, player_a.get_name());
                    }
                    x => panic!("Unexpected player number {}", x),
                };
            }
            1 => {
                info!("Ufordrer tapte. Vinner {} {}", winner, player_a.get_name());
            }
            x => panic!("Unexpected player number {}", x),
        }
    }
}
