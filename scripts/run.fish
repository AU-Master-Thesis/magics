#!/usr/bin/env -S fish --no-config

clear

function run
    echo $argv | fish_indent --ansi
    eval $argv
end

set -l jobs (math (nproc) - 1) # leave one CPU core for interactivity
set -lx RUST_LOG error, magics=info
cargo run --jobs=$jobs --bin magics -- $argv
