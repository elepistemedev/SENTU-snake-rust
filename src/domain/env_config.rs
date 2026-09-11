//! Environment configuration loader.
//! Reads `.env` from the project directory and provides type-safe accessors
//! with fallback to default constants.

use std::fs;
use std::path::Path;
use std::sync::OnceLock;

static INITIALIZED: OnceLock<()> = OnceLock::new();

/// Load environment variables from `.env` file into process environment if not already loaded.
pub fn init() {
    INITIALIZED.get_or_init(|| {
        load_from_path(".env");
    });
}

/// Load key-value pairs from a file at the given path.
pub fn load_from_path<P: AsRef<Path>>(path: P) {
    if let Ok(content) = fs::read_to_string(path) {
        parse_and_set_env(&content);
    }
}

/// Parse lines of `KEY=VALUE` and set them in `std::env`.
pub fn parse_and_set_env(content: &str) {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, val)) = trimmed.split_once('=') {
            let key = key.trim();
            let mut val = val.trim();
            // Remove surrounding quotes if present
            if (val.starts_with('"') && val.ends_with('"') && val.len() >= 2)
                || (val.starts_with('\'') && val.ends_with('\'') && val.len() >= 2)
            {
                val = &val[1..val.len() - 1];
            }
            if !key.is_empty() {
                std::env::set_var(key, val);
            }
        }
    }
}

pub fn env_usize(key: &str, default: usize) -> usize {
    init();
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

pub fn env_f64(key: &str, default: f64) -> f64 {
    init();
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

pub fn env_f32(key: &str, default: f32) -> f32 {
    init();
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

pub fn env_u64(key: &str, default: u64) -> u64 {
    init();
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_set_env_handles_comments_quotes_and_whitespace() {
        let content = "
            # Comment line
            TEST_ENV_USIZE = 42
            TEST_ENV_F64=\"3.1415\"
            TEST_ENV_F32 = '0.5'
            # Another comment
            TEST_ENV_U64=100
        ";
        parse_and_set_env(content);

        assert_eq!(env_usize("TEST_ENV_USIZE", 0), 42);
        assert_eq!(env_f64("TEST_ENV_F64", 0.0), 3.1415);
        assert_eq!(env_f32("TEST_ENV_F32", 0.0), 0.5);
        assert_eq!(env_u64("TEST_ENV_U64", 0), 100);
    }

    #[test]
    fn env_helpers_fallback_to_default_on_missing_or_malformed() {
        assert_eq!(env_usize("NONEXISTENT_KEY_XYZ_123", 999), 999);
        assert_eq!(env_f64("NONEXISTENT_KEY_XYZ_123", 1.23), 1.23);
        assert_eq!(env_f32("NONEXISTENT_KEY_XYZ_123", 4.56), 4.56);
        assert_eq!(env_u64("NONEXISTENT_KEY_XYZ_123", 789), 789);

        // Malformed value fallback
        std::env::set_var("TEST_MALFORMED_NUM", "not_a_number");
        assert_eq!(env_usize("TEST_MALFORMED_NUM", 50), 50);
        assert_eq!(env_f64("TEST_MALFORMED_NUM", 0.5), 0.5);
    }

    #[test]
    fn load_from_path_reads_temporary_file_properly() {
        let temp_file = "test_temp.env";
        std::fs::write(temp_file, "TEMP_TEST_VAR=2048\n# comment\nTEMP_FLOAT=0.125\n").unwrap();
        load_from_path(temp_file);
        std::fs::remove_file(temp_file).ok();

        assert_eq!(env_usize("TEMP_TEST_VAR", 0), 2048);
        assert_eq!(env_f64("TEMP_FLOAT", 0.0), 0.125);
    }
}

