pub mod markdown;
pub mod plaintext;
pub mod source_code;

use anyhow::Result;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Markdown,
    SourceCode(SourceLang),
    PlainText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceLang {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    Java,
    C,
    Cpp,
    Jsx,
    Tsx,
    Other,
}

impl FileType {
    /// Detect file type from extension
    pub fn from_path(path: &Path) -> Self {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "md" | "mdx" | "markdown" => FileType::Markdown,
            "rs" => FileType::SourceCode(SourceLang::Rust),
            "js" | "mjs" | "cjs" => FileType::SourceCode(SourceLang::JavaScript),
            "ts" | "mts" | "cts" => FileType::SourceCode(SourceLang::TypeScript),
            "jsx" => FileType::SourceCode(SourceLang::Jsx),
            "tsx" => FileType::SourceCode(SourceLang::Tsx),
            "py" | "pyw" => FileType::SourceCode(SourceLang::Python),
            "go" => FileType::SourceCode(SourceLang::Go),
            "java" => FileType::SourceCode(SourceLang::Java),
            "c" | "h" => FileType::SourceCode(SourceLang::C),
            "cpp" | "cc" | "cxx" | "hpp" | "hh" => FileType::SourceCode(SourceLang::Cpp),
            _ => FileType::PlainText,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextSpan {
    pub text: String,
    pub line: usize,
    pub column: usize,
    pub original_text: String, // For context
    pub start: usize, // Byte offset where the span starts
    pub end: usize,   // Byte offset where the span ends
}

/// Parse a file and extract checkable text spans
pub fn parse_file(path: &Path, content: &str) -> Result<Vec<TextSpan>> {
    let file_type = FileType::from_path(path);

    match file_type {
        FileType::Markdown => markdown::parse(content),
        FileType::SourceCode(lang) => source_code::parse(content, lang),
        FileType::PlainText => plaintext::parse(content),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_file_type_detection() {
        assert_eq!(
            FileType::from_path(&PathBuf::from("test.md")),
            FileType::Markdown
        );
        assert_eq!(
            FileType::from_path(&PathBuf::from("main.rs")),
            FileType::SourceCode(SourceLang::Rust)
        );
        assert_eq!(
            FileType::from_path(&PathBuf::from("app.tsx")),
            FileType::SourceCode(SourceLang::Tsx)
        );
        assert_eq!(
            FileType::from_path(&PathBuf::from("notes.txt")),
            FileType::PlainText
        );
    }
}
