use crate::CheckResult;
use colored::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Text,
    Json,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Text => write!(f, "text"),
            OutputFormat::Json => write!(f, "json"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonError {
    file: String,
    line: usize,
    column: usize,
    word: String,
    suggestions: Vec<String>,
    context: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonOutput {
    files_checked: usize,
    total_errors: usize,
    errors: Vec<JsonError>,
}

pub fn print_errors(
    file_path: &Path,
    result: &CheckResult,
    colored_output: bool,
    format: &OutputFormat,
) {
    match format {
        OutputFormat::Text => print_text_errors(file_path, result, colored_output),
        OutputFormat::Json => print_json_errors(file_path, result),
    }
}

fn print_text_errors(file_path: &Path, result: &CheckResult, colored_output: bool) {
    if result.errors.is_empty() {
        return;
    }

    let file_name = file_path.display().to_string();

    if colored_output {
        println!("\n{}", file_name.bold().underline());
    } else {
        println!("\n{}", file_name);
    }

    for error in &result.errors {
        let line_info = format!("{}:{}", error.line, error.column);

        if colored_output {
            println!(
                "  {} {} {}",
                line_info.blue().bold(),
                error.word.red().bold(),
                format_context(&error.context, &error.word, colored_output)
            );

            if !error.suggestions.is_empty() {
                let suggestions = error.suggestions
                    .iter()
                    .take(5)
                    .map(|s| s.green().to_string())
                    .collect::<Vec<_>>()
                    .join(&", ".dimmed().to_string());
                println!("    {} {}", "→".dimmed(), suggestions);
            }
        } else {
            println!("  {} {} {}", line_info, error.word, &error.context);

            if !error.suggestions.is_empty() {
                let suggestions = error.suggestions
                    .iter()
                    .take(5)
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("    → {}", suggestions);
            }
        }
    }
}

fn print_json_errors(file_path: &Path, result: &CheckResult) {
    let json_errors: Vec<JsonError> = result
        .errors
        .iter()
        .map(|e| JsonError {
            file: file_path.display().to_string(),
            line: e.line,
            column: e.column,
            word: e.word.clone(),
            suggestions: e.suggestions.clone(),
            context: e.context.clone(),
        })
        .collect();

    let output = JsonOutput {
        files_checked: 1,
        total_errors: result.error_count,
        errors: json_errors,
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

fn format_context(context: &str, word: &str, colored: bool) -> String {
    if colored {
        context.replace(word, &word.red().bold().to_string())
    } else {
        context.to_string()
    }
}

pub fn print_check_summary(total_errors: usize, files: &[impl AsRef<Path>], colored: bool) {
    println!();
    if total_errors == 0 {
        if colored {
            println!("{}", "✓ No spelling errors found!".green().bold());
        } else {
            println!("✓ No spelling errors found!");
        }
    } else {
        let error_word = if total_errors == 1 { "error" } else { "errors" };
        if colored {
            println!(
                "{} {} {} found in {} {}",
                "✗".red().bold(),
                total_errors.to_string().red().bold(),
                error_word,
                files.len(),
                if files.len() == 1 { "file" } else { "files" }
            );
        } else {
            println!(
                "✗ {} {} found in {} {}",
                total_errors,
                error_word,
                files.len(),
                if files.len() == 1 { "file" } else { "files" }
            );
        }
    }
}

pub fn print_fix_summary(total_fixed: usize, files: &[impl AsRef<Path>], colored: bool) {
    println!();
    if total_fixed == 0 {
        if colored {
            println!("{}", "No corrections needed!".green().bold());
        } else {
            println!("No corrections needed!");
        }
    } else {
        let fix_word = if total_fixed == 1 { "correction" } else { "corrections" };
        if colored {
            println!(
                "{} {} {} applied to {} {}",
                "✓".green().bold(),
                total_fixed.to_string().green().bold(),
                fix_word,
                files.len(),
                if files.len() == 1 { "file" } else { "files" }
            );
        } else {
            println!(
                "✓ {} {} applied to {} {}",
                total_fixed,
                fix_word,
                files.len(),
                if files.len() == 1 { "file" } else { "files" }
            );
        }
    }
}

pub fn print_interactive_prompt(
    word: &str,
    suggestions: &[String],
    context: &str,
    line: usize,
    column: usize,
    colored: bool,
) -> Option<String> {
    if colored {
        println!(
            "\n{} {}:{}",
            "Misspelling found:".yellow().bold(),
            line.to_string().blue(),
            column.to_string().blue()
        );
        println!("  {}", format_context(context, word, colored));
        println!("\n{}", "Suggestions:".cyan().bold());
    } else {
        println!("\nMisspelling found: {}:{}", line, column);
        println!("  {}", context);
        println!("\nSuggestions:");
    }

    let mut options = vec!["[s] Skip".to_string()];
    for (i, suggestion) in suggestions.iter().take(9).enumerate() {
        if colored {
            options.push(format!("[{}] {}", i + 1, suggestion.green()));
        } else {
            options.push(format!("[{}] {}", i + 1, suggestion));
        }
    }
    options.push("[a] Add to dictionary".to_string());
    options.push("[q] Quit".to_string());

    for option in &options {
        println!("  {}", option);
    }

    print!("\nChoice: ");
    use std::io::{self, Write};
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    let input = input.trim();

    match input {
        "s" | "S" => None,
        "a" | "A" => Some(word.to_string()), // Will be added to personal dict
        "q" | "Q" => std::process::exit(0),
        num => {
            if let Ok(idx) = num.parse::<usize>() {
                if idx > 0 && idx <= suggestions.len() {
                    return Some(suggestions[idx - 1].clone());
                }
            }
            None
        }
    }
}
