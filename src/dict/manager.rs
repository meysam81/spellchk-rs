use anyhow::{Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::PathBuf;

// Use a specific commit hash for reproducibility and stability
// This prevents unexpected changes from the 'master' branch
const WORDLIST_BASE_URL: &str =
    "https://raw.githubusercontent.com/dwyl/english-words/6e4bc58ad764c3e6df8b5be4048671962c9d6a23";
const WORDLIST_VERSION: &str = "2023.12";

pub struct DictionaryInfo {
    pub language: String,
    pub path: PathBuf,
    pub word_count: usize,
    pub size_bytes: u64,
}

pub fn list_dictionaries() -> Result<()> {
    let data_dir = crate::config::Config::data_dir().context("Failed to get data directory")?;

    if !data_dir.exists() {
        println!("{}", "No dictionaries installed.".yellow());
        println!(
            "Run {} to download a dictionary.",
            "spellchk dict download en_US".cyan()
        );
        return Ok(());
    }

    println!("{}", "Installed dictionaries:".bold());
    println!();

    let entries = fs::read_dir(&data_dir)?;
    let mut found_any = false;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("dict") {
            found_any = true;
            let language = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            let metadata = fs::metadata(&path)?;
            let size_kb = metadata.len() / 1024;

            println!(
                "  {} {} ({})",
                "✓".green(),
                language.cyan().bold(),
                format!("{}KB", size_kb).dimmed()
            );
        }
    }

    if !found_any {
        println!("{}", "No dictionaries found.".yellow());
    }

    println!();
    println!(
        "Data directory: {}",
        data_dir.display().to_string().dimmed()
    );

    Ok(())
}

pub fn download_dictionary(language: &str) -> Result<()> {
    println!(
        "{} dictionary for {} (version: {})...",
        "Downloading".cyan().bold(),
        language.yellow(),
        WORDLIST_VERSION.dimmed()
    );

    let data_dir = crate::config::Config::data_dir().context("Failed to get data directory")?;

    fs::create_dir_all(&data_dir).context("Failed to create data directory")?;

    // Download from pinned commit hash for reproducibility
    // Using specific commit instead of 'master' prevents unexpected changes
    let wordlist_url = match language {
        "en_US" | "en_GB" => {
            format!("{}/words_alpha.txt", WORDLIST_BASE_URL)
        }
        other => {
            anyhow::bail!(
                "Language '{}' is not supported. Only 'en_US' and 'en_GB' are currently available.",
                other
            );
        }
    };

    println!("Source: {}", wordlist_url.dimmed());

    // Download wordlist
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Downloading...");

    let response =
        reqwest::blocking::get(&wordlist_url).context("Failed to download dictionary")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download dictionary: HTTP {}", response.status());
    }

    let content = response.text()?;
    pb.finish_with_message("Download complete");

    // Parse words
    println!("{}", "Building dictionary...".cyan());
    let words: Vec<String> = content
        .lines()
        .map(|line| line.trim().to_lowercase())
        .filter(|line| !line.is_empty() && line.len() > 1)
        .collect();

    println!("Found {} words", words.len().to_string().yellow());

    // Build FST dictionary
    let dict_path = data_dir.join(format!("{}.dict", language));
    crate::checker::dictionary::Dictionary::build_from_words(&words, &dict_path)?;

    println!(
        "{} Dictionary installed: {}",
        "✓".green().bold(),
        dict_path.display().to_string().cyan()
    );

    Ok(())
}

pub fn update_dictionaries() -> Result<()> {
    let data_dir = crate::config::Config::data_dir().context("Failed to get data directory")?;

    if !data_dir.exists() {
        println!("{}", "No dictionaries installed.".yellow());
        return Ok(());
    }

    let entries = fs::read_dir(&data_dir)?;
    let mut languages = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("dict") {
            if let Some(language) = path.file_stem().and_then(|s| s.to_str()) {
                languages.push(language.to_string());
            }
        }
    }

    if languages.is_empty() {
        println!("{}", "No dictionaries to update.".yellow());
        return Ok(());
    }

    println!(
        "{} {} {}...",
        "Updating".cyan().bold(),
        languages.len(),
        if languages.len() == 1 {
            "dictionary"
        } else {
            "dictionaries"
        }
    );
    println!();

    for language in languages {
        download_dictionary(&language)?;
        println!();
    }

    println!("{} All dictionaries updated!", "✓".green().bold());

    Ok(())
}

pub fn show_info(language: &str) -> Result<()> {
    let data_dir = crate::config::Config::data_dir().context("Failed to get data directory")?;

    let dict_path = data_dir.join(format!("{}.dict", language));

    if !dict_path.exists() {
        println!(
            "{} Dictionary for {} not found.",
            "✗".red().bold(),
            language.yellow()
        );
        println!(
            "Run {} to download it.",
            format!("spellchk dict download {}", language).cyan()
        );
        return Ok(());
    }

    let metadata = fs::metadata(&dict_path)?;

    println!("{}", format!("Dictionary: {}", language).bold());
    println!("  Path: {}", dict_path.display());
    println!("  Size: {} KB", metadata.len() / 1024);
    println!("  Version: {}", WORDLIST_VERSION);
    println!("  Format: FST (Finite State Transducer)");

    // Try to load and get word count
    match crate::checker::dictionary::Dictionary::load(language) {
        Ok(_dict) => {
            println!("  Words: {}", "Unknown".yellow());
        }
        Err(e) => {
            println!("  {}: {}", "Error loading dictionary".red(), e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_info() {
        // Test is environment-dependent, so we just ensure it doesn't panic
        let _ = list_dictionaries();
    }
}
