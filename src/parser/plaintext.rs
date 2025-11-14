use crate::parser::TextSpan;
use anyhow::Result;
use unicode_segmentation::UnicodeSegmentation;

/// Parse plain text and extract all words
pub fn parse(content: &str) -> Result<Vec<TextSpan>> {
    let mut spans = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1;
        let words = extract_words(line);

        for (word, column) in words {
            spans.push(TextSpan {
                text: word.clone(),
                line: line_num,
                column: column + 1, // 1-indexed
                original_text: get_context(line, column, word.len()),
            });
        }
    }

    Ok(spans)
}

fn extract_words(text: &str) -> Vec<(String, usize)> {
    let mut words = Vec::new();
    let mut current_word = String::new();
    let mut word_start = 0;
    let mut current_pos = 0;

    for grapheme in text.graphemes(true) {
        let ch = grapheme.chars().next().unwrap_or(' ');

        if ch.is_alphabetic() || ch == '\'' || ch == '-' {
            if current_word.is_empty() {
                word_start = current_pos;
            }
            current_word.push_str(grapheme);
        } else {
            if !current_word.is_empty() {
                // Split camelCase and snake_case
                let split_words = split_compound_word(&current_word);
                for split_word in split_words {
                    if split_word.len() > 1 {
                        words.push((split_word, word_start));
                    }
                }
                current_word.clear();
            }
        }

        current_pos += grapheme.len();
    }

    // Handle last word
    if !current_word.is_empty() {
        let split_words = split_compound_word(&current_word);
        for split_word in split_words {
            if split_word.len() > 1 {
                words.push((split_word, word_start));
            }
        }
    }

    words
}

/// Split camelCase and snake_case into individual words
fn split_compound_word(word: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();

    for ch in word.chars() {
        if ch == '_' || ch == '-' {
            if !current.is_empty() {
                result.push(current.clone());
                current.clear();
            }
        } else if ch.is_uppercase() && !current.is_empty() {
            result.push(current.clone());
            current.clear();
            current.push(ch.to_lowercase().next().unwrap());
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    // If no splitting occurred, return original word
    if result.is_empty() {
        vec![word.to_string()]
    } else {
        result
    }
}

fn get_context(line: &str, offset: usize, word_len: usize) -> String {
    let start = offset.saturating_sub(20);
    let end = (offset + word_len + 20).min(line.len());
    let context = &line[start..end];

    if start > 0 && end < line.len() {
        format!("...{}...", context)
    } else if start > 0 {
        format!("...{}", context)
    } else if end < line.len() {
        format!("{}...", context)
    } else {
        context.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text_parsing() {
        let content = "Hello world! This is a test.";
        let spans = parse(content).unwrap();

        // Note: Single-character words like "a" are filtered out (len > 1 check)
        // So we get: "Hello", "world", "This", "is", "test" = 5 words
        assert_eq!(spans.len(), 5);
        assert_eq!(spans[0].text, "Hello");
        assert_eq!(spans[0].line, 1);
    }

    #[test]
    fn test_camel_case_splitting() {
        let words = split_compound_word("camelCaseWord");
        assert_eq!(words, vec!["camel", "case", "word"]);
    }

    #[test]
    fn test_snake_case_splitting() {
        let words = split_compound_word("snake_case_word");
        assert_eq!(words, vec!["snake", "case", "word"]);
    }

    #[test]
    fn test_multiline() {
        let content = "First line\nSecond line\nThird line";
        let spans = parse(content).unwrap();

        assert!(spans.iter().any(|s| s.line == 1));
        assert!(spans.iter().any(|s| s.line == 2));
        assert!(spans.iter().any(|s| s.line == 3));
    }
}
