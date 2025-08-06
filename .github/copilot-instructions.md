# GitHub Archive Attachments Analyzer (gaaa)

Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.

GitHub Archive Attachments Analyzer (gaaa) is a Rust CLI tool that analyzes GitHub migration archive files to identify large attachments. It helps users optimize archive sizes by finding large image, video, and other attachments that can be removed from issues and pull requests.

## Working Effectively

### Prerequisites
- Rust toolchain is already installed (rustc 1.88.0, cargo 1.88.0)
- No additional setup required

### Build and Test Commands
- **NEVER CANCEL BUILD COMMANDS** - Use timeouts of 90+ seconds for initial builds
- `cargo check` - Check code without producing executables
  - First run: ~40 seconds (downloads dependencies) - NEVER CANCEL
  - Subsequent runs: ~0.5 seconds
- `cargo build` - Build debug binary
  - ~3 seconds after dependencies are downloaded
- `cargo build --release` - Build optimized release binary  
  - ~8 seconds - NEVER CANCEL
- `cargo test` - Run unit tests
  - ~1.2 seconds (runs 3 tests)
- `cargo clippy` - Run linter for code quality warnings
  - ~2 seconds
- `cargo fmt --check` - Check code formatting
  - ~0.5 seconds

### Development Workflow
- **Test your changes**: Always run `cargo run` inside the `fixtures/single-file` or `fixtures/multiple-files` directories to test functionality
- **Build validation**: Run `cargo build` to ensure your changes compile
- **Test validation**: Run `cargo test` to ensure all unit tests pass
- **Code quality**: Run `cargo clippy` and fix any warnings before committing
- **Format check**: Run `cargo fmt --check` to ensure code follows Rust formatting standards

## Validation

### Manual Testing Scenarios
ALWAYS manually test the application after making changes by running these scenarios:

1. **Single file scenario**:
   ```bash
   cd fixtures/single-file
   cargo run
   ```
   Expected output: Should find 1 attachment (todd-trapani-QldMpmrmWuc-unsplash.jpg - 141 KiB)

2. **Multiple files scenario**:
   ```bash
   cd fixtures/multiple-files  
   cargo run
   ```
   Expected output: Should find 2 attachments (both ~141 KiB)

3. **Error handling scenario**:
   ```bash
   cd [repo-root]
   cargo run
   ```
   Expected output: Should display error about missing attachments_000001.json file

### CI Validation Steps
Always run these commands before committing to ensure CI will pass:
- `cargo check`
- `cargo build` 
- `cargo test`
- `cargo clippy` (fix any warnings)
- `cargo fmt --check`

## Repository Structure

### Key Files and Directories
- `src/main.rs` - Single source file containing all application logic
- `Cargo.toml` - Project configuration and dependencies
- `fixtures/` - Test data for development and testing
  - `single-file/` - Test case with one attachment file
  - `multiple-files/` - Test case with multiple attachment files
- `.github/workflows/` - CI/CD pipelines
  - `build.yml` - Runs check, build, and test on every push
  - `release.yml` - Builds release binaries for multiple platforms

### Dependencies
- `byte-unit` - For human-readable file size formatting
- `exitcode` - For standard exit codes
- `glob` - For file pattern matching
- `serde` & `serde_json` - For JSON parsing
- `serde_derive` - For automatic serialization derives

## Application Architecture

### Core Functionality
The application processes GitHub archive attachments by:
1. Reading `attachments_*.json` metadata files
2. Locating corresponding attachment files in `attachments/` directory  
3. Calculating file sizes
4. Sorting by size and displaying results with GitHub links

### Key Functions (src/main.rs)
- `main()` - Entry point, calls process_attachments and handles results
- `process_attachments()` - Main logic for finding and processing attachments
- `read_attachments_files()` - Reads all attachments_*.json files using glob patterns
- `read_attachments_file()` - Parses individual JSON files
- `get_working_directory()` - Determines working directory (current dir or provided path)

### Data Structures
- `Attachment` struct - Represents attachment metadata from JSON files
- Key fields: `asset_name`, `asset_url`, `pull_request`, `issue`, `created_at`

## Common Development Tasks

### Adding New Features
1. Modify `src/main.rs` (single file project)
2. Add unit tests in the `#[cfg(test)]` module
3. Test with fixtures: `cd fixtures/single-file && cargo run`
4. Run full test suite: `cargo test`

### Debugging Issues
- Use `cargo run` in fixtures directories to test specific scenarios
- Add `eprintln!()` statements for debugging output
- Use `cargo check` for quick syntax validation

### Performance Optimization
- Use `cargo build --release` for optimized builds
- Profile with standard Rust tools if needed
- Test with large fixture files if available

## Common Commands Reference

### Quick validation after changes:
```bash
cargo build && cargo test && cd fixtures/single-file && cargo run
```

### Full CI simulation:
```bash
cargo check && cargo build && cargo test && cargo clippy && cargo fmt --check
```

### Development iteration:
```bash
# Make changes to src/main.rs
cargo build                           # Quick compile check
cd fixtures/single-file && cargo run  # Test functionality
cd ../.. && cargo test                # Run unit tests
```

Remember: This is a single-file Rust project, so all logic changes happen in `src/main.rs`. Always test your changes with the fixtures before committing.