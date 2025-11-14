pub mod dictionary;
pub mod suggestions;
pub mod tokenizer;

use crate::cli::output::{print_errors, print_interactive_prompt, OutputFormat};
use crate::{CheckResult, Config, SpellError};
use anyhow::{Context, Result};
use dictionary::Dictionary;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub struct SpellChecker {
    dictionary: Dictionary,
    personal_words: HashSet<String>,
    ignore_patterns: Vec<Regex>,
}

impl SpellChecker {
    pub fn new(config: &Config) -> Result<Self> {
        // Load main dictionary
        let dictionary = Dictionary::load(&config.language)?;

        // Load personal dictionary
        let mut personal_words = HashSet::new();
        if let Some(personal_dict_path) = &config.personal_dictionary {
            if personal_dict_path.exists() {
                let content = fs::read_to_string(personal_dict_path)
                    .context("Failed to read personal dictionary")?;
                for line in content.lines() {
                    let word = line.trim();
                    if !word.is_empty() && !word.starts_with('#') {
                        personal_words.insert(word.to_lowercase());
                    }
                }
            }
        }

        // Compile ignore patterns
        let mut ignore_patterns = Vec::new();
        for pattern in &config.ignore_patterns {
            match Regex::new(pattern) {
                Ok(re) => ignore_patterns.push(re),
                Err(e) => eprintln!("Warning: Invalid regex pattern '{}': {}", pattern, e),
            }
        }

        Ok(Self {
            dictionary,
            personal_words,
            ignore_patterns,
        })
    }

    pub fn check(
        &self,
        file_path: &Path,
        config: &Config,
        colored: bool,
        format: &OutputFormat,
    ) -> Result<CheckResult> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        let spans = crate::parser::parse_file(file_path, &content)?;

        let mut errors = Vec::new();

        for span in spans {
            let word_lower = span.text.to_lowercase();

            // Skip if in personal dictionary
            if self.personal_words.contains(&word_lower) {
                continue;
            }

            // Skip if matches ignore pattern
            if self.should_ignore(&span.text) {
                continue;
            }

            // Skip if in main dictionary
            if self.dictionary.contains(&word_lower) {
                continue;
            }

            // Word is misspelled - generate suggestions
            let suggestions = suggestions::generate(&word_lower, &self.dictionary, config.max_suggestions);

            errors.push(SpellError {
                word: span.text.clone(),
                line: span.line,
                column: span.column,
                context: span.original_text,
                suggestions,
            });
        }

        let result = CheckResult {
            error_count: errors.len(),
            fixed_count: 0,
            errors,
        };

        // Print errors in requested format
        print_errors(file_path, &result, colored, format);

        Ok(result)
    }

    pub fn fix_auto(
        &self,
        file_path: &Path,
        _config: &Config,
        _colored: bool,
    ) -> Result<CheckResult> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        let spans = crate::parser::parse_file(file_path, &content)?;
        let mut replacements = Vec::new();

        for span in spans {
            let word_lower = span.text.to_lowercase();

            if self.personal_words.contains(&word_lower) {
                continue;
            }

            if self.should_ignore(&span.text) {
                continue;
            }

            if self.dictionary.contains(&word_lower) {
                continue;
            }

            // Get top suggestion
            let suggestions = suggestions::generate(&word_lower, &self.dictionary, 1);
            if let Some(top_suggestion) = suggestions.first() {
                replacements.push((span.text.clone(), top_suggestion.clone()));
            }
        }

        // Apply replacements
        let mut new_content = content.clone();
        let mut fixed_count = 0;

        for (old_word, new_word) in &replacements {
            if new_content.contains(old_word) {
                new_content = new_content.replacen(old_word, new_word, 1);
                fixed_count += 1;
            }
        }

        // Write back to file
        if fixed_count > 0 {
            fs::write(file_path, new_content)
                .with_context(|| format!("Failed to write file: {}", file_path.display()))?;
        }

        Ok(CheckResult {
            error_count: 0,
            fixed_count,
            errors: Vec::new(),
        })
    }

    pub fn fix_interactive(
        &self,
        file_path: &Path,
        config: &Config,
        colored: bool,
    ) -> Result<CheckResult> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        let spans = crate::parser::parse_file(file_path, &content)?;
        let mut replacements = Vec::new();
        let mut words_to_add = Vec::new();

        for span in spans {
            let word_lower = span.text.to_lowercase();

            if self.personal_words.contains(&word_lower) {
                continue;
            }

            if self.should_ignore(&span.text) {
                continue;
            }

            if self.dictionary.contains(&word_lower) {
                continue;
            }

            // Get suggestions
            let suggestions = suggestions::generate(&word_lower, &self.dictionary, config.max_suggestions);

            // Prompt user
            if let Some(choice) = print_interactive_prompt(
                &span.text,
                &suggestions,
                &span.original_text,
                span.line,
                span.column,
                colored,
            ) {
                if choice == span.text {
                    // User chose to add to dictionary
                    words_to_add.push(word_lower);
                } else {
                    // User chose a replacement
                    replacements.push((span.text.clone(), choice));
                }
            }
        }

        // Apply replacements
        let mut new_content = content.clone();
        let mut fixed_count = 0;

        for (old_word, new_word) in &replacements {
            if new_content.contains(old_word) {
                new_content = new_content.replacen(old_word, new_word, 1);
                fixed_count += 1;
            }
        }

        // Write back to file
        if fixed_count > 0 {
            fs::write(file_path, new_content)
                .with_context(|| format!("Failed to write file: {}", file_path.display()))?;
        }

        // Add words to personal dictionary
        if !words_to_add.is_empty() {
            if let Some(personal_dict_path) = &config.personal_dictionary {
                let mut dict_content = if personal_dict_path.exists() {
                    fs::read_to_string(personal_dict_path)?
                } else {
                    String::new()
                };

                for word in words_to_add {
                    dict_content.push_str(&format!("{}\n", word));
                }

                fs::write(personal_dict_path, dict_content)?;
            }
        }

        Ok(CheckResult {
            error_count: 0,
            fixed_count,
            errors: Vec::new(),
        })
    }

    fn should_ignore(&self, word: &str) -> bool {
        // Skip single characters
        if word.len() <= 1 {
            return true;
        }

        // Skip all numbers
        if word.chars().all(|c| c.is_numeric()) {
            return true;
        }

        // Check ignore patterns
        for pattern in &self.ignore_patterns {
            if pattern.is_match(word) {
                return true;
            }
        }

        false
    }
}
