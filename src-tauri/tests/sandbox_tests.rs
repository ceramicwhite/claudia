//! Main entry point for sandbox tests
//!
//! This file integrates all the sandbox test modules and provides
//! a central location for running the comprehensive test suite.
#![allow(dead_code)]

#[cfg(unix)]
#[path = "sandbox/mod.rs"]
mod sandbox;

// Re-export test modules to make them discoverable
#[cfg(unix)]
pub use sandbox::*;
