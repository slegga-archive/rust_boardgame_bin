//! This crate is handling GGP for rust.
//! To use this crate
//! use crate::player::plgnn_diamond_tree_search::*;
//! use crate::player::brain::brain::*;
//! use crate::plgnn_diamond_tree_search::PlayerNNDiamondTS;

mod game_utils;
use crate::game_utils::*;
use boardgame_game::game::connect4::connect4::Connect4;
use boardgame_game::game::game::*;
use boardgame_player::player::human::human::PlayerHuman;
use boardgame_player::player::player::Agentish;
use boardgame_player::player::plgnn_diamond_tree_search::plgnn_diamond_tree_search::PlayerNNDiamondTS;

//use boardgame_player::player::plgnn_diamond_tree_search;
//use boardgame_player::player::player::PlayerNNDiamondTS;

use log::debug;
use std::collections::HashMap;

/// To run a match RUST_LOG=debug cargo run --bin game_match
/// This file is master of linking player and game.
fn main() -> Result<(), anyhow::Error> {
    println!("Started");
    env_logger::init();
    //let mut gamestruct = crate::game::connect4::connect4::Connect4::default();
    //let game: Game = Game::Connect4(&mut gamestruct); // Combined::InnerOne(InnerOne);
    // let player_library = PlayerLibrary::new();
    let mut game = Connect4::default();
    println!(
        "La oss spille {}. Jeg starter ...",
        game.get_game_static().name
    );

    let game_static = game.get_game_static();
    let player_types = HashMap::from([("Red", "PlayerMC"), ("Yellow", "PlayerHuman")]);
    let mut player_a = PlayerNNDiamondTS::default();
    player_a.get_ready(&game_static, "Red")?;

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
        x => panic!("Unexpected player number {}", x), // This should never happen
    };
    let player_name = match player_types.get(winner.as_str()) {
        Some(x) => x,
        None => "Draw",
    };

    game.pretty_print();
    //println!("{}",game.clone());
    println!(
        "Game ended. The winner is {}! The winners name is {}",
        winner, player_name
    );
    Ok(())
}
