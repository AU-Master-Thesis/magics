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

# echo madvise > /sys/kernel/mm/transparent_hugepage/enabled

# read thp </sys/kernel/mm/transparent_hugepage/enabled

set -l thp (string match --regex --groups-only '\[(\w+)\]' < /sys/kernel/mm/transparent_hugepage/enabled)

# if test $thp = never
#     printf ''
#     exit 1
# end

switch $thp
    case always madvise
        set -x MALLOC_CONF "thp:always,metadata_thp:always"
    case never
        printf '%swarn%s: transparent_hugepage not enabled\n' (set_color yellow) (set_color normal) >&2
end

# https://kobzol.github.io/rust/rustc/2023/10/21/make-rust-compiler-5percent-faster.html

command cargo build --profile=release
