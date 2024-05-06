#!/usr/bin/env -S fish --no-config

clear

function run
    echo $argv | fish_indent --ansi
    eval $argv
end

set -l jobs (math (nproc) - 1) # leave one CPU core for interactivity
set -lx RUST_LOG error, gbpplanner_rs=info
cargo run --jobs=$jobs --bin gbpplanner-rs -- $argv
