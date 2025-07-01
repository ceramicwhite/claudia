//! Common test utilities

/// Check if sandboxing is supported on the current platform
pub fn is_sandboxing_supported() -> bool {
    matches!(std::env::consts::OS, "linux" | "macos" | "freebsd")
}