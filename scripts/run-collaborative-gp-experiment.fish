#!/usr/bin/env nix-shell
#! nix-shell -i fish -p jq bat sd yq-go

argparse f/force -- $argv; or exit 2

function run
    echo "$argv" | fish_indent --ansi
    eval "$argv"
end

set -l seeds 0 31 227 252 805

# --- Low Qin ---

set -l experiment "Collaborative GP"
set -l config_file "config/scenarios/$experiment/config.toml"
set -l formation_file "config/scenarios/$experiment/formation.yaml"

if not test -f $config_file
    printf '%serror%s: %s does not exist!\n' (set_color red) (set_color normal) $config_file >&2
    exit 1
end

if not test -f $formation_file
    printf '%serror%s: %s does not exist!\n' (set_color red) (set_color normal) $formation_file >&2
    exit 1
end

set -l t_start (date "+%s")

printf '%sinfo%s: starting experiment\n' (set_color green) (set_color normal) >&2

for seed in $seeds
    sed --regexp-extended "s/prng-seed\s*=\s*([0-9]+)/prng-seed = $seed/" -i $config_file
    for tracking in false true
        sed --regexp-extended "s/tracking\s*=\s*(.*)/tracking = $tracking/" -i $config_file
        set -l sigmas
        if test $tracking = true
            set sigmas 0.15 0.5
        else
            set sigmas 0.15
        end

        for sigma_tracking in $sigmas
            sed --regexp-extended "s/sigma-factor-tracking\s*=\s*(.*)/sigma-factor-tracking = $sigma_tracking/" -i $config_file
            printf '%sinfo%s: testing seed: %d, tracking: %s, sigma-tracking: %s\n' (set_color green) (set_color normal) $seed $tracking $sigma_tracking >&2

            set -l output_file experiments/collaborative-gp/tracking-$tracking-sigma-tracking-$sigma_tracking-seed-$seed.json

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
            set -l exported_json (printf '%s\n' export_collaborative\ gp*.json | tail -n 1)
            set -l dirname (path dirname "$output_file")
            command mkdir -p "$dirname"
            mv "$exported_json" "$output_file"
        end
    end
end
