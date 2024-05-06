#!/usr/bin/env -S fish --no-config

set -l reset (set_color normal)
set -l bold (set_color --bold)
set -l italics (set_color --italics)
set -l red (set_color red)
set -l green (set_color green)
set -l yellow (set_color yellow)
set -l blue (set_color blue)
set -l cyan (set_color cyan)
set -l magenta (set_color magenta)

set -l deps cargo llvm-profdata

for d in $deps
    if not command -q $d
        printf '%serror%s: program %s not installed\n' (set_color red) (set_color normal) $d >&2
        exit 1
    end
end

# https://doc.rust-lang.org/rustc/profile-guided-optimization.html
