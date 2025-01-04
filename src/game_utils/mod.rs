use boardgame_game::game::Playable;
use log::debug;
/// This to not have to mark players with game tags. Players should
/// have no info about the game.
use std::fmt::Display;

use boardgame_player::player::player::Agentish;
use boardgame_player::player::random::random::PlayerRandom;
pub fn run_match<T: Playable + Display>(
    game: &mut T,
    player_a: &impl Agentish,
    player_b: &impl Agentish,
) -> u8 {
    if game.is_terminal() {
        panic!("Game can not start if it is already terminated. Make new game or do game.reset()");
    }
    let stat = game.get_game_static();
    while !game.is_terminal() {
        let active_player = game.get_active_player();

        let pos = stat.players.iter().position(|x| x.eq(&active_player));
        debug!(
            "Player type: {:?} {}",
            match pos {
                Some(x) => {
                    match x {
                        0 => player_a.get_name(),
                        1 => player_b.get_name(),
                        n => panic!("Player out of range: {n}"),
                    }
                }
                None => panic!("Unknown player name {active_player}"),
            },
            active_player
        );
        let cmove: String;
        let valid_moves = game.get_valid_moves();
        {
            match valid_moves.len() {
                1 => cmove = valid_moves[0].to_string(),
                0 => panic!("Game active but no moves {}", game),
                // agent_get_moves is from players.rs. Handle all get_move from all players
                // Have tried to change the following line with player.get_move(....) but the compiler will then complain ∏∏
                // borrow checker mutiable / immutibate
                // if you want to try again, make an minimal complete example of code and ask on Stackoverflow or ask AI
                _ => {
                    cmove = match pos {
                        Some(x) => match x {
                            0 => player_a
                                .get_move(&valid_moves, &active_player, game)
                                .map_or("NO VALID MOVES".to_string(), |v| v),
                            1 => player_b
                                .get_move(&valid_moves, &active_player, game)
                                .map_or("NO VALID MOVES".to_string(), |v| v),
                            n => panic!("Player out of range: {n}"),
                        },
                        None => panic!("Unknown player name {active_player}"),
                    }
                }
            }
        }

        game.play(cmove.as_str()); // Mutable borrow happens after immutable borrow ends
        debug!("game after move: {}", game);
    }
    //let retur = 0u8;
    let winner = match game.get_winner() {
        Some(x) => x,
        None => "Draw".to_string(),
    };
    let pos = stat.players.iter().position(|x| x.eq(&winner));
    debug!("After play {}", winner);
    let retur = match pos {
        Some(x) => x + 1,
        None => 0,
    };
    //      println!("Vinner: {}", retur);
    u8::try_from(retur).unwrap()
}

///run 1000 matches against randomplayer. 500 as first player and 500 as last player.
#[allow(dead_code)]
pub fn mass_run_match_with_random_player<T: Playable + Display>(
    game: &mut T,
    player: &impl Agentish,
    tot_matches: usize,
) -> f32 {
    let mut win_points: f32 = 0.0;
    let players = game.get_game_static().players;
    let random_player = PlayerRandom {
        name: "random".to_string(),
        name_in_game: players[0].clone(),
    };
    for _i in 0..(tot_matches / 2) {
        game.reset();
        let winner = run_match(game, player, &random_player);
        match winner {
            0 => win_points += 50.0,
            1 => win_points += 100.0,
            2 => {}
            x => panic!("Unknown player number {}", x),
        }
    }
    let random_player = PlayerRandom {
        name: "random".to_string(),
        name_in_game: players[1].clone(),
    };
    for _i in 0..(tot_matches / 2) {
        game.reset();
        let winner = run_match(game, &random_player, player);
        match winner {
            0 => win_points += 50.0,
            2 => win_points += 100.0,
            1 => {}
            x => panic!("Unknown player number {}", x),
        }
    }

    // return
    win_points / tot_matches as f32
}
