pub mod checker;
pub mod cli;
pub mod config;
pub mod dict;
pub mod parser;

pub use checker::SpellChecker;
pub use config::Config;

#[derive(Debug, Clone, Default)]
pub struct CheckResult {
    pub error_count: usize,
    pub fixed_count: usize,
    pub errors: Vec<SpellError>,
}

#[derive(Debug, Clone)]
pub struct SpellError {
    pub word: String,
    pub line: usize,
    pub column: usize,
    pub context: String,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TextSpan {
    pub start: usize,
    pub end: usize,
}
