DESEMBER 2024
=============
Fryser ned prosjektet. Må bruke det neste halve året på lære den nye jobben. Ikke tid til dette lenger.

Status nå er at AI'en ikke er spesielt god. Selv med tree Search. Er vel mulig å bruke eksisterende for å
veilede hva som er riktig move.

Problemet nå er at AI'en ikke kjenner igjen en dårlig posisjon. Det bør fikses av fremgangsmåten beskrevet lengre ned. Se "TRENINGSTRATEGI: VEILEDER FOR DIAMOND" i readme til rust_boardgame_player. Den går i korthet ut på å tilpasse AI til Tree Search AI og diffe vurderingene og vekte resultatet og slik identifisere hvilke gates/porter i siste lag som bidrar minst/saboterer.

Akkurat nå tenker jeg siste forslag er det beste.

TODO
====
[ ] Lag et statisikk program for å finne snitt avvik på score, gjennomsnitt evaluering, fordeling av gatetyper, statistikk for armadresser i førstelaget osv.
[ ] begrenset random evolve script: utfra statistikken finn dårligste gate og rull den og bakendforliggende gates på nytt (inkl første lag med adresser)
[*] rust_boardgame_player/src/player/plgnn_diamond_tree_search.rs: select_leaf:
    Først ta vare på  adressen til beste posisjon/best_move.
    velg best_move sin adresse for å explore. Altså returner den adressen hvis den er på level 9 eller mindre.
[*] Rydd i warnings.
    - [*] Tror jeg må splitte lib og bin for å bli kvitt warnings
[*] Game pretty_print viser siste move i gult.
[*] TODO vurder å fjerne pretty_print og legge inn farge på Display (NEI display holder på fargen. Blir ikke bra.)
[*] tree search player
    - [X] Lag explore for alle get_valid_moves og velg den med best state_value av de med mest poeng
    - [X] Lag state trestruktur
    - [X] Lag tree search logikk. Prøv å få det modulært.
    - [-] (for mye jobb) Kopier over MTS fra git/ggp-rs/player og over i src/player/ og bruk som grunnlag.
[*] Fått til mini dimond 2-8 slik at det er lettere å analysere.
[*] Lag diamant LGNN som ender med 256 bits. Tenker 6 lag. lag 1 blir størrelse 16000 stor
[*] Bare kaos med bit_state på Connect4. Skriv masse tester
[*] Flytt ut Nuralnetwork ut fra player/mod.rs
[*] Innfør 16(14) gates istedenfor dagens 6.
[*] Gjør noe målinger.
[*] Oppdaget feil og fikset. Prøv med utskrift av brett: game after move: Turn: Yellow - [[0, 0, 0, 0, 0, 0], [1, 0, 0, 0, 0, 0], [2, 2, 1, 2, 2, 2], [1, 2, 1, 1, 1, 2], [2, 1, 2, 1, 1, 1], [0, 0, 0, 0, 0, 0], [2, 2, 2, 1, 1, 0]]
[*] Skriv ut brett
[*] Prøv ut game som enum.
[*] Rydd i koden. Fjern panic og returner en Error istedenfor.(Beholder noen panic. Er glad i panic spesielt der jeg ikke forventer at de slår til.)
[*] Rydd i koden. Fjern unwrap,expect i modulene game,players,connect4,logic_gates
[*] Lag et script som måler hvor god eksisterende hjerne er i spillet. Kjører 500 spill som start spiller og 500 som slutt spiller, og ser på hvor mange seier hver får. Rapporterer resultatet i 00.0% format. Kjører i evig loop. Sjekker om hjernefilen har endret seg.
[*] Begynn på perserpsjon av seier situasjon/ tap situasjon
[*] Endre bit_state til å se fra perspektivet til en av spillere. Dvs mine og den andre istedenfor rød og gul.
