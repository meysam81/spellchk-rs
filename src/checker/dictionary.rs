use anyhow::{Context, Result};
use fst::{Automaton, IntoStreamer, Set, SetBuilder, Streamer};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read};
use std::path::{Path, PathBuf};

pub struct Dictionary {
    set: Set<Vec<u8>>,
}

impl Dictionary {
    /// Load dictionary for given language
    pub fn load(language: &str) -> Result<Self> {
        let dict_path = Self::get_dictionary_path(language)?;

        if !dict_path.exists() {
            // Try to create a basic embedded dictionary
            return Self::create_embedded(language);
        }

        Self::load_from_path(&dict_path)
    }

    /// Load dictionary from a specific path (useful for testing)
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open dictionary: {}", path.display()))?;

        let reader = BufReader::new(file);
        let set = Set::new(reader.bytes().collect::<Result<Vec<_>, _>>()?)
            .context("Failed to parse dictionary")?;

        Ok(Self { set })
    }

    /// Check if word exists in dictionary
    pub fn contains(&self, word: &str) -> bool {
        self.set.contains(word.as_bytes())
    }

    /// Get all words with a given prefix
    pub fn words_with_prefix(&self, prefix: &str) -> Vec<String> {
        let mut results = Vec::new();
        let mut stream = self
            .set
            .search(fst::automaton::Str::new(prefix).starts_with())
            .into_stream();

        while let Some(key) = stream.next() {
            if let Ok(word) = String::from_utf8(key.to_vec()) {
                results.push(word);
            }
        }

        results
    }

    /// Get all words in dictionary (for building suggestions)
    ///
    /// WARNING: This is an expensive operation that loads the entire dictionary
    /// into memory. Use sparingly and consider caching the result if called multiple times.
    /// Prefer using `words_with_prefix()` or direct `contains()` checks when possible.
    pub fn all_words(&self) -> Vec<String> {
        let mut words = Vec::new();
        let mut stream = self.set.stream();

        while let Some(key) = stream.next() {
            if let Ok(word) = String::from_utf8(key.to_vec()) {
                words.push(word);
            }
        }

        words
    }

    /// Build dictionary from word list
    pub fn build_from_words(words: &[String], output_path: &Path) -> Result<()> {
        let mut sorted_words = words.to_vec();
        sorted_words.sort();
        sorted_words.dedup();

        let file = File::create(output_path)
            .with_context(|| format!("Failed to create dictionary: {}", output_path.display()))?;

        let writer = BufWriter::new(file);
        let mut builder = SetBuilder::new(writer).context("Failed to create FST builder")?;

        for word in sorted_words {
            builder
                .insert(word.as_bytes())
                .context("Failed to insert word into dictionary")?;
        }

        builder.finish().context("Failed to finalize dictionary")?;

        Ok(())
    }

    fn get_dictionary_path(language: &str) -> Result<PathBuf> {
        let data_dir = crate::config::Config::data_dir().context("Failed to get data directory")?;

        std::fs::create_dir_all(&data_dir).context("Failed to create data directory")?;

        Ok(data_dir.join(format!("{}.dict", language)))
    }

    /// Create a minimal embedded dictionary for bootstrapping
    fn create_embedded(language: &str) -> Result<Self> {
        // For MVP, create a very basic dictionary
        // In production, this would be a larger embedded wordlist
        let basic_words = Self::get_basic_wordlist(language);

        let dict_path = Self::get_dictionary_path(language)?;
        Self::build_from_words(&basic_words, &dict_path)?;

        Self::load(language)
    }

    fn get_basic_wordlist(language: &str) -> Vec<String> {
        // This is a minimal set for bootstrapping
        // In production, embed a proper dictionary or require download
        match language {
            "en_US" | "en_GB" => {
                // Most common English words for basic functionality
                vec![
                    "the",
                    "be",
                    "to",
                    "of",
                    "and",
                    "a",
                    "in",
                    "that",
                    "have",
                    "i",
                    "it",
                    "for",
                    "not",
                    "on",
                    "with",
                    "he",
                    "as",
                    "you",
                    "do",
                    "at",
                    "this",
                    "but",
                    "his",
                    "by",
                    "from",
                    "they",
                    "we",
                    "say",
                    "her",
                    "she",
                    "or",
                    "an",
                    "will",
                    "my",
                    "one",
                    "all",
                    "would",
                    "there",
                    "their",
                    "what",
                    "so",
                    "up",
                    "out",
                    "if",
                    "about",
                    "who",
                    "get",
                    "which",
                    "go",
                    "me",
                    "when",
                    "make",
                    "can",
                    "like",
                    "time",
                    "no",
                    "just",
                    "him",
                    "know",
                    "take",
                    "people",
                    "into",
                    "year",
                    "your",
                    "good",
                    "some",
                    "could",
                    "them",
                    "see",
                    "other",
                    "than",
                    "then",
                    "now",
                    "look",
                    "only",
                    "come",
                    "its",
                    "over",
                    "think",
                    "also",
                    "back",
                    "after",
                    "use",
                    "two",
                    "how",
                    "our",
                    "work",
                    "first",
                    "well",
                    "way",
                    "even",
                    "new",
                    "want",
                    "because",
                    "any",
                    "these",
                    "give",
                    "day",
                    "most",
                    "us",
                    // Common programming terms
                    "function",
                    "class",
                    "method",
                    "variable",
                    "string",
                    "integer",
                    "boolean",
                    "array",
                    "list",
                    "dictionary",
                    "object",
                    "parameter",
                    "return",
                    "import",
                    "export",
                    "async",
                    "await",
                    "promise",
                    "callback",
                    "error",
                    "exception",
                    "test",
                    "debug",
                    "compile",
                    "build",
                    "deploy",
                    "version",
                    "configuration",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect()
            }
            _ => {
                // Default to English wordlist for unknown languages
                [
                    "the", "be", "to", "of", "and", "a", "in", "that", "have", "i",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_build_and_load_dictionary() {
        let dir = tempdir().unwrap();
        let dict_path = dir.path().join("test.dict");

        let words = vec!["hello".to_string(), "world".to_string(), "test".to_string()];

        Dictionary::build_from_words(&words, &dict_path).unwrap();

        // Load from the specific path we just created
        let dict = Dictionary::load_from_path(&dict_path).unwrap();
        assert!(dict.contains("hello"));
        assert!(dict.contains("world"));
        assert!(!dict.contains("notfound"));
    }
}
