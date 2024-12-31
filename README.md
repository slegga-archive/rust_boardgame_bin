# rust_boardgame

Del at serie på 3 git repoer som hører sammen.
Gjort det slik for ellers blir det masse warnings

De tre repoene er:
rust_boardgame_bin -  binaries
rust_boardgame_player - player
rust_boardgame_game - game


Disse Rust repo er for å forske ut logisk port nettverk på GGP spill.



HVORDAN STARTE
==============
RUST_LOG=warn cargo run --bin report_score
RUST_LOG=warn cargo run --bin train_random
RUST_LOG=debug RUST_BACKTRACE=1 cargo run --bin game_match

1-2 NIM
=======
Starter med "1-2 Nim" som er en veldig enkelt tall spill hvor man starter på tallet 20.
Det er to spillere. De kan velge å fjerne 1 eller 2 fra det gjennstående tallet.
Den som ikke kan gjøre et trekk, taper.

Neste få til tilfeldig trening. Dvs generere opp en tilfeldig hjerne og la den spille mot den gjeldende hjernen.
Hvis utfordrer vinner bytt ut gammel hjerne med ny.
