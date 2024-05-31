#!/usr/bin/env nix-shell
#! nix-shell -i fish -p jq

argparse f/force -- $argv; or exit 2

function run
    echo $argv | fish_indent --ansi
    eval $argv
end

set -l config_file config/simulations/Iteration\ Amount\ Experiment/config.toml

if not test -f $config_file
    printf '%serror%s: %s does not exist!\n' (set_color red) (set_color normal) $config_file >&2
    exit 1
end

set -l t_start (date "+%s")

printf '%sinfo%s: starting experiment\n' (set_color green) (set_color normal) >&2

for seed in 0
    sed --regexp-extended "s/prng-seed\s*=\s*([0-9]+)/prng-seed = $seed/" -i $config_file
    for internal in (seq 5 5 30)
        sed --regexp-extended "s/internal\s*=\s*(.*)/internal = $internal/" -i $config_file
        for external in (seq 5 5 30)
            sed --regexp-extended "s/external\s*=\s*(.*)/external = $external/" -i $config_file

            printf '%sinfo%s: testing seed: %d, internal: %s, external: %d\n' (set_color green) (set_color normal) $seed $internal $external >&2

            set -l output_file experiments/iteration-amount/internal-$internal-external-$external-seed-$seed.json

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

            RUST_LOG=gbpplanner_rs=error ./target/release/gbpplanner-rs -i 'Iteration Amount Experiment' 2>/dev/null
            set -l exported_json (printf '%s\n' export_iteration\ amount\ experiment*.json | tail -n 1)
            set -l dirname (path dirname "$output_file")
            command mkdir -p "$dirname"
            mv "$exported_json" "$output_file"
        end
    end
end
