//! Main entry: human vs AI Connect4 match using PlayerNNDiamondTS.

use boardgame_binaries::game_utils::*;
use boardgame_game::game::connect4::Connect4;
use boardgame_game::game::*;
use boardgame_player::player::human::PlayerHuman;
use boardgame_player::player::plgnn_diamond_tree_search::PlayerNNDiamondTS;
use boardgame_player::player::Agentish;

use log::debug;
use std::collections::HashMap;

/// To run a match RUST_LOG=debug cargo run --bin game_match
fn main() -> Result<(), anyhow::Error> {
    println!("Started");
    env_logger::init();
    let mut game = Connect4::default();
    println!(
        "La oss spille {}. Jeg starter ...",
        game.get_game_static().name
    );

    let game_static = game.get_game_static();
    let player_types = HashMap::from([("Red", "PlayerMC"), ("Yellow", "PlayerHuman")]);
    let mut player_a = PlayerNNDiamondTS::default();
    player_a.get_ready(&game_static, "Red", 10)?;

    let player_b: PlayerHuman = PlayerHuman {
        name: "Human".to_string(),
    };
    debug!("BEFORE run_match");

    let win = run_match(&mut game, &player_a, &player_b);
    debug!("match ended");
    let winner = match win {
        1 => game_static.players[0].clone(),
        2 => game_static.players[1].clone(),
        0 => "DRAW".to_string(),
        x => panic!("Unexpected player number {}", x),
    };
    let player_name = match player_types.get(winner.as_str()) {
        Some(x) => x,
        None => "Draw",
    };

    game.pretty_print();
    println!(
        "Game ended. The winner is {}! The winners name is {}",
        winner, player_name
    );
    Ok(())
}
