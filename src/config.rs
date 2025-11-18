use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub language: String,
    pub personal_dictionary: Option<PathBuf>,
    pub ignore_patterns: Vec<String>,

    #[serde(default)]
    pub enabled_rules: Vec<String>,

    #[serde(default = "default_max_suggestions")]
    pub max_suggestions: usize,

    #[serde(default)]
    pub case_sensitive: bool,
}

fn default_max_suggestions() -> usize {
    5
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: "en_US".to_string(),
            personal_dictionary: None,
            ignore_patterns: vec![
                r"\b[A-Z0-9_]{2,}\b".to_string(),    // ALL_CAPS
                r"https?://\S+".to_string(),         // URLs
                r"\b[a-fA-F0-9]{32,}\b".to_string(), // Hashes
                r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}".to_string(), // Emails
            ],
            enabled_rules: vec!["check-compound".to_string(), "check-rare".to_string()],
            max_suggestions: 5,
            case_sensitive: false,
        }
    }
}

impl Config {
    /// Load configuration with priority: CLI args > local config > global config > defaults
    pub fn load(
        language: String,
        personal_dict: Option<PathBuf>,
        cli_patterns: Vec<String>,
    ) -> Result<Self> {
        let mut config = Self::default();

        // Load global config
        if let Some(global_path) = Self::global_config_path() {
            if global_path.exists() {
                let global_config = Self::from_file(&global_path)?;
                config = config.merge(global_config);
            }
        }

        // Load local config (overrides global)
        let local_path = PathBuf::from(".spellchk.toml");
        if local_path.exists() {
            let local_config = Self::from_file(&local_path)?;
            config = config.merge(local_config);
        }

        // Apply CLI overrides
        config.language = language;
        if let Some(dict) = personal_dict {
            config.personal_dictionary = Some(dict);
        }
        if !cli_patterns.is_empty() {
            config.ignore_patterns.extend(cli_patterns);
        }

        // Set default personal dictionary if not specified
        if config.personal_dictionary.is_none() {
            config.personal_dictionary = Self::default_personal_dict_path();
        }

        // Ensure personal dictionary file exists
        if let Some(path) = &config.personal_dictionary {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .context("Failed to create personal dictionary directory")?;
            }
            if !path.exists() {
                fs::write(path, "").context("Failed to create personal dictionary file")?;
            }
        }

        Ok(config)
    }

    fn from_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))
    }

    fn merge(mut self, other: Self) -> Self {
        // Merge logic: other's values override self's if they differ from defaults
        if other.language != "en_US" {
            self.language = other.language;
        }
        if other.personal_dictionary.is_some() {
            self.personal_dictionary = other.personal_dictionary;
        }
        if !other.ignore_patterns.is_empty() {
            self.ignore_patterns = other.ignore_patterns;
        }
        if !other.enabled_rules.is_empty() {
            self.enabled_rules = other.enabled_rules;
        }
        if other.max_suggestions != default_max_suggestions() {
            self.max_suggestions = other.max_suggestions;
        }
        self.case_sensitive = other.case_sensitive;
        self
    }

    pub fn global_config_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "spellchk").map(|dirs| dirs.config_dir().join("config.toml"))
    }

    pub fn default_personal_dict_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "spellchk").map(|dirs| dirs.config_dir().join("personal.txt"))
    }

    pub fn cache_dir() -> Option<PathBuf> {
        ProjectDirs::from("", "", "spellchk").map(|dirs| dirs.cache_dir().to_path_buf())
    }

    pub fn data_dir() -> Option<PathBuf> {
        ProjectDirs::from("", "", "spellchk").map(|dirs| dirs.data_dir().to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.language, "en_US");
        assert_eq!(config.max_suggestions, 5);
        assert!(!config.case_sensitive);
    }

    #[test]
    fn test_merge_configs() {
        let base = Config::default();
        let override_config = Config {
            language: "en_GB".to_string(),
            ..Default::default()
        };

        let merged = base.merge(override_config);
        assert_eq!(merged.language, "en_GB");
    }
}
