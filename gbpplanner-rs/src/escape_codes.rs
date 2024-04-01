#![allow(dead_code)]

// ANSI color codes
pub const GREEN: &'static str = "\x1b[32m";
pub const RED: &'static str = "\x1b[31m";
pub const BLUE: &'static str = "\x1b[34m";
pub const MAGENTA: &'static str = "\x1b[35m";
pub const CYAN: &'static str = "\x1b[36m";
pub const YELLOW: &'static str = "\x1b[33m";
pub const WHITE: &'static str = "\x1b[37m";

// ANSI text style codes
pub const BOLD: &'static str = "\x1b[1m";
pub const UNDERLINE: &'static str = "\x1b[4m";
pub const REVERSED: &'static str = "\x1b[7m";
pub const ITALIC: &'static str = "\x1b[3m";
pub const DIM: &'static str = "\x1b[2m";

pub const RESET: &'static str = "\x1b[0m";
