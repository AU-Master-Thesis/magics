#!/usr/bin/env nix-shell
#! nix-shell -i fish -p jq sd

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

function run
    echo $argv | fish_indent --ansi
    eval $argv
end

set -l scenario 'Collaborative Complex'

set -l config_file config/simulations/$scenario/config.toml

if not test -f $config_file
    printf '%serror%s: %s does not exist!\n' $red $reset $config_file >&2
    exit 1
end

set -l t_start (date "+%s")

printf '%sinfo%s: starting experiment\n' $green $reset >&2

for tracking in true false
    # flip tracking on or off
    # sed --regexp-extended "s/^tracking\s*=\s*(.*)/tracking = $tracking/" -i $config_file
    sd '^tracking\s*=\s*(true|false)' "tracking = $tracking" $config_file

    set -l output_file experiments/collaborative-complex/tracking-$tracking.json

    if test -f $output_file
        if not set -q _flag_force
            printf '%swarn%s: %s already exists, use -f or --force to overwrite\n' $yellow $reset $output_file >&2
            continue
        else
            printf '%sinfo%s: overwriting %s\n' $green $reset $output_file >&2
        end
    end

    RUST_LOG=magics=error ./target/release/magics -i $scenario 2>/dev/null
    # set -l exported_json (printf '%s\n' export_communications\ failure\ experiment*.json | tail -n 1)
    set -l exported_json (printf '%s\n' export_collaborative\ complex*.json | tail -n 1)
    set -l dirname (path dirname "$output_file")
    command mkdir -p "$dirname"
    mv "$exported_json" "$output_file"

    set -l t_end (date "+%s")
    set -l t_diff (math "$t_end - $t_start")
    if functions -q peopletime
        printf '%sinfo%s: time elapsed: \n' $green $reset (peopletime (math "$t_diff * 1000")) >&2
    end
end
