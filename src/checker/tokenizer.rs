// Tokenizer utilities - currently implemented in parser modules
// This module can be expanded for more sophisticated tokenization

pub fn split_compound_word(word: &str) -> Vec<String> {
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

    if result.is_empty() {
        vec![word.to_string()]
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compound_splitting() {
        assert_eq!(split_compound_word("camelCase"), vec!["camel", "case"]);
        assert_eq!(split_compound_word("snake_case"), vec!["snake", "case"]);
        assert_eq!(split_compound_word("kebab-case"), vec!["kebab", "case"]);
    }
}
