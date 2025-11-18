use crate::parser::{SourceLang, TextSpan};
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // Regex patterns for different comment styles
    static ref C_STYLE_COMMENT: Regex = Regex::new(r"//(.*)$|/\*([^*]*\*+(?:[^/*][^*]*\*+)*)*/").unwrap();
    static ref PYTHON_COMMENT: Regex = Regex::new(r#"#(.*)$|'''([^']*)'''|"""([^"]*)""""#).unwrap();
    static ref STRING_LITERAL: Regex = Regex::new(r#""([^"\\]*(\\.[^"\\]*)*)"|'([^'\\]*(\\.[^'\\]*)*)'"#).unwrap();
}

/// Parse source code and extract checkable text from comments and strings
pub fn parse(content: &str, lang: SourceLang) -> Result<Vec<TextSpan>> {
    match lang {
        SourceLang::Python => parse_python_style(content),
        _ => parse_c_style(content),
    }
}

fn parse_c_style(content: &str) -> Result<Vec<TextSpan>> {
    let mut spans = Vec::new();
    let mut byte_offset = 0;

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1;

        // Extract from line comments (// ...)
        if let Some(idx) = line.find("//") {
            let comment = &line[idx + 2..];
            let words = extract_words(comment);
            let comment_start = byte_offset + idx + 2;

            for (word, offset) in words {
                let start = comment_start + offset;
                let end = start + word.len();

                spans.push(TextSpan {
                    text: word.clone(),
                    line: line_num,
                    column: idx + 2 + offset,
                    start,
                    end,
                    original_text: comment.to_string(),
                });
            }
        }

        // Extract from strings (improved: handles escaped quotes)
        let mut chars = line.char_indices().peekable();
        while let Some((start_idx, ch)) = chars.next() {
            if ch == '"' || ch == '\'' {
                let quote = ch;
                let mut content = String::new();
                let mut escaped = false;
                for (_i, c) in chars.by_ref() {
                    if escaped {
                        content.push(c);
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == quote {
                        break;
                    } else {
                        content.push(c);
                    }
                }
                let words = extract_words(&content);
                for (word, _) in words {
                    // For strings, we have approximate byte offsets
                    // Exact calculation would require tracking within the string content
                    let start = byte_offset + start_idx + 1;
                    let end = start + word.len();

                    spans.push(TextSpan {
                        text: word.clone(),
                        line: line_num,
                        column: start_idx + 1,
                        start,
                        end,
                        original_text: content.clone(),
                    });
                }
            }
        }

        // Move to next line (line bytes + newline)
        byte_offset += line.len() + 1;
    }

    // TODO: Handle multi-line block comments /* ... */
    // For now, we do basic single-line processing

    Ok(spans)
}

fn parse_python_style(content: &str) -> Result<Vec<TextSpan>> {
    let mut spans = Vec::new();
    let mut byte_offset = 0;

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1;

        // Extract from comments (# ...)
        if let Some(idx) = line.find('#') {
            // Make sure it's not inside a string
            let before = &line[..idx];
            let quote_count = before.matches('"').count() + before.matches('\'').count();

            if quote_count % 2 == 0 {
                let comment = &line[idx + 1..];
                let words = extract_words(comment);
                let comment_start = byte_offset + idx + 1;

                for (word, offset) in words {
                    let start = comment_start + offset;
                    let end = start + word.len();

                    spans.push(TextSpan {
                        text: word.clone(),
                        line: line_num,
                        column: idx + 1 + offset,
                        start,
                        end,
                        original_text: comment.to_string(),
                    });
                }
            }
        }

        // Extract from strings
        for cap in STRING_LITERAL.captures_iter(line) {
            if let Some(string_content) = cap.get(1).or_else(|| cap.get(3)) {
                let content = string_content.as_str();
                let words = extract_words(content);
                let string_start = byte_offset + string_content.start();

                for (word, _) in words {
                    // Approximate byte offset within string
                    let start = string_start + 1; // +1 for opening quote
                    let end = start + word.len();

                    spans.push(TextSpan {
                        text: word.clone(),
                        line: line_num,
                        column: string_content.start(),
                        start,
                        end,
                        original_text: content.to_string(),
                    });
                }
            }
        }

        // Move to next line (line bytes + newline)
        byte_offset += line.len() + 1;
    }

    Ok(spans)
}

fn extract_words(text: &str) -> Vec<(String, usize)> {
    let mut words = Vec::new();
    let mut current_word = String::new();
    let mut word_start = 0;
    let mut in_word = false;

    for (i, ch) in text.char_indices() {
        if ch.is_alphabetic() {
            if !in_word {
                word_start = i;
                in_word = true;
            }
            current_word.push(ch);
        } else if in_word && !current_word.is_empty() && current_word.len() > 1 {
            words.push((current_word.clone(), word_start));
            current_word.clear();
            in_word = false;
        } else if in_word {
            current_word.clear();
            in_word = false;
        }
    }

    if in_word && !current_word.is_empty() && current_word.len() > 1 {
        words.push((current_word, word_start));
    }

    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_style_comments() {
        let content = r#"
// This is a comment with words
fn main() {
    println!("A string with text");
}
"#;

        let spans = parse(content, SourceLang::Rust).unwrap();
        assert!(!spans.is_empty());

        // Should extract from comment
        let comment_words: Vec<_> = spans.iter().filter(|s| s.text == "comment").collect();
        assert!(!comment_words.is_empty());
    }

    #[test]
    fn test_python_comments() {
        let content = r#"
# This is a Python comment
def main():
    print("A string with text")
"#;

        let spans = parse(content, SourceLang::Python).unwrap();
        assert!(!spans.is_empty());
    }
}
