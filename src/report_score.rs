//! This crate is handling GGP for rust.
//! To start this script
//! cargo run --bin report_score

mod game_utils;
use crate::game_utils::*;
use boardgame_game::game::*;
use boardgame_player::player::plgnn_diamond_tree_search::PlayerNNDiamondTS;
use boardgame_player::player::Agentish;

/// Test like this:
/// cargo run --bin train_random
fn main() -> Result<(), anyhow::Error> {
    println!("Started");
    env_logger::init();
    // let mut old_wins = 50;
    let mut gamestruct = boardgame_game::game::connect4::Connect4::default();
    let mut game: Game = Game::Connect4(&mut gamestruct);
    let game_static = game.get_game_static();
    let start_player = game_static.players[0].as_str();
    let mut player_a = PlayerNNDiamondTS {
        sec_to_move: 1,
        ..Default::default()
    };
    loop {
        player_a.get_ready(&game_static, start_player, 3)?;

        println!("Start mass_run");
        let wins = mass_run_match_with_random_player(&mut game, &player_a, 2);

        println!("Vinner % {:.1}", wins);
    }
}
