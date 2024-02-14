# threads := num_cpus() - 1

alias b := build
alias r := run

default:
    @just --list

build:
    cargo build 

run:
    cargo run

# TODO: use mold here
dev:
    cargo run --features bevy/dynamic_linking
