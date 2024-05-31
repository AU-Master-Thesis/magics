#!/usr/bin/env nix-shell
#! nix-shell -i fish -p jq

argparse f/force -- $argv; or exit 2

function run
    echo $argv | fish_indent --ansi
    eval $argv
end

set -l config_file config/simulations/Communications\ Failure\ Experiment/config.toml

if not test -f $config_file
    printf '%serror%s: %s does not exist!\n' (set_color red) (set_color normal) $config_file >&2
    exit 1
end

set -l t_start (date "+%s")

printf '%sinfo%s: starting experiment\n' (set_color green) (set_color normal) >&2

for seed in 0 31 227 252 805
    sed --regexp-extended "s/prng-seed\s*=\s*([0-9]+)/prng-seed = $seed/" -i $config_file

    for v0 in 10 15
        sed --regexp-extended "s/max-speed\s*=\s*([0-9]+)/max-speed = $v0/" -i $config_file

        seq 0.0 0.1 0.7 | string replace ',' '.' | while read failure_probability
            sed --regexp-extended "s/failure-rate\s*=\s*(.*)/failure-rate = $failure_probability/" -i $config_file
            printf '%sinfo%s: seed=%d v0=%d failure-probability=%s\n' (set_color green) (set_color normal) $seed $v0 $failure_probability >&2
            set -l output_file experiments/communications-failure/v0-$v0-failure-$failure_probability-seed-$seed.json

            set -l t_end (date "+%s")
            set -l t_diff (math "$t_end - $t_start")
            if functions -q peopletime
                printf '%sinfo%s: time elapsed: \n' $green $reset (peopletime (math "$t_diff * 1000")) >&2
            end

            if test -f $output_file
                if not set -q _flag_force
                    printf '%swarn%s: %s already exists, use -f or --force to overwrite\n' (set_color yellow) (set_color normal) $output_file >&2
                    continue
                else
                    printf '%sinfo%s: overwriting %s\n' (set_color green) (set_color normal) $output_file >&2
                end
            end

            RUST_LOG=gbpplanner_rs=error ./target/release/gbpplanner-rs -i 'Communications Failure Experiment' 2>/dev/null
            set -l exported_json (printf '%s\n' export_communications\ failure\ experiment*.json | tail -n 1)
            set -l dirname (path dirname "$output_file")
            command mkdir -p "$dirname"
            mv "$exported_json" "$output_file"
        end
    end
end
