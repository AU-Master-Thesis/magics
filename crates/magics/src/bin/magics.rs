//! Main entry point for the Magics application
//!
//! This binary provides the full application with UI and visualization,
//! but can also run in headless mode if requested.

use magics::prelude::*;

fn main() -> anyhow::Result<()> {
    // Re-export the main binary from the crate
    // This will run the full application with UI by default
    magics::main()
}
