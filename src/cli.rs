//! Subcommand CLI that merges the previous `train_random`, `train_random_evolve`,
//! `rank_gates`, `report_score`, and `plain_diamond_match` binaries.
//!
//! Usage:
//!     cli train   - run the random challenger training loop (replaces train_random)
//!     cli evolve  - run the mutate-and-evaluate training loop   (replaces train_random_evolve)
//!     cli rank    - run one match and dump gate-rank stats      (replaces rank_gates)
//!     cli report  - report the loaded TS brain vs random        (replaces report_score)
//!     cli match   - play a single match AI vs Human             (replaces plain_diamond_match)

use boardgame_binaries::game_utils::*;
use boardgame_game::game::connect4::Connect4;
use boardgame_game::game::*;
use boardgame_player::player::brain::LogicGatesError;
use boardgame_player::player::brain::lg_diamond::{generate_random_brain, BrainDiamond};
use boardgame_player::player::human::PlayerHuman;
use boardgame_player::player::plgnn_diamond::PlayerNNDiamond;
use boardgame_player::player::plgnn_diamond_tree_search::PlayerNNDiamondTS;
use boardgame_player::player::Agentish;
use log::{debug, info, warn};
use rand::distributions::{Alphanumeric, DistString};

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    let arg = std::env::args().nth(1);
    let cmd = arg.as_deref().unwrap_or("help");
    match cmd {
        "train" => train_random()?,
        "evolve" => train_random_evolve()?,
        "rank" => rank_gates()?,
        "report" => report_score()?,
        "match" => plain_diamond_match()?,
        _ => {
            eprintln!("usage: cli {{train|evolve|rank|report|match}}");
            std::process::exit(2);
        }
    }
    Ok(())
}

/// Original `train_random.rs` body. Trains a brain by mutating it via random
/// challengers and keeping whoever scores highest against random.
fn train_random() -> Result<(), anyhow::Error> {
    println!("Started");
    let mut gamestruct = Connect4::default();
    let mut old_wins = 40.0;
    let game_static = gamestruct.get_game_static();
    debug!("{}", gamestruct);

    let mut brain_a = BrainDiamond {
        game_name: game_static.name.clone(),
        ..BrainDiamond::default()
    };
    match brain_a.from_file() {
        Ok(_) => (),
        Err(e) => match e {
            LogicGatesError::Io { source: x } => {
                if x.kind() == std::io::ErrorKind::NotFound {
                    warn!("File not found loading brain. Generate a new one");
                    brain_a = generate_random_brain(&game_static);
                    brain_a.save_to_file()?;
                };
            }
            other_error => panic!("Problem creating the file: {other_error:?}"),
        },
    }

    let mut player_a = PlayerNNDiamond::default();
    player_a.get_ready(&game_static, "Red", 3)?;
    warn!("Start");
    loop {
        let name_b = Alphanumeric.sample_string(&mut rand::thread_rng(), 8);
        debug!("Ny utfordrer: {}", name_b);

        let mut player_b = PlayerNNDiamond::default();
        player_b.get_ready(&game_static, "Yellow", 3)?;

        gamestruct.reset();
        let winner = run_match(&mut gamestruct, &player_a, &player_b);
        match winner {
            0 | 2 => {
                gamestruct.reset();
                let winnera = run_match(&mut gamestruct, &player_b, &player_a);
                match winnera {
                    0 | 1 => {
                        let wins = mass_run_match_with_random_player(&mut gamestruct, &player_b, 100);
                        if wins < old_wins || wins < 45.0 {
                            info!("Wins to low {wins} < {old_wins}");
                            continue;
                        }
                        old_wins = mass_run_match_with_random_player(&mut gamestruct, &player_b, 100);
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

/// Original `train_random_evolve.rs` body. Mutate the loaded brain and keep
/// improvements.
fn train_random_evolve() -> Result<(), anyhow::Error> {
    println!("Started");
    let mut old_wins: f32 = 0.0;
    let mut gamestruct = Connect4::default();
    loop {
        gamestruct.reset();
        let game_static = gamestruct.get_game_static();
        debug!("La oss spille {}]. Jeg starter ...", game_static.name);

        let mut player_a = PlayerNNDiamond::default();
        player_a.get_ready(&game_static, "Red", 3)?;

        let name_b = Alphanumeric.sample_string(&mut rand::thread_rng(), 8);
        if old_wins == 0.0 {
            old_wins = mass_run_match_with_random_player(&mut gamestruct, &player_a, 100);
            warn!("Load old brain from disk with win chance of: {:.1}", old_wins);
        }
        debug!("Ny utfordrer: {}", name_b);
        let mut player_b = player_a.clone();
        player_b.get_ready(&game_static, "yellow", 3)?;
        let stat = gamestruct.get_game_static();

        // evolve loop
        loop {
            let state_size = stat.get_state_size();
            player_b.brain.do_mutate(&state_size);
            gamestruct.reset();

            // new first
            let winner = run_match(&mut gamestruct, &player_b, &player_a);
            gamestruct.reset();

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
            let winner = run_match(&mut gamestruct, &player_a, &player_b);
            gamestruct.reset();

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

            let new_old_wins = mass_run_match_with_random_player(&mut gamestruct, &player_b, 100);
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

/// Original `rank_gates.rs` body.
fn rank_gates() -> Result<(), anyhow::Error> {
    println!("Started");
    let mut gamestruct = Connect4::default();
    let game_static = gamestruct.get_game_static();
    let start_player = game_static.players[0].as_str();
    let mut player_a = PlayerNNDiamondTS::default();
    let mut player_b = PlayerNNDiamond::default();
    player_a.get_ready(&game_static, start_player, 3)?;
    player_b.get_ready(&game_static, start_player, 3)?;

    println!("Start single match");
    let mut statistics = run_match_return_statistics(&mut gamestruct, &player_a, &player_b);
    statistics.calculate();
    statistics.dump_rank_gates();
    statistics.dump();
    gamestruct.pretty_print();
    println!("Vinner % {:.1}", statistics.winner);
    Ok(())
}

/// Original `report_score.rs` body.
fn report_score() -> Result<(), anyhow::Error> {
    println!("Started");
    let mut gamestruct = Connect4::default();
    let game_static = gamestruct.get_game_static();
    let start_player = game_static.players[0].as_str();
    let mut player_a = PlayerNNDiamondTS {
        sec_to_move: 1,
        ..Default::default()
    };
    loop {
        player_a.get_ready(&game_static, start_player, 3)?;
        println!("Start mass_run");
        let wins = mass_run_match_with_random_player(&mut gamestruct, &player_a, 2);
        println!("Vinner % {:.1}", wins);
    }
}

/// Original `plain_diamond_match.rs` body.
fn plain_diamond_match() -> Result<(), anyhow::Error> {
    println!("Started");
    let mut gamestruct = Connect4::default();
    let game_static = gamestruct.get_game_static();
    let player_types = std::collections::HashMap::from([
        ("Red", "PlayerNN"),
        ("Yellow", "PlayerHuman"),
    ]);
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
        x => panic!("Unexpected player number {}", x),
    };
    let player_name = match player_types.get(winner.as_str()) {
        None => "Draw",
        Some(x) => x,
    };
    debug!("XXXX {player_name}");
    println!("Game ended. The winner is {}! The winners name is {}", winner, player_name);
    Ok(())
}
