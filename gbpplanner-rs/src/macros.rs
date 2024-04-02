#![deny(missing_docs)]
//! Declarative macros used in more than one file throughout the crate.

/// Creates a "boolean bevy Resource" i.e. a Resource(bool), together with a
/// handful of methods to work with a boolean resource.
/// # Example
/// ```rust
/// boolean_bevy_resource(ManualMode, default = false);
/// ```
/// expands to:
/// ```rust
/// #[derive(bevy::ecs::system::Resource, Debug, PartialEq, Eq, Deref)]
/// pub struct ManualMode(bool);
///
/// #[allow(dead_code)]
/// impl ManualMode {
///     #[inline]
///     pub fn enabled(res: Res<ManualMode>) -> bool {
///         res.0
///     }
///
///     #[inline]
///     pub fn disabled(res: Res<ManualMode>) -> bool {
///         !res.0
///     }
///
///     #[inline]
///     pub fn toggle(&mut self) {
///         self.0 = !self.0;
///     }
///
///     #[inline]
///     pub fn enable(&mut self) {
///         self.0 = true;
///     }
///
///     #[inline]
///     pub fn disable(&mut self) {
///         self.0 = false;
///     }
///
///     #[inline]
///     pub fn set(&mut self, value: bool) {
///         self.0 = value;
///     }
/// }
///
/// impl Default for ManualMode {
///     fn default() -> ManualMode {
///         ManualMode(false)
///     }
/// }
/// ```
#[macro_export]
macro_rules! boolean_bevy_resource {
    ($name:ident, default = $default:expr) => {
        #[derive(bevy::ecs::system::Resource, Debug, PartialEq, Eq, Deref)]
        pub struct $name(bool);

        #[allow(dead_code)]
        impl $name {
            #[inline]
            pub fn enabled(res: Res<$name>) -> bool {
                res.0
            }

            #[inline]
            pub fn disabled(res: Res<$name>) -> bool {
                !res.0
            }

            #[inline]
            pub fn toggle(&mut self) {
                self.0 = !self.0;
            }

            #[inline]
            pub fn enable(&mut self) {
                self.0 = true;
            }

            #[inline]
            pub fn disable(&mut self) {
                self.0 = false;
            }

            #[inline]
            pub fn set(&mut self, value: bool) {
                self.0 = value;
            }
        }

        impl Default for $name {
            fn default() -> $name {
                $name($default)
            }
        }
    };
}

/// Pretty prints a message to the console.
/// Takes either a [`FactorId]` and a [`VariableId`], as 'from' and 'to' of the
/// message OR
/// Takes a [`VariableId`] and a [`FactorId`], as 'from' and 'to' of the message
#[macro_export]
macro_rules! pretty_print_message {
    ($from:expr, $to:expr, $post:expr) => {
        println!(
            "{}:{} │ {}{}{} -> {}{}{} │ {}",
            file!().split("/").last().unwrap(),
            line!(),
            $from.color,
            $from.global_id(),
            crate::escape_codes::RESET,
            $to.color,
            $to.global_id(),
            crate::escape_codes::RESET,
            $post
        );
    };
}

/// Pretty print title
/// # Example output
/// ```sh
/// ══════════════════ Title ══════════════════
/// ```
#[macro_export]
macro_rules! pretty_print_title {
    ($title:expr) => {{
        let columns = termsize::get()
            .map(|ts| if ts.cols == 0 { 80 } else { ts.cols })
            .unwrap_or(80) as usize;

        let title_len = $title.len();

        let left_padding = (columns - title_len - 2) / 2;
        let right_padding = columns - title_len - 2 - left_padding;

        println!(
            "{}{} {} {}{}",
            crate::escape_codes::BOLD,
            "═".repeat(left_padding),
            $title,
            "═".repeat(right_padding),
            crate::escape_codes::RESET,
        );
    }};
}

/// Pretty print subtitle
/// Uses
/// # Example output
/// ```sh
/// ────────────────── subtitle ──────────────────
/// ```
#[macro_export]
macro_rules! pretty_print_subtitle {
    ($subtitle:expr) => {{
        let columns = termsize::get()
            .map(|ts| if ts.cols == 0 { 80 } else { ts.cols })
            .unwrap_or(80) as usize;

        let subtitle_len = $subtitle.len();

        let left_padding = (columns - subtitle_len - 2) / 2;
        let right_padding = columns - subtitle_len - 2 - left_padding;

        println!(
            "{}{} {} {}{}",
            crate::escape_codes::BOLD,
            "─".repeat(left_padding),
            $subtitle,
            "─".repeat(right_padding),
            crate::escape_codes::RESET,
        );
    }};
}

/// Pretty print a line
/// # Example output
/// ```sh
/// ──────────────────────────────────────────────
/// ```
#[macro_export]
macro_rules! pretty_print_line {
    () => {{
        let columns = termsize::get()
            .map(|ts| if ts.cols == 0 { 80 } else { ts.cols })
            .unwrap_or(80) as usize;

        println!(
            "{}{}{}",
            crate::escape_codes::BOLD,
            "─".repeat(columns),
            crate::escape_codes::RESET,
        );
    }};
}
