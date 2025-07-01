#![allow(dead_code)]

// Pricing constants (per million tokens)
pub const OPUS_3_INPUT_PRICE: f64 = 15.0;
pub const OPUS_3_OUTPUT_PRICE: f64 = 75.0;
pub const OPUS_3_CACHE_WRITE_PRICE: f64 = 18.75;
pub const OPUS_3_CACHE_READ_PRICE: f64 = 1.50;

pub const SONNET_3_INPUT_PRICE: f64 = 3.0;
pub const SONNET_3_OUTPUT_PRICE: f64 = 15.0;
pub const SONNET_3_CACHE_WRITE_PRICE: f64 = 3.75;
pub const SONNET_3_CACHE_READ_PRICE: f64 = 0.30;

pub const OPUS_4_INPUT_PRICE: f64 = 15.0;
pub const OPUS_4_OUTPUT_PRICE: f64 = 75.0;
pub const OPUS_4_CACHE_WRITE_PRICE: f64 = 18.75;
pub const OPUS_4_CACHE_READ_PRICE: f64 = 1.50;

pub const SONNET_4_INPUT_PRICE: f64 = 3.0;
pub const SONNET_4_OUTPUT_PRICE: f64 = 15.0;
pub const SONNET_4_CACHE_WRITE_PRICE: f64 = 3.75;
pub const SONNET_4_CACHE_READ_PRICE: f64 = 0.30;

// Token calculation constant
pub const TOKENS_PER_MILLION: f64 = 1_000_000.0;

// Default values
pub const DEFAULT_MODEL: &str = "sonnet";
pub const DEFAULT_SANDBOX_ENABLED: bool = true;
pub const DEFAULT_FILE_READ_ENABLED: bool = true;
pub const DEFAULT_FILE_WRITE_ENABLED: bool = true;
pub const DEFAULT_NETWORK_ENABLED: bool = false;

// Process constants
pub const PROCESS_KILL_TIMEOUT_MS: u64 = 5000;
pub const OUTPUT_READ_BUFFER_SIZE: usize = 8192;

// Scheduling constants
pub const SCHEDULE_CHECK_INTERVAL_MS: u64 = 60000; // 1 minute

// API constants
pub const CLAUDE_USAGE_LIMIT_PATH: &str = "https://api.anthropic.com/dashboard/settings/limits";

// Export version
pub const AGENT_EXPORT_VERSION: u32 = 1;