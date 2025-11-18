# spellchk 🚀

A blazingly fast, production-ready spellchecker CLI built with Rust. Check spelling in any text file with intelligent file-type-specific parsing, automatic fixes, and on-demand dictionary updates.

## Features

✨ **Blazingly Fast** - Built with Rust and FST (Finite State Transducer) for ultra-fast dictionary lookups
📝 **Smart File Parsing** - Intelligently handles Markdown, source code, and plain text
🎨 **Beautiful Output** - Colored diffs with context and suggestions
🔧 **Auto-Fix Mode** - Automatically fix misspellings or use interactive mode
🌍 **Multiple Languages** - Support for English (US/GB) with easy extensibility
⚙️ **Configurable** - Global and per-project configuration files
🔌 **Shell Completion** - Built-in completion for bash, zsh, fish, and PowerShell
📦 **Dictionary Management** - Download and update dictionaries on demand

## Installation

### From Source

```bash
cargo install --path .
```

### From Cargo (coming soon)

```bash
cargo install spellchk
```

## Quick Start

```bash
# Download a dictionary (first time only)
spellchk dict download en_US

# Check files
spellchk myfile.md another-file.md

# Auto-fix misspellings
spellchk --fix myfile.md

# Interactive fix mode (like git add -p)
spellchk --fix --interactive myfile.md

# Generate shell completions
spellchk --completion bash > ~/.local/share/bash-completion/completions/spellchk
```

## Usage

### Basic Checking

```bash
# Check single file
spellchk document.md

# Check multiple files
spellchk file1.md file2.txt file3.js

# Disable colored output
spellchk --no-color document.md

# JSON output for editor integration
spellchk --format json document.md

# Don't fail on errors (exit code 0)
spellchk --no-fail document.md
```

### Fixing Misspellings

```bash
# Auto-fix with top suggestion
spellchk --fix document.md

# Interactive mode (choose corrections)
spellchk --fix --interactive document.md
```

Interactive mode provides a git-like interface:
```
Misspelling found: 12:5
  This is a mispeled word in context

Suggestions:
  [s] Skip
  [1] misspelled
  [2] dispelled
  [3] misapplied
  [a] Add to dictionary
  [q] Quit

Choice:
```

### Dictionary Management

```bash
# List installed dictionaries
spellchk dict list

# Download a dictionary
spellchk dict download en_US
spellchk dict download en_GB

# Update all dictionaries
spellchk dict update

# Show dictionary info
spellchk dict info en_US
```

### Language Selection

```bash
# Use British English
spellchk --language en_GB document.md

# Or set in config file
```

## Configuration

### Global Configuration

Create `~/.config/spellchk/config.toml`:

```toml
language = "en_US"
personal_dictionary = "~/.config/spellchk/personal.txt"
max_suggestions = 5
case_sensitive = false

# Patterns to ignore (regex)
ignore_patterns = [
    "\\b[A-Z0-9_]{2,}\\b",      # ALL_CAPS constants
    "https?://\\S+",             # URLs
    "\\b[a-fA-F0-9]{32,}\\b",   # Hashes
]

# Enable specific checking rules
enabled_rules = [
    "check-compound",
    "check-rare",
]
```

### Project Configuration

Create `.spellchk.toml` in your project root:

```toml
language = "en_US"
personal_dictionary = "./.spellchk-words.txt"

ignore_patterns = [
    "TODO|FIXME|XXX",  # Code annotations
    "\\bv\\d+\\.\\d+",  # Version numbers
]
```

Project configuration overrides global configuration.

### Personal Dictionary

Add words to your personal dictionary (`~/.config/spellchk/personal.txt`):

```
mycompany
customword
api_endpoint
# Comments are allowed
```

Words are automatically added when using `[a] Add to dictionary` in interactive mode.

## File Type Support

spellchk intelligently handles different file types:

### Markdown (`.md`, `.mdx`, `.markdown`)
- Skips code blocks (\`\`\`)
- Skips inline code (\`)
- Checks text content, headings, lists, etc.

### Source Code (`.rs`, `.js`, `.ts`, `.py`, `.go`, `.java`, `.c`, `.cpp`)
- Checks comments (`//`, `/* */`, `#`)
- Checks string literals
- Ignores code syntax

### Plain Text (`.txt`, and other files)
- Checks all text content
- Smart camelCase/snake_case splitting

## Command-Line Options

```
Usage: spellchk [OPTIONS] [FILES]... [COMMAND]

Arguments:
  [FILES]...  Files to check

Options:
  -f, --fix                     Fix misspellings in place
  -i, --interactive             Interactive mode for selecting corrections
      --no-color                Disable colored output
      --no-fail                 Exit with code 0 even if errors found
  -l, --language <LANGUAGE>     Language/dictionary to use [default: en_US]
  -o, --format <FORMAT>         Output format (text, json) [default: text]
      --ignore-pattern <REGEX>  Pattern to ignore (regex)
      --personal-dict <PATH>    Personal dictionary file
      --completion <SHELL>      Generate shell completion script
  -h, --help                    Print help
  -V, --version                 Print version

Commands:
  dict  Dictionary management
    list      List installed dictionaries
    download  Download a dictionary
    update    Update all dictionaries
    info      Show dictionary info
```

## Shell Completion

Generate completion scripts for your shell:

### Bash
```bash
spellchk --completion bash > ~/.local/share/bash-completion/completions/spellchk
```

### Zsh
```bash
spellchk --completion zsh > ~/.zsh/completion/_spellchk
```

### Fish
```bash
spellchk --completion fish > ~/.config/fish/completions/spellchk.fish
```

### PowerShell
```powershell
spellchk --completion powershell > spellchk.ps1
```

## Architecture

spellchk is built with performance and reliability in mind:

- **FST-based dictionary** - Finite State Transducer for ultra-fast lookups (10-100x faster than HashMap)
- **Parallel processing** - Process multiple files concurrently with Rayon
- **Smart caching** - LRU cache for suggestions, memory-mapped dictionary files
- **Intelligent parsing** - File-type-specific tokenization and text extraction
- **Edit distance algorithm** - Levenshtein distance for accurate suggestions

## Development

### Building from Source

```bash
git clone https://github.com/meysam81/spellchk-rs.git
cd spellchk-rs
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Project Structure

```
spellchk-rs/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs            # Library interface
│   ├── cli/              # CLI output & formatting
│   ├── checker/          # Core spellcheck logic
│   │   ├── dictionary.rs # FST-based dictionary
│   │   └── suggestions.rs # Suggestion generation
│   ├── dict/             # Dictionary management
│   ├── parser/           # File type parsers
│   │   ├── markdown.rs
│   │   ├── source_code.rs
│   │   └── plaintext.rs
│   └── config.rs         # Configuration handling
└── Cargo.toml
```

## Performance

spellchk is designed to be blazingly fast:

- **Dictionary lookup**: O(word length) with FST
- **File processing**: Parallel with Rayon
- **Memory efficient**: Memory-mapped files, zero-copy parsing
- **Optimized build**: LTO enabled, single codegen unit for release builds

Typical performance:
- Check 1000-line file: ~10ms
- Generate suggestions: ~1ms per word
- Dictionary load: ~50ms (cached)

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- Dictionary data from [SCOWL](http://wordlist.aspell.net/) and [dwyl/english-words](https://github.com/dwyl/english-words)
- Inspired by tools like `aspell`, `hunspell`, and `codespell`
- Built with amazing Rust crates: `clap`, `fst`, `rayon`, `pulldown-cmark`

---

**Made with ❤️ and Rust** 🦀
