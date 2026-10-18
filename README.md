# rust_boardgame

Del at serie på 3 git repoer som hører sammen.
Gjort det slik for ellers blir det masse warnings

De tre repoene er:
rust_boardgame_bin -  binaries
rust_boardgame_player - player
rust_boardgame_game - game

De bygges som ett workspace fra meta-repoets rot.

Disse Rust repo er for å forske ut logisk port nettverk på GGP spill.



HVORDAN STARTE
==============
Kjør fra meta-repoets rot (workspace):
RUST_LOG=warn cargo run -p boardgame_binaries --bin cli -- report
RUST_LOG=warn cargo run -p boardgame_binaries --bin cli -- train
RUST_LOG=debug RUST_BACKTRACE=1 cargo run -p boardgame_binaries --bin game_match
cargo run -p boardgame_binaries --bin cli -- tui                 # du starter (Red), 3 sek tenketid
cargo run -p boardgame_binaries --bin cli -- tui 10              # 10 sek tenketid
cargo run -p boardgame_binaries --bin cli -- tui connect4 5 --ai-first  # AI starter (Red), 5 sek
cargo run -p boardgame_game                                       # human-vs-human TUI uten AI

cli-subkommandoer: train | evolve | rank | report | match | bench | tui

CONNECT4 TUI
============
`cli tui [GAME] [SEC] [--ai-first]` starter Connect4 i terminalen mot
Diamond Tree Search AI. GAME er foreløpig bare connect4 (default).
SEC må være én av 1, 3, 5, 10 (default 3) og er AI-ens tenketid per trekk.
Uten --ai-first starter du (Red); med den starter AI-en (Red).

Når TUI-en åpner får du en setup-skjerm der CLI-verdiene er forhåndsvalg
og kan overstyres: Up/Down velger rad (Starter: Du/AI, Tenketid: 1/3/5/10),
Left/Right endrer verdi, Enter starter spillet, q/Esc avslutter.
I selve spillet: 1-7 eller Left/Right+Enter spiller, r starter på nytt.

Arkitektur: all Connect4-spesifikk kode ligger i rust_boardgame_game
(board_lines, spill-løkke, setup-skjerm i src/connect4_tui.rs).
Løkken er generisk (`run_tui_for` + `TuiDesc`), AI-en plugges inn via
`AiOpponent`-closure fra bin, så game-crate avhenger aldri av player-crate.
Nytt spill legges til med: Playable+Display-impl, ny GameKind-variant med
egen TuiDesc, og ett navn i GAME-matchingen i cli.rs.

1-2 NIM
=======
Starter med "1-2 Nim" som er en veldig enkelt tall spill hvor man starter på tallet 20.
Det er to spillere. De kan velge å fjerne 1 eller 2 fra det gjennstående tallet.
Den som ikke kan gjøre et trekk, taper.

Neste få til tilfeldig trening. Dvs generere opp en tilfeldig hjerne og la den spille mot den gjeldende hjernen.
Hvis utfordrer vinner bytt ut gammel hjerne med ny.

Connect4
========
Fire på rad. AI virker ikke så veldig bra per desember 2024.