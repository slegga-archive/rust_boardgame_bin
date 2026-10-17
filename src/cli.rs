//! Subcommand CLI that merges the previous `train_random`, `train_random_evolve`,
//! `rank_gates`, `report_score`, and `plain_diamond_match` binaries.
//!
//! Usage:
//!     cli train   - run the random challenger training loop (replaces train_random)
//!     cli evolve  - run the mutate-and-evaluate training loop   (replaces train_random_evolve)
//!     cli rank    - run one match and dump gate-rank stats      (replaces rank_gates)
//!     cli report  - report the loaded TS brain vs random        (replaces report_score)
//!     cli match   - play a single match AI vs Human             (replaces plain_diamond_match)

//!     cli match   - play a single match AI vs Human             (replaces plain_diamond_match)
//!     cli bench [n] [sec] - benchmark PlayerNNDiamondTS vs PlayerNNDiamond on Connect4

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
use std::cell::Cell;
use std::fmt::Display;
use std::time::Instant;

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "help".to_string());
    let rest: Vec<String> = args.collect();
    match cmd.as_str() {
        "train" => train_random()?,
        "evolve" => train_random_evolve()?,
        "rank" => rank_gates()?,
        "report" => report_score()?,
        "match" => plain_diamond_match()?,
        "bench" => bench(
            rest.first().and_then(|s| s.parse().ok()).unwrap_or(20),
            rest.get(1).and_then(|s| s.parse().ok()).unwrap_or(1),
        )?,
        _ => {
            eprintln!("usage: cli {{train|evolve|rank|report|match|bench}} [args]");
            std::process::exit(2);
        }
    }
    Ok(())
}

/// Wraps an agent and accumulates time spent in get_move. Shared-ref friendly
/// via Cell since run_match takes `&impl Agentish`.
struct Timed<'a, T: Agentish> {
    inner: &'a T,
    secs: Cell<f32>,
    moves: Cell<u64>,
}

impl<T: Agentish> Agentish for Timed<'_, T> {
    fn get_move<T2: Playable + Display>(
        &self,
        moves: &[String],
        game_player: &str,
        game: &T2,
    ) -> Option<String> {
        let start = Instant::now();
        let r = self.inner.get_move(moves, game_player, game);
        self.secs.set(self.secs.get() + start.elapsed().as_secs_f32());
        self.moves.set(self.moves.get() + 1);
        r
    }
    fn get_name(&self) -> String {
        self.inner.get_name()
    }
    fn get_ready(
        &mut self,
        _game_static: &GameStatic,
        _me_color: &str,
        _sec_to_move: u64,
    ) -> Result<(), LogicGatesError> {
        Ok(())
    }
}

/// Bench: PlayerNNDiamondTS (PUCT + TT + tree reuse) vs PlayerNNDiamond on
/// Connect4. n/2 games each color. Prints win % and time per move
/// (arena node count is not exposed by the player crate, so skipped).
fn bench(n_parties: usize, sec_to_move: u64) -> Result<(), anyhow::Error> {
    let mut gamestruct = Connect4::default();
    let game_static = gamestruct.get_game_static();
    let start_player = game_static.players[0].as_str();

    let mut ts = PlayerNNDiamondTS {
        sec_to_move,
        ..Default::default()
    };
    ts.get_ready(&game_static, start_player, sec_to_move)?;
    let mut plain = PlayerNNDiamond::default();
    plain.get_ready(&game_static, start_player, 3)?;

    let mut ts_score = 0.0f32; // win=1, draw=0.5
    let mut ts_secs = 0.0f32;
    let mut ts_moves = 0u64;
    let mut plain_secs = 0.0f32;
    let mut plain_moves = 0u64;

    for i in 0..n_parties {
        let ts_timed = Timed {
            inner: &ts,
            secs: Cell::new(0.0),
            moves: Cell::new(0),
        };
        let plain_timed = Timed {
            inner: &plain,
            secs: Cell::new(0.0),
            moves: Cell::new(0),
        };
        gamestruct.reset();
        // alternate colors: even i -> TS first (winner 1), odd i -> TS second (winner 2)
        let winner = if i % 2 == 0 {
            run_match(&mut gamestruct, &ts_timed, &plain_timed)
        } else {
            run_match(&mut gamestruct, &plain_timed, &ts_timed)
        };
        ts_score += match (i % 2 == 0, winner) {
            (_, 0) => 0.5,
            (true, 1) | (false, 2) => 1.0,
            _ => 0.0,
        };
        ts_secs += ts_timed.secs.get();
        ts_moves += ts_timed.moves.get();
        plain_secs += plain_timed.secs.get();
        plain_moves += plain_timed.moves.get();
        info!("game {i}: winner={winner}");
    }

    let ts_ms = if ts_moves > 0 { ts_secs * 1000.0 / ts_moves as f32 } else { 0.0 };
    let plain_ms = if plain_moves > 0 { plain_secs * 1000.0 / plain_moves as f32 } else { 0.0 };
    println!(
        "BENCH n={} sec_to_move={}s | TS win%: {:.1} (score {:.1}/{}) | TS: {:.1} ms/move over {} moves | Plain: {:.1} ms/move over {} moves",
        n_parties,
        sec_to_move,
        100.0 * ts_score / n_parties as f32,
        ts_score,
        n_parties,
        ts_ms,
        ts_moves,
        plain_ms,
        plain_moves
    );
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
