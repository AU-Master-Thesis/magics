//! Main entry point for the Magics CLI
//!
//! This binary provides a unified interface for running simulations
//! in both headless mode and with UI visualization.

use magics_cli::prelude::*;
use std::env;
use anyhow::Result;
use log::{info, error};

fn main() -> Result<()> {
    // Parse command-line arguments
    let args = parse_arguments();
    
    // Set up logging based on verbosity
    setup_logging(&args);
    
    // Handle metadata display if requested
    if args.metadata {
        print_metadata();
        return Ok(());
    }
    
    // Set working directory if specified
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(dir) = &args.working_dir {
        std::env::set_current_dir(dir)
            .map_err(|e| anyhow::anyhow!("Failed to set working directory: {}", e))?;
        info!("Changed working directory to: {}", dir.display());
    }
    
    // Check if we're running in headless mode
    if args.is_headless() {
        // Set environment variable for other components to detect
        env::set_var("MAGICS_HEADLESS", "1");
        
        // Run in headless mode
        run_headless(&args)
    } else {
        // Clear environment variable if set
        env::remove_var("MAGICS_HEADLESS");
        
        // Run with UI if available
        #[cfg(feature = "ui")]
        {
            run_with_ui(&args)
        }
        
        // If UI feature is not enabled, warn and run in headless mode
        #[cfg(not(feature = "ui"))]
        {
            error!("UI mode requested but not available. Building with '--no-default-features' disables the UI.");
            println!("UI mode not available in this build. Running in headless mode instead.");
            run_headless(&args)
        }
    }
}
