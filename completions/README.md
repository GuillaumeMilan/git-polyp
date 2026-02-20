# Shell Completions for git-polyp

This directory contains shell completion scripts for `git-polyp` that enable tab completion for commands, flags, and git branch names.

## Features

- ✅ Command completion (`rebase-stack`)
- ✅ Flag completion (`--help`, `--version`, `--continue`, `--abort`)
- ✅ Dynamic git branch name completion
- ✅ Context-aware suggestions
- ✅ Fast performance (no subprocess overhead)

## Quick Installation

### Automated Installation

Run the install script to automatically set up completions for your shell:

```bash
cd completions
./install.sh
```

The script will auto-detect your shell and install the appropriate completion file.

You can also specify the shell explicitly:

```bash
./install.sh bash   # Install for Bash
./install.sh zsh    # Install for Zsh
./install.sh fish   # Install for Fish
```

### Manual Installation

Choose the instructions for your shell:

#### Bash

**Option 1: System-wide installation (requires sudo)**

```bash
sudo cp git-polyp.bash /usr/local/etc/bash_completion.d/git-polyp
```

Then restart your shell or run:
```bash
source /usr/local/etc/bash_completion.d/git-polyp
```

**Option 2: User installation**

Add this line to your `~/.bashrc` or `~/.bash_profile`:

```bash
source /path/to/git_polyp/completions/git-polyp.bash
```

Then reload your configuration:
```bash
source ~/.bashrc
```

#### Zsh

**Option 1: Using fpath**

1. Copy the completion file with the correct name:
```bash
mkdir -p ~/.zsh/completions
cp git-polyp.zsh ~/.zsh/completions/_git-polyp
```

2. Add to your `~/.zshrc` (if not already present):
```zsh
fpath=(~/.zsh/completions $fpath)
autoload -Uz compinit && compinit
```

3. Restart your shell or run:
```bash
exec zsh
```

**Option 2: System-wide installation (requires sudo)**

```bash
sudo cp git-polyp.zsh /usr/local/share/zsh/site-functions/_git-polyp
```

Then run:
```bash
compinit
```

#### Fish

Copy the completion file to Fish's completions directory:

```bash
cp git-polyp.fish ~/.config/fish/completions/
```

Completions will be available immediately in new Fish shell sessions.

## Usage Examples

Once installed, you can use tab completion with `git-polyp`:

### Command completion
```bash
$ git-polyp <TAB>
rebase-stack  --help  --version
```

### Flag completion
```bash
$ git-polyp rebase-stack --<TAB>
--continue  --abort  --help
```

### Branch name completion
```bash
$ git-polyp rebase-stack <TAB>
main  develop  feature-1  feature-2  bugfix-login
```

### Context-aware completion
```bash
$ git-polyp rebase-stack main <TAB>
# Suggests branches (excluding 'main' if you have smart completion)
feature-1  feature-2  feature-3
```

## Troubleshooting

### Completions not working in Bash

1. Verify bash-completion is installed:
   - macOS: `brew install bash-completion@2`
   - Ubuntu/Debian: `sudo apt install bash-completion`

2. Ensure the completion file is sourced:
   ```bash
   type _git_polyp_completion
   ```
   If this shows "not found", the completion file isn't loaded.

### Completions not working in Zsh

1. Ensure `compinit` is called in your `~/.zshrc`:
   ```zsh
   autoload -Uz compinit && compinit
   ```

2. Check if the completion file is in your fpath:
   ```zsh
   echo $fpath
   ```

3. Rebuild completion cache:
   ```zsh
   rm -f ~/.zcompdump && compinit
   ```

### Completions not working in Fish

1. Verify the file is in the right location:
   ```bash
   ls ~/.config/fish/completions/git-polyp.fish
   ```

2. Restart Fish or run:
   ```fish
   fish_update_completions
   ```

### Branch names not appearing

Make sure you're in a git repository:
```bash
git branch
```

The completion scripts use `git branch` to fetch branch names, so this command must work.

## How It Works

The completion scripts use a hybrid approach:

1. **Static completions**: Commands and flags are defined directly in the shell scripts
2. **Dynamic completions**: Branch names are fetched in real-time using `git branch`
3. **Context-aware**: The scripts analyze the command line to provide relevant suggestions

This approach ensures:
- ⚡ Fast performance (no escript startup overhead)
- 🔄 Always up-to-date branch names
- 🎯 Relevant suggestions based on context

## Development

If you're adding new commands or flags to `git-polyp`, you'll need to update the completion scripts:

1. **git-polyp.bash**: Update the `commands` or `*_flags` variables
2. **git-polyp.zsh**: Update the `commands` or `*_flags` arrays
3. **git-polyp.fish**: Add new `complete` definitions

## License

These completion scripts are part of the git-polyp project and follow the same license.
