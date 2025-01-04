//! This crate is handling GGP for rust.

mod game_utils;
//mod player;
//use crate::game_utils::run_match;
use boardgame_game::game::*;
//use boardgame_player::player::player::*;
use boardgame_player::player::human::human::PlayerHuman;
//use crate::player::human::*;
use crate::game_utils::*;
use crate::plgnn_diamond::PlayerNNDiamond;
use boardgame_game::game::Game;
use boardgame_player::player::plgnn_diamond::*;
//use crate::player::monte_carlo::*;

//use crate::Game;
use log::debug;
use std::collections::HashMap;

/// To run a match RUST_LOG=debug cargo run --bin game_match
/// This file is master of linking player and game.
fn main() {
    println!("Started");
    env_logger::init();
    let mut gamestruct = boardgame_game::game::connect4::Connect4::default();
    let game: Game = Game::Connect4(&mut gamestruct); // Combined::InnerOne(InnerOne);
                                                      // let player_library = PlayerLibrary::new();
    println!(
        "La oss spille {}. Jeg starter ...",
        game.get_game_static().name
    );

    let game_static = game.get_game_static();
    let player_types = HashMap::from([("Red", "PlayerMC"), ("Yellow", "PlayerHuman")]);
    let player_a = PlayerNNDiamond::default();

    let player_b: PlayerHuman = PlayerHuman {
        name: "Human".to_string(),
    };
    debug!("BEFORE run_match");

    let win = run_match(&mut gamestruct, &player_a, &player_b);
    debug!("match ended");
    let winner = match win {
        1 => game_static.players[0].clone(),
        2 => game_static.players[1].clone(),
        0 => "DRAW".to_string(),
        x => panic!("Unexpected player number {}", x), // This should never happen
    };
    let player_name = match player_types.get(winner.as_str()) {
        None => "Draw",
        Some(x) => x,
    };

    debug!("XXXX {player_name}");

    println!(
        "Game ended. The winner is {}! The winners name is {}",
        winner, player_name
    );
}
