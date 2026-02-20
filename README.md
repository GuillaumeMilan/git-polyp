# GitPolyp 🐙

GitPolyp is a high-performance CLI toolkit written in Rust for advanced git automation and tooling. It provides powerful commands to streamline complex git workflows and repository management, with a focus on speed, safety, and reliability.

## Installation

### Build from source

```bash
# Clone the repository
git clone https://github.com/yourusername/git-polyp.git
cd git-polyp

# Build the release binary
cargo build --release

# Optional: Install globally
cp target/release/git-polyp /usr/local/bin/

# Or use cargo install
cargo install --path .
```

### Pre-built binaries

Download the latest release from the [releases page](https://github.com/yourusername/git-polyp/releases).

## Commands

### `rebase-stack` - Rebase a linear stack of branches

Automates rebasing a linear stack of local Git branches onto a specified upstream branch, handling conflicts, and allowing you to resume or abort the process.

#### Usage

**Start a rebase:**
```bash
git-polyp rebase-stack <upstream-branch> [target-branch]

# With optional base specification
git-polyp rebase-stack --base <base-branch> <upstream-branch> [target-branch]
```

**Continue after resolving conflicts:**
```bash
git-polyp rebase-stack --continue
```

**Abort the rebase:**
```bash
git-polyp rebase-stack --abort
```

**Undo the rebase (restore to original state):**
```bash
git-polyp rebase-stack --undo
```

#### Options

- `--base <BASE>`: Specify a custom base commit/branch to rebase from
- `--abort`: Abort the current rebase operation and clean up
- `--undo`: Restore the repository to its state before the rebase began
- `--continue`: Continue a rebase operation after resolving conflicts

#### Example Workflow

Suppose you have a stack of feature branches:
```
main
  └─ feature-1
      └─ feature-2
          └─ feature-3
```

To rebase all three features onto an updated `main`:

```bash
# Rebase the entire stack
git-polyp rebase-stack main feature-3

# The tool will show a preview:
#   Stack to rebase:
#   1. [abc1234] feature-1: Add authentication
#   2. [def5678] feature-2: Add user profile  
#   3. [ghi9012] feature-3: Add settings page
#
#   Do you want to rebase this stack? [y/N]

# Confirm the rebase by typing 'y'

# If conflicts occur:
# 1. Resolve conflicts manually
# 2. Stage changes: git add .
# 3. Continue git rebase: git rebase --continue  
# 4. Resume stack update: git-polyp rebase-stack --continue

# After successful rebase, push branches:
git push --force-with-lease origin feature-1
git push --force-with-lease origin feature-2
git push --force-with-lease origin feature-3
```

#### Advanced Usage

**Using a custom base:**
```bash
# Rebase only commits after a specific point
git-polyp rebase-stack --base feature-1 main feature-3
```

**Error recovery:**
```bash
# If something goes wrong, restore original state
git-polyp rebase-stack --undo

# Or just abort and clean up
git-polyp rebase-stack --abort
```

#### How it works

1. **Validates environment**: Ensures you're in a git repository and checks for existing operations
2. **Identifies the stack**: Finds all commits and local branches between base/upstream and target
3. **Shows preview**: Displays the stack with commit messages and asks for confirmation
4. **Performs rebase**: Executes optimized git operations to rebase the entire stack
5. **Updates branches**: Automatically updates all branch pointers to new commits
6. **Provides guidance**: Shows commands to push rebased branches or offers to push automatically

#### Features

- ✅ **Fast and reliable**: Written in Rust for optimal performance
- ✅ **Safe operations**: Comprehensive error handling and state management
- ✅ **Resume capability**: Continue operations after resolving conflicts
- ✅ **Undo support**: Restore repository to original state before any changes
- ✅ **Interactive confirmations**: Clear prompts before destructive operations  
- ✅ **Colored output**: Easy-to-read, color-coded terminal output
- ✅ **State persistence**: Metadata stored in `.git/polyp/` for resuming operations
- ✅ **Conflict resolution**: Seamless workflow for handling merge conflicts

#### Limitations

- Supports linear history only (no merge commits in the stack)
- Manages local branches only  
- Requires interactive confirmation (no silent/batch mode yet)

### `unstack` - Split stacked branches *(Coming Soon)*

```bash
git-polyp unstack <from-branch>
```

Split a stack of commits into separate branches. Currently in development.

## Development

### Prerequisites

- Rust 1.70+ (2024 edition)
- Git 2.0+

### Building

```bash
# Development build
cargo build

# Release build  
cargo build --release

# Run tests
cargo test

# Check code
cargo check
```

### Code Structure

```
src/
├── main.rs              # CLI entry point and argument parsing
├── commands.rs          # Command definitions and routing
├── client.rs            # Git command execution and repository interface
├── io.rs               # User input/output utilities  
├── stack.rs            # Core stack manipulation logic
└── commands/
    └── rebase_stack/
        ├── mod.rs      # Main rebase-stack command implementation
        ├── messages.rs # User-facing messages and formatting
        └── stack.rs    # Stack data structures and persistence
```

### Dependencies

- `clap`: Command-line argument parsing with derives
- `colored`: Terminal output coloring
- `serde` + `serde_json`: Serialization for state persistence

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure `cargo test` and `cargo check` pass
6. Submit a pull request

### Code Style

This project follows standard Rust conventions:
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Follow the existing code organization patterns

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by advanced Git workflows and the need for better stack management
- Built with the Rust ecosystem's excellent tooling and libraries