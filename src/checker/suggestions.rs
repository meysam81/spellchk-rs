use crate::checker::dictionary::Dictionary;

/// Generate spelling suggestions using edit distance
pub fn generate(word: &str, dictionary: &Dictionary, max_suggestions: usize) -> Vec<String> {
    // Try progressively more expensive operations
    let mut suggestions = Vec::new();

    // 1. Try prefix matching (fast)
    if word.len() >= 3 {
        let prefix = &word[..3];
        let mut prefix_matches = dictionary.words_with_prefix(prefix);
        prefix_matches.sort_by_key(|w| edit_distance(word, w));
        prefix_matches.truncate(max_suggestions);

        for suggestion in prefix_matches {
            if edit_distance(word, &suggestion) <= 2 {
                suggestions.push(suggestion);
            }
        }
    }

    if suggestions.len() >= max_suggestions {
        suggestions.truncate(max_suggestions);
        return suggestions;
    }

    // 2. Try common transformations (medium speed)
    let transformations = generate_transformations(word);
    for transform in transformations {
        if dictionary.contains(&transform) && !suggestions.contains(&transform) {
            suggestions.push(transform);
            if suggestions.len() >= max_suggestions {
                suggestions.truncate(max_suggestions);
                return suggestions;
            }
        }
    }

    // 3. Try different prefix lengths (medium speed)
    if suggestions.len() < max_suggestions && word.len() >= 2 {
        // Try 2-character prefix for shorter words
        let prefix = &word[..2];
        let mut prefix_matches = dictionary.words_with_prefix(prefix);
        prefix_matches.sort_by_key(|w| edit_distance(word, w));

        for candidate in prefix_matches {
            let distance = edit_distance(word, &candidate);
            if distance <= 3 && !suggestions.contains(&candidate) {
                suggestions.push(candidate);
                if suggestions.len() >= max_suggestions {
                    suggestions.truncate(max_suggestions);
                    return suggestions;
                }
            }
        }
    }

    // 4. Only do expensive full-dictionary search for very short words (≤3 chars)
    // This is a last resort and only acceptable for words like "is", "an", "to", etc.
    // Most misspellings are longer, so this rarely executes in practice
    if suggestions.len() < max_suggestions && word.len() <= 3 {
        // For very short words only, do a limited full-dictionary scan
        let all_words = dictionary.all_words();
        let mut candidates: Vec<_> = all_words
            .into_iter()
            .filter(|w| {
                // Pre-filter by length to reduce edit distance calculations
                let len_diff = (w.len() as i32 - word.len() as i32).abs();
                len_diff <= 1
            })
            .take(100) // Limit candidates to first 100 matching length criteria
            .filter_map(|w| {
                let dist = edit_distance(word, &w);
                if dist <= 2 && !suggestions.contains(&w) {
                    Some((dist, w))
                } else {
                    None
                }
            })
            .collect();

        candidates.sort_by_key(|(dist, _)| *dist);

        for (_, candidate) in candidates {
            suggestions.push(candidate);
            if suggestions.len() >= max_suggestions {
                break;
            }
        }
    }

    suggestions.truncate(max_suggestions);
    suggestions
}

/// Calculate Levenshtein distance between two strings
fn edit_distance(a: &str, b: &str) -> usize {
    let a_len = a.chars().count();
    let b_len = b.chars().count();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(a_len + 1) {
        row[0] = i;
    }
    for (j, item) in matrix[0].iter_mut().enumerate().take(b_len + 1) {
        *item = j;
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    for (i, a_char) in a_chars.iter().enumerate() {
        for (j, b_char) in b_chars.iter().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };

            matrix[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(
                    matrix[i][j + 1] + 1, // deletion
                    matrix[i + 1][j] + 1, // insertion
                ),
                matrix[i][j] + cost, // substitution
            );
        }
    }

    matrix[a_len][b_len]
}

/// Generate common transformations of a word
fn generate_transformations(word: &str) -> Vec<String> {
    let mut transformations = Vec::new();
    let chars: Vec<char> = word.chars().collect();

    // Deletions
    for i in 0..chars.len() {
        let mut new_word = chars.clone();
        new_word.remove(i);
        transformations.push(new_word.iter().collect());
    }

    // Transpositions (swap adjacent)
    for i in 0..chars.len().saturating_sub(1) {
        let mut new_word = chars.clone();
        new_word.swap(i, i + 1);
        transformations.push(new_word.iter().collect());
    }

    // Replacements (common typos)
    let common_replacements = [
        ('a', 'e'),
        ('e', 'i'),
        ('i', 'o'),
        ('o', 'u'),
        ('b', 'v'),
        ('c', 'k'),
        ('f', 'v'),
        ('g', 'j'),
        ('m', 'n'),
        ('s', 'z'),
        ('t', 'd'),
    ];

    for (i, &ch) in chars.iter().enumerate() {
        for &(from, to) in &common_replacements {
            if ch == from {
                let mut new_word = chars.clone();
                new_word[i] = to;
                transformations.push(new_word.iter().collect());
            }
        }
    }

    transformations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("hello", "hello"), 0);
        assert_eq!(edit_distance("hello", "hallo"), 1);
        assert_eq!(edit_distance("hello", "hullo"), 1);
        assert_eq!(edit_distance("hello", "world"), 4);
    }

    #[test]
    fn test_transformations() {
        let transforms = generate_transformations("hello");
        assert!(transforms.contains(&"hllo".to_string())); // deletion
        assert!(transforms.contains(&"ehllo".to_string())); // transposition
    }
}
