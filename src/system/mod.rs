use anyhow::Result;
use rayon::prelude::*;
use regex::Regex;
use std::process::Command;

pub struct SystemManager;

#[derive(Debug, Clone, PartialEq)]
pub enum DefaultsValueType {
    Bool,
    Int,
    Float,
    String,
    Array,
}

#[derive(Debug, Clone)]
pub struct DefaultsSetting {
    pub domain: String,
    pub key: String,
    pub value_type: DefaultsValueType,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingStatus {
    Applied,    // Current value matches config
    NotApplied, // Current value differs from config
    Unknown,    // Can't read current value (key doesn't exist, permission issue, etc.)
    Skipped,    // Not a defaults command (e.g., killall)
}

#[derive(Debug, Clone)]
pub struct SettingCheck {
    pub command: String,
    pub status: SettingStatus,
}

impl SystemManager {
    pub fn new() -> Self {
        Self
    }

    pub fn apply_commands(&self, commands: &[String]) -> Result<()> {
        for cmd in commands {
            log::info!("→ Running: {}", cmd);

            let result = Command::new("sh").arg("-c").arg(cmd).status()?;

            if !result.success() {
                log::warn!("Command failed: {}", cmd);
            }
        }

        Ok(())
    }

    /// Parse a defaults command into structured format
    /// Example: "defaults write com.apple.dock autohide -bool true"
    pub fn parse_defaults_command(cmd: &str) -> Option<DefaultsSetting> {
        let cmd = cmd.trim();

        // Check if this is a defaults write command
        if !cmd.starts_with("defaults write") {
            return None;
        }

        // Regex to parse: defaults write <domain> <key> <-type> <value>
        // Handle quoted strings and various value types
        // Also handle empty arrays: "defaults write domain key -array"
        let re = Regex::new(
            r#"defaults\s+write\s+(\S+)\s+(\S+)\s+-(bool|int|float|string|array)(?:\s+(.+))?"#,
        )
        .ok()?;

        let caps = re.captures(cmd)?;

        let domain = caps.get(1)?.as_str().to_string();
        let key = caps.get(2)?.as_str().to_string();
        let type_str = caps.get(3)?.as_str();

        // Value might be empty for arrays (e.g., "defaults write ... -array")
        let value = caps
            .get(4)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();

        let value_type = match type_str {
            "bool" => DefaultsValueType::Bool,
            "int" => DefaultsValueType::Int,
            "float" => DefaultsValueType::Float,
            "string" => DefaultsValueType::String,
            "array" => DefaultsValueType::Array,
            _ => return None,
        };

        Some(DefaultsSetting {
            domain,
            key,
            value_type,
            value,
        })
    }

    /// Read current value of a defaults setting
    fn get_current_value(domain: &str, key: &str) -> Option<String> {
        let output = Command::new("defaults")
            .args(["read", domain, key])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let value = String::from_utf8_lossy(&output.stdout);
        Some(value.trim().to_string())
    }

    /// Normalize values for comparison
    /// Handles bool conversion (1/0 <-> true/false), string quoting, etc.
    fn normalize_value(value: &str, value_type: &DefaultsValueType) -> String {
        match value_type {
            DefaultsValueType::Bool => {
                // Convert various bool representations to canonical form
                match value.to_lowercase().as_str() {
                    "1" | "true" | "yes" => "1".to_string(),
                    "0" | "false" | "no" => "0".to_string(),
                    _ => value.to_string(),
                }
            }
            DefaultsValueType::String => {
                // Remove quotes if present
                value.trim_matches(|c| c == '"' || c == '\'').to_string()
            }
            DefaultsValueType::Int | DefaultsValueType::Float => {
                // Numbers stay as-is
                value.to_string()
            }
            DefaultsValueType::Array => {
                // For arrays, normalize empty representations
                let trimmed = value.trim();
                if trimmed.is_empty() || trimmed == "()" {
                    // Empty array
                    String::new()
                } else {
                    trimmed.to_string()
                }
            }
        }
    }

    /// Check if a setting is currently applied
    pub fn is_setting_applied(&self, cmd: &str) -> SettingStatus {
        // Try to parse as defaults command
        let setting = match Self::parse_defaults_command(cmd) {
            Some(s) => s,
            None => return SettingStatus::Skipped, // Not a defaults command
        };

        // Try to read current value
        let current = match Self::get_current_value(&setting.domain, &setting.key) {
            Some(v) => v,
            None => return SettingStatus::Unknown, // Can't read current value
        };

        // Normalize both values for comparison
        let current_normalized = Self::normalize_value(&current, &setting.value_type);
        let desired_normalized = Self::normalize_value(&setting.value, &setting.value_type);

        if current_normalized == desired_normalized {
            SettingStatus::Applied
        } else {
            SettingStatus::NotApplied
        }
    }

    /// Check all settings in parallel
    pub fn check_settings(&self, commands: &[String]) -> Result<Vec<SettingCheck>> {
        // Use rayon for parallel checking - much faster for many settings
        let checks: Vec<SettingCheck> = commands
            .par_iter()
            .map(|cmd| {
                let status = self.is_setting_applied(cmd);
                SettingCheck {
                    command: cmd.clone(),
                    status,
                }
            })
            .collect();

        Ok(checks)
    }
}
