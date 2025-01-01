//! This crate is handling GGP for rust.

//mod brain;
//mod game;
//mod player;
mod game_utils;
use boardgame_game::game::game::*;
use boardgame_player::player::plgnn_diamond::plgnn_diamond::PlayerNNDiamond;
//use boardgame_player::player::player::*;
//use boardgame_player::player::brain::brain::NO_LAYERS;
use env_logger;
use log::{debug, info, warn};
use rand::distributions::{Alphanumeric, DistString};
use crate::game_utils::game_utils::*;
use boardgame_player::player::brain::brain::LogicGatesError;
use boardgame_player::player::brain::lg_diamond::lg_diamond::BrainDiamond;
use boardgame_player::player::brain::lg_diamond::lg_diamond::generate_random_brain;
use boardgame_player::player::player::Agentish;

/// Test like this:
///
/// cargo run --bin train_random

fn main() -> Result<(), anyhow::Error> {
    println!("Started");
    let mut gamestruct = boardgame_game::game::connect4::connect4::Connect4::default();
    let mut game: Game = Game::Connect4(&mut gamestruct); // Combined::InnerOne(InnerOne);
    env_logger::init();
    let mut old_wins = 40.0;
    let game_static = game.get_game_static();
    debug!("{}", game);
    let mut brain_a = BrainDiamond {game_name: game_static.name.clone(),..BrainDiamond::default()};// {

      //  filepath: "".to_string(),
      //  layers: [layer; NO_LAYERS],
    //};
    match brain_a.from_file() {
        Ok(_value) => (),
        Err(e) => match e {
            LogicGatesError::Io { source: x } => {
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
    // let stat = game.get_game_static();
    /*let pos = stat
                .players
                .iter()
                .position(|x| x.eq(&active_player) == true);
    */
    let mut player_a = PlayerNNDiamond::default(); /*{
        name: "from_disk".to_string(),
        is_loaded: true,
        brain: brain_a,
        me_color: "Red".to_string(),
        opponent_color: "Yellow".to_string(),
    };*/
    player_a.get_ready(&game_static, "Red")?;
    warn!("Start");
    loop {
        // let mut player_library = PlayerLibrary::new();
        //brain boilerplate

        let name_b = Alphanumeric.sample_string(&mut rand::thread_rng(), 8);
        debug!("Ny utfordrer: {}", name_b);

        let mut player_b = PlayerNNDiamond::default();
        player_b.get_ready(&game_static, "Yellow")?;

         /*{
            name: name_b,
            is_loaded: true,
            brain: generate_random_brain(&game_static),
        };*/
        game.reset();
        //    todo!("Lag while under til en funksjon som tar player_a og player_b som innputt. Returnerer &str 'A' hvis 'A' vant og 'B' hvis B vant");
        let winner = run_match(&mut game, &player_a, &player_b);
        match winner {
            0 | 2 => {
                game.reset();
                let winnera = run_match(&mut game, &player_b, &player_a);
                match winnera {
                    0 | 1 => {
                        let wins = mass_run_match_with_random_player(&mut game, &player_b, 100);
                        if wins < old_wins || wins < 45.0 {
                            info!("Wins to low {wins} < {old_wins}");
                            continue;
                        }
                        old_wins = mass_run_match_with_random_player(&mut game, &player_b, 100);
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
