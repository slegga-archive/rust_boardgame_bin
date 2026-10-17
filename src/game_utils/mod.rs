use boardgame_game::game::connect4::Connect4;
use boardgame_game::game::Playable;
use log::debug;
/// This to not have to mark players with game tags. Players should
/// have no info about the game.
use std::fmt::Display;

use boardgame_player::player::brain::CELL_SIZE;
use boardgame_player::player::brain::NO_LAYERS;
use boardgame_player::player::plgnn_diamond::PlayerNNDiamond;
use boardgame_player::player::random::PlayerRandom;
use boardgame_player::player::Agentish;
use boardgame_player::player::Brainy;

#[derive(Debug, Clone)]
pub struct Statistics {
    states: Vec<Evaluate>,
    pub winner: usize,
    rank_gates: Vec<usize>,
}

impl Default for Statistics {
    fn default() -> Self {
        Self {
            states: Vec::new(), // empty
            winner: 0,
            rank_gates: Vec::new(),
        }
    }
}

impl Statistics {
    pub fn dump(&self) -> () {
        println!("Move number, Move string, Apprentice score, Master score");
        for ev in self.states.iter() {
            println!(
                "{}     {}     {}     {}",
                ev.move_number, ev.move_string, ev.apprentice_score, ev.master_score
            );
        }
        //println!();
        //todo!("Lag dump funksjon for statistics")
    }

    /// calculate rank_gates to find the end gate that does least contribute to the best move, so a later logic
    /// can change this to something better.
    pub fn calculate(&mut self) -> () {
        if self.rank_gates.len() > 0 {
            panic!("rank_gates has value. Cannot add more. Expected empty!");
        }
        for _i in 0 .. CELL_SIZE {
            self.rank_gates.push(0);
        }
        for state in &self.states {
            let score_diff: isize = state.master_score as isize - state.apprentice_score as isize;
            if score_diff > 0 {  //diff is positive.
                for i in 0 .. CELL_SIZE {
                    if ! state.gate_result[i] {
                        self.rank_gates[i as usize] += score_diff as usize;
                    }
                }

            }
            else if score_diff < 0 { // diff is negative
                for i in 0 .. CELL_SIZE {
                    if state.gate_result[i] {
                        self.rank_gates[i as usize] -= score_diff as usize;
                    }
                }

            }

        }

    }

    pub fn dump_rank_gates(&self) -> () {
        println!("Gate number, Gate rank");
        for (i, &rank) in self.rank_gates.iter().enumerate() {
            println!("{}     {}", i, rank);
        }
    }
}

#[derive(Debug, Clone)]
struct Evaluate {
    pub move_number: usize,
    pub move_string: String,
    pub apprentice_score: usize,
    pub master_score: usize,
    pub gate_result: Vec<bool>,
    pub game_bit_state: Vec<bool>,
}

impl Default for Evaluate {
    fn default() -> Self {
        Self {
            move_number: 0,
            move_string: "".to_string(),
            apprentice_score: 0,
            master_score: 0,
            gate_result: Vec::new(),
            game_bit_state: Vec::new(),
        }
    }
}

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
pub fn mass_run_match_with_random_player<T: Playable + Display, U: Agentish + Clone>(
    game: &mut T,
    player: &U,
    tot_matches: usize,
) -> f32 {
    let game_static = game.get_game_static();
    let mut win_points: f32 = 0.0;
    let players = game.get_game_static().players;
    let random_player = PlayerRandom {
        name: "random".to_string(),
        name_in_game: players[1].clone(),
    };

    let first_role = game_static.players[0].as_str();
    let mut player_as_first = player.clone();
    let _ = player_as_first.get_ready(&game_static, first_role, 1);

    for _i in 0..(tot_matches / 2) {
        game.reset();
        let winner = run_match(game, &player_as_first, &random_player);
        match winner {
            0 => win_points += 50.0,
            1 => win_points += 100.0,
            2 => {}
            x => panic!("Unknown player number {}", x),
        }
    }
    let random_player = PlayerRandom {
        name: "random".to_string(),
        name_in_game: players[0].clone(),
    };

    let last_role = game_static.players[1].as_str();
    let mut player_as_last = player.clone();
    let _ = player_as_last.get_ready(&game_static, last_role, 1);

    for _i in 0..(tot_matches / 2) {
        game.reset();
        let winner = run_match(game, &random_player, &player_as_last);
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

pub fn run_match_return_statistics<U: Agentish + Brainy + Clone>(
    game: &mut Connect4,
    master_player: &U,
    apprentice_player: &PlayerNNDiamond,
) -> Statistics {
    if game.is_terminal() {
        panic!("Game can not start if it is already terminated. Make new game or do game.reset()");
    }
    let stat = game.get_game_static();
    let mut statistics = Statistics::default();
    let mut prev_move = "".to_string();
    let mut move_no = 0;
    //let thebrain = apprentice_player.brain;
    while !game.is_terminal() {
        let active_player = game.get_active_player();
        let mut evaluate = Evaluate::default();
        evaluate.move_string = prev_move;
        evaluate.move_number = move_no;
        //todo!("Expand trait Brainy with evaluate_bit_state",);
        let bit_state = game.get_bit_state(active_player.as_str());
        evaluate.apprentice_score = apprentice_player.evaluate_bit_state(game, &bit_state);
        evaluate.master_score = master_player.evaluate_bit_state(game, &bit_state);

        let cmove: String;
        let valid_moves = game.get_valid_moves();
        match valid_moves.len() {
            1 => cmove = valid_moves[0].to_string(),
            0 => panic!("Game active but no moves {}", game),
            // agent_get_moves is from players.rs. Handle all get_move from all players
            // Have tried to change the following line with player.get_move(....) but the compiler will then complain ∏∏
            // borrow checker mutiable / immutibate
            // if you want to try again, make an minimal complete example of code and ask on Stackoverflow or ask AI
            _ => {
                cmove = apprentice_player
                    .get_move(&valid_moves, &active_player, game)
                    .map_or("NO VALID MOVES".to_string(), |v| v);
            }
        }
        evaluate.gate_result = apprentice_player
            .brain
            .get_gate_result_for_level(bit_state.clone(), NO_LAYERS.try_into().unwrap());
        evaluate.game_bit_state = bit_state.clone();
        game.play(cmove.as_str()); // Mutable borrow happens after immutable borrow ends
        debug!("game after move: {}", game);
        statistics.states.push(evaluate.clone());
        prev_move = cmove.to_owned();
        move_no += 1;
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
    //u8::try_from(retur).unwrap()
    statistics.winner = retur;
    statistics
}
