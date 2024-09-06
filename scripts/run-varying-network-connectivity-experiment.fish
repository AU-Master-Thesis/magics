#!/usr/bin/env nix-shell
#! nix-shell -i fish -p jq

argparse f/force -- $argv; or exit 2

set -l reset (set_color normal)
set -l bold (set_color --bold)
set -l italics (set_color --italics)
set -l red (set_color red)
set -l green (set_color green)
set -l yellow (set_color yellow)
set -l blue (set_color blue)
set -l cyan (set_color cyan)
set -l magenta (set_color magenta)

#'Varying Network Connectivity Experiment'

set -l config_file config/scenarios/Varying\ Network\ Connectivity\ Experiment/config.toml
set -l formation_file config/scenarios/Varying\ Network\ Connectivity\ Experiment/formation.yaml

if not test -f $config_file
    printf '%serror%s: %s does not exist!\n' $red $reset $config_file >&2
    exit 1
end
if not test -f $formation_file
    printf '%serror%s: %s does not exist!\n' $red $reset $formation_file >&2
    exit 1
end

printf '%sinfo%s: starting experiment\n' $green $reset >&2

set -l t_start (date "+%s")

for seed in 0 31 227 252 805

    sed --regexp-extended "s/prng-seed\s*=\s*([0-9]+)/prng-seed = $seed/" -i $config_file
    printf '%sinfo%s: changed prng-seed to: %d\n' $green $reset $seed >&2

    for comms_radius in 20 40 60 80
        # sed "s/^radius\s*=\s*([0-9]\+)/radius = $comms_radius/" -i $config_file
        sed --regexp-extended "s/^radius\s*=\s*[0-9]+/radius = $comms_radius/" -i $config_file

        printf '%sinfo%s: changed comms-radius to: %d\n' $green $reset $comms_radius >&2

        set -l output_file experiments/varying-network-connectivity-lm-3-tk-13.33/comms-radius-$comms_radius-seed-$seed.json

        set -l t_end (date "+%s")
        set -l t_diff (math "$t_end - $t_start")
        if functions -q peopletime
            printf '%sinfo%s: time elapsed: %s\n' $green $reset (peopletime (math "$t_diff * 1000")) >&2
        end

        if test -f $output_file
            if not set -q _flag_force
                printf '%swarn%s: %s already exists, use -f or --force to overwrite\n' $yellow $reset $output_file >&2
                continue
            else
                printf '%sinfo%s: overwriting %s\n' $green $reset $output_file >&2
            end
        end

        RUST_LOG=magics=error ./target/release/magics -i 'Varying Network Connectivity Experiment' 2>/dev/null
        set -l exported_json (printf '%s\n' export_varying\ network\ connectivity\ experiment*.json | tail -n 1)
        set -l dirname (path dirname "$output_file")
        command mkdir -p "$dirname"
        mv "$exported_json" "$output_file"
    end
end

# exit 0
