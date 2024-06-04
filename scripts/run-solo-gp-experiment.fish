#!/usr/bin/env nix-shell
#! nix-shell -i fish -p jq bat sd

argparse f/force -- $argv; or exit 2

function run
    echo $argv | fish_indent --ansi
    eval $argv
end

set -l experiment "Solo GP"
set -l config_file "config/simulations/$experiment/config.toml"

if not test -f $config_file
    printf '%serror%s: %s does not exist!\n' (set_color red) (set_color normal) $config_file >&2
    exit 1
end

set -l t_start (date "+%s")

printf '%sinfo%s: starting experiment\n' (set_color green) (set_color normal) >&2

for seed in 0 31 227 252 805
    sed --regexp-extended "s/prng-seed\s*=\s*([0-9]+)/prng-seed = $seed/" -i $config_file
    for tracking in false true
        sed --regexp-extended "s/^tracking\s*=\s*(.*)/tracking = $tracking/" -i $config_file

        printf '%sinfo%s: testing seed: %d, tracking: %s\n' (set_color green) (set_color normal) $seed $tracking >&2

        set -l output_file experiments/solo-gp/tracking-$tracking-seed-$seed.json

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

        RUST_LOG=magics=error ./target/release/magics -i "$experiment" 2>/dev/null
        set -l exported_json (printf '%s\n' export_solo\ gp*.json | tail -n 1)
        set -l dirname (path dirname "$output_file")
        command mkdir -p "$dirname"
        echo mv "$exported_json" "$output_file"
        mv "$exported_json" "$output_file"
    end
end
