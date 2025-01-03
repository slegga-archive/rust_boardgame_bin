#!/bin/bash
DIR=`pwd`
cd ~/git/rust_boardgame_bin
cargo build
cd ../rust_boardgame_player
cargo biuld
cd ../rust_boardgame
cargo build
cd $DIR