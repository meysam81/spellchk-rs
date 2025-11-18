use crate::parser::TextSpan;
use anyhow::Result;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};

/// Parse markdown and extract checkable text (skip code blocks, inline code, URLs)
pub fn parse(content: &str) -> Result<Vec<TextSpan>> {
    let mut spans = Vec::new();
    let parser = Parser::new(content);

    let mut in_code_block = false;
    let mut in_inline_code = false;
    let mut current_line = 1;
    let mut current_column = 1;

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
            }
            Event::Code(_) => {
                in_inline_code = true;
            }
            Event::Text(text) if !in_code_block && !in_inline_code => {
                // Extract words from text
                let words = extract_words(&text);
                for (word, offset) in words {
                    spans.push(TextSpan {
                        text: word.clone(),
                        line: current_line,
                        column: current_column + offset,
                        start: 0, // TODO: Calculate accurate byte offsets for markdown
                        end: 0,   // For now, fix mode works better with plain text
                        original_text: get_context(&text, offset, word.len()),
                    });
                }
                // Update position tracking (approximate)
                current_line += text.matches('\n').count();
                if let Some(last_newline) = text.rfind('\n') {
                    current_column = text.len() - last_newline;
                } else {
                    current_column += text.len();
                }
            }
            _ => {}
        }

        if in_inline_code {
            in_inline_code = false;
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
        if ch.is_alphabetic() || ch == '\'' || ch == '-' {
            if !in_word {
                word_start = i;
                in_word = true;
            }
            current_word.push(ch);
        } else if in_word && !current_word.is_empty() {
            words.push((current_word.clone(), word_start));
            current_word.clear();
            in_word = false;
        }
    }

    // Don't forget the last word
    if in_word && !current_word.is_empty() {
        words.push((current_word, word_start));
    }

    words
}

fn get_context(text: &str, offset: usize, word_len: usize) -> String {
    let start = offset.saturating_sub(20);
    let end = (offset + word_len + 20).min(text.len());
    let context = &text[start..end];

    if start > 0 {
        format!("...{}", context)
    } else if end < text.len() {
        format!("{}...", context)
    } else {
        context.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_parsing() {
        let content = r#"
# Title

This is a test paragraph with some words.

```rust
fn main() {
    println!("This should be ignored");
}
```

More text with `inline_code` here.
"#;

        let spans = parse(content).unwrap();
        assert!(!spans.is_empty());

        // Verify we're not extracting from code blocks
        let code_words: Vec<_> = spans
            .iter()
            .filter(|s| s.text.contains("println"))
            .collect();
        assert!(code_words.is_empty());
    }

    #[test]
    fn test_word_extraction() {
        let text = "Hello, world! This is a test.";
        let words = extract_words(text);

        assert_eq!(words.len(), 6);
        assert_eq!(words[0].0, "Hello");
        assert_eq!(words[1].0, "world");
    }
}
