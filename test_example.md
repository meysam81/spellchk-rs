# Test Document

This is a test document with some misspellings.

## Features

- Spellchecking is important
- We should check all files
- Grammar matters too

Here is a code block that should be ignored:

```rust
fn main() {
    let misspelled = "this should not be checked";
    println!("{}", misspelled);
}
```

And some `inline_code_here` that should also be ignored.

But regular text like "receive" and "occurred" should be caught.

The URL https://example.com/something should mostly be ignored.
