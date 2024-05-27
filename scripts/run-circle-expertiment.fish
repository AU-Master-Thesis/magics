#!/usr/bin/env nix-shell
#! nix-shell -i fish -p jq

argparse f/force -- $argv; or exit 2

function run
    echo $argv | fish_indent --ansi
    eval $argv
end

set -l config_file config/simulations/Circle\ Experiment/config.toml
set -l formation_file config/simulations/Circle\ Experiment/formation.yaml

if not test -f $config_file
    printf '%serror%s: %s does not exist!\n' (set_color red) (set_color normal) $config_file >&2
    exit 1
end

printf '%sinfo%s: starting experiment\n' (set_color green) (set_color normal) >&2

for seed in 0 31 227 252 805
    sed --regexp-extended "s/prng-seed\s*=\s*([0-9]+)/prng-seed = $seed/" -i $config_file
    for num_robots in (seq 5 5 50)
        sed --regexp-extended "s/robots:\s+(\d+)/robots: $num_robots/" -i $formation_file
        set -l output_file experiments/communications-failure/num-robots-$num_robots-seed-$seed.json

        if test -f $output_file
            if not set -q _flag_force
                printf '%swarn%s: %s already exists, use -f or --force to overwrite\n' (set_color yellow) (set_color normal) $output_file >&2
                continue
            else
                printf '%sinfo%s: overwriting %s\n' (set_color green) (set_color normal) $output_file >&2
            end
        end

        RUST_LOG=gbpplanner_rs=error ./target/release/gbpplanner-rs -i 'Circle Experiment' 2>/dev/null
        set -l exported_json (printf '%s\n' export_circle\ experiment*.json | tail -n 1)
        command mkdir -p (path basename "$output_file")
        run mv "$exported_json" "$output_file"
    end
end
