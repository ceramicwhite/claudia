//! Test modules for Claudia backend

// Common test utilities
mod test_utils;

// Existing test modules
#[cfg(test)]
mod sandbox;
mod sandbox_tests;

// New test modules for agents and scheduler
#[cfg(test)]
mod agents_tests;
#[cfg(test)]
mod scheduler_tests;
#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod edge_cases_tests;