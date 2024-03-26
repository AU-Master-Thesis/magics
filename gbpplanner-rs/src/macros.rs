#![deny(missing_docs)]
//! Declarative macros used in more than one file throughout the crate.

/// Creates a "boolean bevy Resource" i.e. a Resource(bool), together with a handful
/// of methods to work with a boolean resource.
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
