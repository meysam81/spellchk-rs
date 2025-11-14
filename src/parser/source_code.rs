use crate::parser::{SourceLang, TextSpan};
use anyhow::Result;
use regex::Regex;
use lazy_static::lazy_static;

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

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1;

        // Extract from line comments (// ...)
        if let Some(idx) = line.find("//") {
            let comment = &line[idx + 2..];
            let words = extract_words(comment);
            for (word, offset) in words {
                spans.push(TextSpan {
                    text: word.clone(),
                    line: line_num,
                    column: idx + 2 + offset,
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
                let mut end_idx = start_idx + 1;
                while let Some((i, c)) = chars.next() {
                    end_idx = i + c.len_utf8();
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
                    spans.push(TextSpan {
                        text: word.clone(),
                        line: line_num,
                        column: start_idx + 1,
                        original_text: content.clone(),
                    });
                }
            }
        }
    }

    // TODO: Handle multi-line block comments /* ... */
    // For now, we do basic single-line processing

    Ok(spans)
}

fn parse_python_style(content: &str) -> Result<Vec<TextSpan>> {
    let mut spans = Vec::new();

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
                for (word, offset) in words {
                    spans.push(TextSpan {
                        text: word.clone(),
                        line: line_num,
                        column: idx + 1 + offset,
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
                for (word, _) in words {
                    spans.push(TextSpan {
                        text: word.clone(),
                        line: line_num,
                        column: string_content.start(),
                        original_text: content.to_string(),
                    });
                }
            }
        }
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
        } else {
            if in_word && !current_word.is_empty() && current_word.len() > 1 {
                words.push((current_word.clone(), word_start));
                current_word.clear();
                in_word = false;
            } else if in_word {
                current_word.clear();
                in_word = false;
            }
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
        let comment_words: Vec<_> = spans
            .iter()
            .filter(|s| s.text == "comment")
            .collect();
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
