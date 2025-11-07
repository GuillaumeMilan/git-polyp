# GitPolyp 🐙

GitPolyp is a CLI and Elixir library toolkit for advanced git automation and tooling. It provides powerful commands and programmatic interfaces to streamline complex git workflows and repository management.

## Installation

### Build from source

```bash
# Clone the repository
git clone https://github.com/yourusername/git_polyp.git
cd git_polyp

# Install dependencies
mix deps.get

# Build the executable
mix escript.build

# Optional: Install globally
cp git-polyp /usr/local/bin/
```

### As a library

Add `git_polyp` to your list of dependencies in `mix.exs`:

```elixir
def deps do
  [
    {:git_polyp, "~> 0.1.0"}
  ]
end
```

## Commands

### `rebase-stack` - Rebase a linear stack of branches

Automates rebasing a linear stack of local Git branches onto a specified base branch, handling conflicts, and allowing you to resume or abort the process.

#### Usage

**Start a rebase:**
```bash
git-polyp rebase-stack <base-branch> <target-branch>
```

**Continue after resolving conflicts:**
```bash
git-polyp rebase-stack --continue
```

**Abort the rebase:**
```bash
git-polyp rebase-stack --abort
```

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

# Review the stack and confirm
# The tool will show:
#   1. [abc1234] feature-1
#      Add authentication
#   2. [def5678] feature-2
#      Add user profile
#   3. [ghi9012] feature-3
#      Add settings page

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

#### How it works

1. **Identifies the stack**: Finds all commits and local branches between base and target
2. **Shows preview**: Displays the stack and asks for confirmation
3. **Rebases**: Executes `git rebase --onto` for the entire stack
4. **Updates branches**: Automatically updates all branch pointers to new commits
5. **Provides instructions**: Shows commands to push rebased branches

#### Features

- ✓ Handles linear stacks of branches
- ✓ Preserves all branch pointers through the rebase
- ✓ Supports conflict resolution with resume capability
- ✓ Idempotent operations - safe to retry
- ✓ Clear, color-coded output
- ✓ Metadata persistence for resuming operations

#### Limitations (Current Version)

- Only supports linear history (no merge commits)
- Only manages local branches
- Interactive mode only (no auto-confirm)

## Development

### Running tests

```bash
mix test
```

### Building documentation

```bash
mix docs
```

### Running the CLI locally

```bash
mix escript.build
./git-polyp --help
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[Your chosen license]

