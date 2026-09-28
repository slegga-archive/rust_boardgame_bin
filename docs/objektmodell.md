# Objekt- og filmodell for rust_boardgame_*-repoene

Modellen dekker de tre repoene som hører sammen:

| Repo | Crate | Type | Innhold |
|------|-------|------|---------|
| `rust_boardgame_game` | `boardgame_game` | bibliotek | spillregler (spill-objekter) |
| `rust_boardgame_player` | `boardgame_player` | bibliotek | spillere/agenter og "hjerner" (logikkport-nettverk) |
| `rust_boardgame_bin` | `boardgame_binaries` | binærer | kjørbare programmer + kamp-logikk (`game_utils`) |

Diagrammene er Mermaid. De kan vises direkte på GitHub/GitLab, i VS Code
(Markdown Preview Mermaid), eller på <https://mermaid.live>.

Samme modell finnes i to alternative formater for sammenligning:
`objektmodell.d2` (D2, rendres med `d2 objektmodell.d2 objektmodell.svg`) og
`objektmodell.puml` (PlantUML, rendres med `plantuml objektmodell.puml`).

---

## 1. Avhengigheter mellom crates

`boardgame_binaries` er toppen av haugen og eier alle `[[bin]]`-programmene.
`boardgame_game` er den eneste som ikke avhenger av de andre.

```mermaid
flowchart RL
    subgraph game_repo["rust_boardgame_game (crate: boardgame_game)"]
        lib_game["lib.rs -> game/"]
    end
    subgraph player_repo["rust_boardgame_player (crate: boardgame_player)"]
        lib_player["lib.rs -> player/"]
    end
    subgraph bin_repo["rust_boardgame_bin (crate: boardgame_binaries)"]
        bins["6 x [[bin]] + game_utils"]
    end

    lib_player -->|"path = ../rust_boardgame_game/"| lib_game
    bins -->|"path = ../rust_boardgame_game/"| lib_game
    bins -->|"path = ../rust_boardgame_player/"| lib_player
```

---

## 2. Filer og moduler

```mermaid
flowchart TB
    subgraph GAME["rust_boardgame_game/src"]
        g_lib["lib.rs"]
        g_mod["game/mod.rs"]
        g_c4["game/connect4.rs"]
        g_nim["game/nim12.rs"]
        g_lib --> g_mod
        g_mod --> g_c4
        g_mod --> g_nim
    end

    subgraph PLAYER["rust_boardgame_player/src"]
        p_lib["lib.rs"]
        p_mod["player/mod.rs"]
        p_rnd["player/random.rs"]
        p_hum["player/human.rs"]
        p_plgnn["player/plgnn_diamond.rs"]
        p_ts["player/plgnn_diamond_tree_search.rs"]
        p_bmod["player/brain/mod.rs"]
        p_lg["player/brain/lg_diamond.rs"]
        p_lib --> p_mod
        p_mod --> p_rnd
        p_mod --> p_hum
        p_mod --> p_plgnn
        p_mod --> p_ts
        p_mod --> p_bmod
        p_bmod --> p_lg
    end

    subgraph OLD["rust_boardgame_player/old (ikke kompilert)"]
        o_mc["monto_carlo_simple.rs"]
        o_plgnn["plgnn_trunc.rs"]
        o_brain["brain/lg_trunc.rs"]
    end

    subgraph BIN["rust_boardgame_bin/src"]
        b_utils["game_utils/mod.rs"]
        b_match["main.rs -> game_match"]
        b_plain["plain_diamond_match.rs"]
        b_train["train_random.rs"]
        b_evol["train_random_evolve.rs"]
        b_rank["rank_gates.rs"]
        b_score["report_score.rs"]
    end

    subgraph DATA["rust_boardgame_bin/data"]
        d_brain["Connect4-lgnndiamond-6-256.txt"]
    end

    p_mod -.->|"importerer crate"| g_mod
    b_utils -.->|"importerer crate"| g_mod
    b_utils -.->|"importerer crate"| p_mod
    b_match --> b_utils
    b_plain --> b_utils
    b_train --> b_utils
    b_evol --> b_utils
    b_rank --> b_utils
    b_score --> b_utils
    p_lg <-->|"from_file / save_to_file"| d_brain
```

`old/`-katalogen ligger utenfor `src/` og kompileres ikke. Den er en tidligere
variant av samme mønster (`Brain`/`Layer`, `PlayerMCS`, `PlayerNeuralNetwork`).

---

## 3. Objektmodell – spill (`boardgame_game`)

Kjernegrensesnittet er traiten `Playable`. Alle spill implementerer den, og
enumen `Game` videresender til dem.

```mermaid
classDiagram
    class Playable {
        <<trait>>
        +reset()
        +get_game_static() GameStatic
        +get_valid_moves() Vec~String~
        +is_terminal() bool
        +get_winner() Option~String~
        +get_active_player() String
        +play(play_move str)
        +get_bit_state(perspective str) Vec~bool~
        +get_bit_state_from_bit_state_and_move(perspective, tmp_state, cmove) Vec~bool~
        +get_valid_moves_from_bit_state(perspective, tmp_state) Vec~String~
        +pretty_print()
    }

    class GameStatic {
        +name String
        +players Vec~String~
        -state_size usize
        +new(name, players, state_size) GameStatic
        +get_state_size() usize
    }

    class Game {
        <<enum>>
        Connect4 mut ref
        Nim12 mut ref
    }

    class TerminalState {
        <<enum>>
        Me
        Opponent
        Draw
    }

    class Connect4 {
        -turn C4Player
        -table u8[7][6]
        -is_terminated bool
        -winner Option~C4Player~
        -last_printed_table u8[7][6]
    }

    class C4Player {
        <<enum>>
        Red
        Yellow
    }

    class Nim12 {
        -turn bool
        -tokens_left u8
    }

    class game_funksjoner {
        <<modul-funksjoner>>
        get_terminal_state_from_bit_state(state) Option~TerminalState~
        get_active_player_from_bit_state(game_static, state) String
    }

    Playable <|.. Connect4 : impl
    Playable <|.. Nim12 : impl
    Playable <|.. Game : dispatch paa variant
    Game o-- Connect4 : holder mut ref
    Game o-- Nim12 : holder mut ref
    Connect4 *-- C4Player : tur og vinner
    Connect4 ..> GameStatic : returneres av get_game_static
    Nim12 ..> GameStatic : returneres av get_game_static
    game_funksjoner ..> TerminalState : returnerer
    game_funksjoner ..> GameStatic : leser players
    Playable ..> GameStatic : get_game_static
```

**Bit-tilstand:** `get_bit_state` returnerer en `Vec<bool>` per posisjon.
Indeksene 2-6 har fast betydning (brukt av hjelpesfunksjonene):
`state[2]`/`state[3]` = tur, `state[4]` = uavgjort, `state[5]` = "jeg vinner",
`state[6]` = "motstander vinner".

---

## 4. Objektmodell – spillere og hjerner (`boardgame_player`)

To traits: `Agentish` (velger trekk) og `Brainy` (vurderer stilling via en
hjerne). Alle agenter implementerer `Agentish`; de nevrale spillerne
implementerer i tillegg `Brainy` med `MyBrain = BrainDiamond`.

```mermaid
classDiagram
    class Agentish {
        <<trait>>
        +get_move(moves, game_player, game generisk T) Option~String~
        +get_name() String
        +get_ready(game_static, me_color, sec_to_move) Result~LogicGatesError~
    }

    class Brainy {
        <<trait>>
        +evaluate_bit_state(game Game, bit_state) usize
        +get_brain() MyBrain
    }

    class PlayerNNDiamond {
        +name String
        +is_loaded bool
        +brain BrainDiamond
    }

    class PlayerNNDiamondTS {
        +name String
        +is_loaded bool
        +brain BrainDiamond
        +sec_to_move u64
        +me_color String
        +opponent_color String
        -has_time_left(move_start) bool
        -select_leaf(states) usize og Option~TSNode~
        -explore(game, states)
        -backpropagate(states)
        -add_leaf(states)
        -get_best_move_and_score(states)
        +transform_to_mocked_version()
    }

    class TSNode {
        <<internt>>
        -moves HashMap~String,usize~
        -score usize
        -best_move Option~String~
        -bit_state Vec~bool~
        -is_open bool
        -level u8
        -player String
        -score_level u8
        -score_address usize
    }

    class PlayerRandom {
        +name String
        +name_in_game String
    }

    class PlayerHuman {
        +name String
    }

    class BrainDiamond {
        +game_name String
        +filepath String
        +layers Vec~Cell~ x 6
        +evaluate_bit_state(bit_state) usize
        +get_gate_result_for_level(bit_state, level) Vec~bool~
        +do_mutate(max_address)
        +from_file() Result~LogicGatesError~
        +save_to_file() Result~LogicGatesError~
    }

    class Cell {
        +address_a usize[256]
        +address_b usize[256]
        +operator BitOp[256]
    }

    class BitOp {
        <<enum 16 logikkporter>>
        FALSE
        AND
        XOR
        OR
        NAND
        TRUE
        ... 10 til
    }

    class LogicGatesError {
        <<enum thiserror>>
        InvalidBitOperator
        Io
        InvalidChar
        General
    }

    class brain_funksjoner {
        <<modul-funksjoner>>
        get_default_cell_layer(layer) Vec~Cell~
        generate_random_cell_layer(max_address, layer) Vec~Cell~
        generate_random_brain(game_static) BrainDiamond
    }

    Agentish <|.. PlayerNNDiamond
    Agentish <|.. PlayerNNDiamondTS
    Agentish <|.. PlayerRandom
    Agentish <|.. PlayerHuman
    Brainy <|.. PlayerNNDiamond : MyBrain = BrainDiamond
    Brainy <|.. PlayerNNDiamondTS : MyBrain = BrainDiamond

    PlayerNNDiamond *-- BrainDiamond
    PlayerNNDiamondTS *-- BrainDiamond
    PlayerNNDiamondTS o-- TSNode : soketree over stillinger
    BrainDiamond *-- Cell : 6 lag a 256 porter
    Cell *-- BitOp : operator per port

    BrainDiamond ..> LogicGatesError : returnerer
    BrainDiamond ..> GameStatic : generate_random_brain
    Agentish ..> Playable : get_move tar generisk T som er Playable
    Agentish ..> GameStatic : get_ready
    Brainy ..> Game : evaluate_bit_state tar Game
    PlayerNNDiamond ..> TerminalState : leser terminalstatus
    PlayerNNDiamondTS ..> TerminalState : leser terminalstatus
    brain_funksjoner ..> Cell : bygger lag
    brain_funksjoner ..> BrainDiamond : returnerer
```

**Hjernens oppbygging:** `BrainDiamond` er et rent logikkport-nettverk med
`NO_LAYERS = 6` lag og `CELL_SIZE = 256` porter per celle. Hver `Cell` har to
inngangsadresser (`address_a`/`address_b` peker på bits i forrige lag) og en
`BitOp` per port. Nettverket serialiseres til tekst
(`data/<spill>-lgnndiamond-6-256.txt`, format `Layer:`/`Data:`/`adr_a,adr_b,OP`)
via `from_file`/`save_to_file`, og muteres med `do_mutate`.

**Tresøk:** `PlayerNNDiamondTS` bygger et tre av `TSNode` (velg blad,
utforsk, backpropager, legg til blad) og scorer bladene med
`BrainDiamond::evaluate_bit_state` / `get_gate_result_for_level`.

---

## 5. Objektmodell – binærer og kamp-logikk (`boardgame_binaries`)

`game_utils` er en delt modul (inkludert med `mod game_utils;` i hver binær)
med kamp-driveren og statistikk-objektene.

```mermaid
classDiagram
    class Statistics {
        -states Vec~Evaluate~
        +winner usize
        -rank_gates Vec~usize~
        +dump()
        +calculate()
        +dump_rank_gates()
    }

    class Evaluate {
        -move_number usize
        -move_string String
        -apprentice_score usize
        -master_score usize
        -gate_result Vec~bool~
        -game_bit_state Vec~bool~
    }

    class game_utils {
        <<funksjoner>>
        run_match(game generisk T, player_a, player_b) u8
        mass_run_match_with_random_player(game generisk T, player, tot_matches) f32
        run_match_return_statistics(game Game, master_player, apprentice_player) Statistics
    }

    Statistics *-- Evaluate : ett per trekk
    game_utils --> Statistics : returnerer
    game_utils --> Evaluate : fyller ut per trekk
    game_utils ..> Agentish : player_a / player_b
    game_utils ..> Brainy : master vs apprentice
    game_utils ..> Playable : game er generisk T som er Playable
    game_utils ..> PlayerRandom : motstander i mass_run
    game_utils ..> PlayerNNDiamond : apprentice i statistikk
    game_utils ..> BrainDiamond : gate_result per trekk
```

### Hvilken binær bruker hva

```mermaid
flowchart LR
    subgraph bins["[[bin]] i rust_boardgame_bin"]
        game_match["game_match<br/>main.rs"]
        plain["plain_diamond_match"]
        train["train_random"]
        evolve["train_random_evolve"]
        rank["rank_gates"]
        score["report_score"]
    end

    utils["game_utils<br/>run_match / mass_run / statistics"]

    c4["Connect4"]
    g_enum["Game::Connect4"]

    nn["PlayerNNDiamond"]
    ts["PlayerNNDiamondTS"]
    hum["PlayerHuman"]
    rnd["PlayerRandom"]
    brain["BrainDiamond<br/>generate_random_brain"]

    game_match --> utils
    game_match --> c4
    game_match --> ts
    game_match --> hum
    game_match -->|"run_match"| utils

    plain --> utils
    plain --> c4
    plain --> nn
    plain --> hum

    train --> utils
    train --> g_enum
    train --> nn
    train --> brain
    train -->|"run_match + mass_run"| utils

    evolve --> utils
    evolve --> g_enum
    evolve --> nn
    evolve -->|"run_match + mass_run,<br/>mutasjon"| utils

    rank --> utils
    rank --> g_enum
    rank --> ts
    rank --> nn
    rank -->|"run_match_return_statistics"| utils

    score --> utils
    score --> g_enum
    score --> ts
    score -->|"mass_run_match_with_random_player"| utils

    utils --> rnd
```

| Binær | Formål | Oppsett |
|-------|--------|---------|
| `game_match` | spille et parti mot menneske | `PlayerNNDiamondTS` vs `PlayerHuman` på `Connect4` |
| `plain_diamond_match` | parti uten tresøk | `PlayerNNDiamond` vs `PlayerHuman` på `Connect4` |
| `train_random` | tilfeldig trening av hjerne | to `PlayerNNDiamond` spiller mot hverandre, vinneren beholdes, måles mot `PlayerRandom` |
| `train_random_evolve` | trening med mutasjon | som over, men hjernen muteres mellom generasjoner |
| `rank_gates` | ranger portene i hjernen | `PlayerNNDiamondTS` (master) vs `PlayerNNDiamond` (apprentice) gir `Statistics::calculate` |
| `report_score` | mål styrke | `PlayerNNDiamondTS` vs `PlayerRandom` (1000 partier) |

---

## 6. Kjøreflyt for ett parti (`run_match`)

```mermaid
sequenceDiagram
    participant B as bin (f.eks. game_match)
    participant U as game_utils.run_match
    participant G as Game / Connect4
    participant A as player_a (Agentish)
    participant C as player_b (Agentish)

    B->>U: run_match(game, player_a, player_b)
    U->>G: get_game_static()
    loop til is_terminal()
        U->>G: get_active_player()
        U->>G: get_valid_moves()
        alt spiller A sin tur
            U->>A: get_move(valid_moves, player, game)
            A-->>U: cmove
        else spiller B sin tur
            U->>C: get_move(valid_moves, player, game)
            C-->>U: cmove
        end
        U->>G: play(cmove)
    end
    U->>G: get_winner()
    G-->>U: winner
    U-->>B: vinner-indeks (u8)
```

For `PlayerNNDiamondTS` skjer alt i `get_move`: den bygger/utvider et tre av
`TSNode`, scorer bladene med `BrainDiamond`, og returnerer beste trekk innen
`sec_to_move` sekunder.

---

## 7. Ordbok (domeneterminologi)

| Begrep | Objekt | Betydning |
|--------|--------|-----------|
| Spill | `Connect4`, `Nim12` | regler og brett-tilstand, implementerer `Playable` |
| Statisk spill-info | `GameStatic` | navn, spillernavn, antall bits i tilstanden |
| Agent / spiller | `Player*` | velger trekk, implementerer `Agentish` |
| Hjerne | `BrainDiamond` | logikkport-nettverk som scorer stillinger |
| Celle / port | `Cell`, `BitOp` | 256 porter med to inngangsadresser og en operator |
| Bit-tilstand | `Vec<bool>` | flatt stillingsformat, sett fra én spillers perspektiv |
| Trekk | `String` (f.eks. `"3"`) | felles format på tvers av spill |
| Master / apprentice | `Brainy`-spillere | master vurderer, apprentice spiller i `rank_gates` |
| Gate-rangering | `Statistics::rank_gates` | hvilke porter som minst bidrar til beste trekk |
| Tresøk | `TSNode`-tre i `PlayerNNDiamondTS` | utforsk/backpropager før trekkvalg |
