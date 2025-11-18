use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use spellchk::{checker, cli, dict, Config};
use spellchk::cli::output::OutputFormat;
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "spellchk")]
#[command(version, about = "A blazingly fast spellchecker CLI", long_about = None)]
struct Cli {
    /// Files to check
    #[arg(value_name = "FILES")]
    files: Vec<PathBuf>,

    /// Fix misspellings in place (auto-apply top suggestion)
    #[arg(short, long)]
    fix: bool,

    /// Interactive mode for selecting corrections
    #[arg(short, long, requires = "fix")]
    interactive: bool,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,

    /// Exit with code 0 even if errors are found
    #[arg(long)]
    no_fail: bool,

    /// Language/dictionary to use (e.g., en_US, en_GB)
    #[arg(short, long, default_value = "en_US")]
    language: String,

    /// Output format (text, json)
    #[arg(short = 'o', long, default_value = "text")]
    format: OutputFormat,

    /// Add words to personal dictionary
    #[arg(long)]
    add_to_dict: Vec<String>,

    /// Pattern to ignore (regex)
    #[arg(long)]
    ignore_pattern: Vec<String>,

    /// Personal dictionary file
    #[arg(long)]
    personal_dict: Option<PathBuf>,

    /// Generate shell completion script
    #[arg(long, value_name = "SHELL")]
    completion: Option<Shell>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Dictionary management
    Dict {
        #[command(subcommand)]
        action: DictCommands,
    },
}

#[derive(Parser, Debug)]
enum DictCommands {
    /// List installed dictionaries
    List,
    /// Download a dictionary
    Download {
        /// Language code (e.g., en_US, en_GB, fr_FR)
        language: String,
    },
    /// Update all dictionaries
    Update,
    /// Show dictionary info
    Info {
        /// Language code
        language: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle shell completion generation
    if let Some(shell) = cli.completion {
        let mut cmd = Cli::command();
        generate(shell, &mut cmd, "spellchk", &mut io::stdout());
        return Ok(());
    }

    // Handle subcommands
    if let Some(command) = cli.command {
        return handle_command(command);
    }

    // Load configuration
    let config = Config::load(
        cli.language.clone(),
        cli.personal_dict.clone(),
        cli.ignore_pattern.clone(),
    )?;

    // Validate input files
    if cli.files.is_empty() {
        anyhow::bail!("No files specified. Use --help for usage information.");
    }

    // Initialize checker
    let checker = checker::SpellChecker::new(&config)?;

    // Process files
    let mut total_errors = 0;
    let mut total_fixed = 0;

    for file_path in &cli.files {
        if !file_path.exists() {
            eprintln!("Error: File not found: {}", file_path.display());
            continue;
        }

        let result = if cli.fix {
            if cli.interactive {
                checker.fix_interactive(file_path, &config, !cli.no_color)?
            } else {
                checker.fix_auto(file_path, &config, !cli.no_color)?
            }
        } else {
            checker.check(file_path, &config, !cli.no_color, &cli.format)?
        };

        total_errors += result.error_count;
        total_fixed += result.fixed_count;
    }

    // Print summary
    if cli.fix {
        cli::output::print_fix_summary(total_fixed, &cli.files, !cli.no_color);
    } else {
        cli::output::print_check_summary(total_errors, &cli.files, !cli.no_color);
    }

    // Exit with appropriate code
    if total_errors > 0 && !cli.no_fail && !cli.fix {
        std::process::exit(1);
    }

    Ok(())
}

fn handle_command(command: Commands) -> Result<()> {
    match command {
        Commands::Dict { action } => match action {
            DictCommands::List => {
                dict::manager::list_dictionaries()?;
            }
            DictCommands::Download { language } => {
                dict::manager::download_dictionary(&language)?;
            }
            DictCommands::Update => {
                dict::manager::update_dictionaries()?;
            }
            DictCommands::Info { language } => {
                dict::manager::show_info(&language)?;
            }
        },
    }
    Ok(())
}
