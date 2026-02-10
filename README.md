# macup

A thin orchestrator for Mac bootstrap and setup. Declaratively configure your macOS setup with Homebrew, npm, cargo, custom scripts, and system settings.

## Features

- 🍺 **Homebrew**: Install formulae, casks, and taps
- 📱 **Mac App Store**: Install apps via mas-cli
- 📦 **Package Managers**: Support for npm, cargo
- 🔧 **Custom Scripts**: Run curl installers (rustup, oh-my-zsh, etc.)
- ⚙️ **System Settings**: Apply macOS defaults and configurations
- 🚀 **Parallel Installation**: Install packages concurrently for speed
- ✅ **Idempotent**: Safe to run multiple times, only installs what's missing
- 🎯 **Dependency Resolution**: Automatic execution order based on dependencies
- ➕ **Easy Adding**: `macup add npm pnpm` to install and save to config
- 🤖 **Auto-Install**: Automatically installs required managers and runtimes (Homebrew, mas-cli, Node.js, Rust)
- 🔄 **Error Recovery**: Continue on failures and retry with idempotent re-runs

## Quick Start

### Option 1: Use Pre-built Binary (Fastest) ⚡

```bash
# 1. Clone repo
git clone https://github.com/yourusername/macup.git
cd macup

# 2. Run directly (no build needed!)
./macup apply
```

**That's it!** macup will:
- ✅ Auto-install Homebrew if not present
- ✅ Install all packages from config
- ✅ Apply system settings

> **Note:** Pre-built binary is for macOS Apple Silicon (M1/M2/M3). For Intel Macs, use Option 2.

### Option 2: Build from Source

```bash
# 1. Clone repo
git clone https://github.com/yourusername/macup.git
cd macup

# 2. Run bootstrap script
./bootstrap.sh
```

This will:
- Auto-install Homebrew if missing
- Install Rust if needed
- Build macup from source
- Install binary to `~/.cargo/bin/macup`

### 3. Customize your config (optional)

```bash
vim macup.toml
```

Customize the example config with your preferred tools and apps.

### 4. Preview what will be installed (optional)

```bash
./macup apply --dry-run
```

### 5. Apply your setup

```bash
macup apply
```

## Usage

### Apply full configuration

```bash
macup apply                # Install everything from config
macup apply --dry-run      # Preview changes without applying
```

### Add packages dynamically

```bash
# Add and install packages
macup add brew ripgrep bat eza
macup add cask ghostty arc
macup add npm pnpm typescript
macup add cargo tokei sd

# Only add to config, skip install
macup add npm eslint --no-install
```

When you use `macup add`:
1. Packages are installed first
2. Only successfully installed packages are saved to config
3. Config file is updated automatically

### Check differences (future)

```bash
macup diff    # Show what's missing or changed
```

## Configuration

Config file locations (in priority order):
1. `./macup.toml` (current directory)
2. `~/.config/macup/macup.toml`
3. `~/.macup.toml`

Or specify custom location:
```bash
macup apply --config /path/to/config.toml
```

### Automatic Manager Detection

**macup automatically detects which package managers you need** based on your config sections:

- `[brew]` section with packages → auto-installs Homebrew if missing
- `[mas]` section with apps → auto-installs mas-cli if missing  
- `[npm]` section with packages → auto-installs Node.js if missing
- `[cargo]` section with packages → auto-installs Rust if missing

**You don't need to declare managers explicitly!** Just add the packages you want.

### Error Recovery & Retrying

macup continues on errors by default (`fail_fast = false`):

- ✅ If one package fails, others continue installing
- ✅ At the end, shows a summary of all failures
- ✅ Run `macup apply` again after fixing issues
- ✅ Already-installed packages are automatically skipped

Example error recovery workflow:
```bash
# First run - mas installation fails, but npm/cargo continue
$ macup apply
⚠️  macup completed with errors
  ❌ mas (manager installation failed)
     Fix: Try 'brew install mas' manually

# Fix the issue
$ brew install mas

# Re-run - picks up where it left off
$ macup apply
✓ macup apply completed!  # Only installs what was missing
```

### Example Config

```toml
[settings]
fail_fast = false      # Continue on errors (recommended)
max_parallel = 4       # Max concurrent installs

# No [managers] section needed!
# macup auto-detects from the sections below

[brew]
taps = ["homebrew/cask-fonts"]
formulae = ["git", "neovim", "ripgrep", "fd"]
casks = ["visual-studio-code", "iterm2"]

[mas]
# mas-cli will be auto-installed via brew if needed
apps = [
    { name = "Xcode", id = 497799835 },
]

[npm]
# Node.js will be auto-installed via brew if needed
global = ["pnpm", "typescript"]

[cargo]
# Rust will be auto-installed via brew if needed
packages = ["ripgrep", "bat"]

[[install.scripts]]
name = "oh-my-zsh"
check = "test -d ~/.oh-my-zsh"
command = 'sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" "" --unattended'
required = false

[system]
commands = [
    "defaults write com.apple.dock autohide -bool true",
    "killall Dock",
]
```

### Config Sections

#### `[settings]`
- `fail_fast`: Stop on first error (default: false). Set to `true` to halt immediately on any failure.
- `max_parallel`: Max concurrent package installs (default: 4)

#### `[managers]` (Optional)
You typically **don't need this section** - macup auto-detects required managers from your package declarations.

Only use this for explicit control:
- `required`: Force these managers to be installed even if not auto-detected

#### `[brew]`
- `depends_on`: Dependencies (usually empty or can be omitted)
- `taps`: Homebrew taps to add
- `formulae`: CLI tools
- `casks`: GUI applications

#### `[mas]`
Requires mas-cli (auto-installed via brew if needed)
- `apps`: Array of `{name, id}` objects

**Finding app IDs:**
```bash
# Search for an app
mas search Xcode

# Copy the numeric ID
497799835  Xcode (15.0.1)
```

#### `[npm]`
Requires Node.js (auto-installed via brew if needed)
- `global`: Global npm packages

#### `[cargo]`
Requires Rust (auto-installed via brew if needed, or uses existing rustup)
- `packages`: Cargo packages

#### `[[install.scripts]]`
For custom curl installers:
- `name`: Script identifier
- `check`: Command to check if already installed (optional)
- `command`: Install command
- `required`: If false, continues on error (default: true)

#### `[system]`
- `commands`: Array of shell commands (defaults, killall, etc.)
- Executed sequentially after all packages are installed

## How It Works

### Execution Flow

1. **Parse & Validate Config**: Load TOML and check for dependency cycles
2. **Pre-flight Checks**: Verify required managers are installed
3. **Build Execution Plan**: Topological sort based on `depends_on`
4. **Install Managers**: Sequential (brew → mas → npm → cargo)
5. **Install Packages**: Parallel within each manager
6. **Run Install Scripts**: Sequential, with idempotency checks
7. **Apply System Settings**: Execute commands sequentially

### Idempotency

macup checks before installing:
- **Brew**: `brew list --formula` / `brew list --cask`
- **mas**: `mas list`
- **npm**: `npm list -g`
- **cargo**: `cargo install --list`
- **Install scripts**: Custom `check` command

Already-installed packages are skipped automatically.

### Dependency Resolution

Using `depends_on`, macup determines execution order:

```toml
[npm]
depends_on = ["brew"]  # npm runs after brew

[cargo]
depends_on = ["brew"]  # cargo runs after brew
```

Sections without dependencies can run earlier. Circular dependencies are detected and rejected.

## Workflow: Setup New Mac

```bash
# 1. Clone your macup repo
git clone https://github.com/yourusername/macup.git
cd macup

# 2. Bootstrap
./bootstrap.sh

# 3. Apply setup
macup apply

# Done! Your Mac is configured.
```

## Workflow: Add New Tool

```bash
# Discover a new tool
macup add brew bat

# Or add multiple at once
macup add npm pnpm typescript eslint

# Commit changes
git add macup.toml
git commit -m "Add bat, pnpm, typescript, eslint"
git push

# On other machines
git pull
macup apply  # Installs new tools
```

## Advanced

### Verbose logging

```bash
macup apply --verbose
```

### Apply specific section (future feature)

```bash
macup apply brew    # Only install Homebrew packages
macup apply system  # Only apply system settings
```

### Custom config location

```bash
macup apply --config ~/.config/my-mac-setup.toml
```

## Architecture

```
macup/
├── src/
│   ├── cli.rs           # CLI commands (clap)
│   ├── config/          # TOML parsing & validation
│   ├── managers/        # Brew, mas, npm, cargo managers
│   ├── executor/        # Execution planner & applier
│   ├── system/          # System commands executor
│   ├── commands/        # Command implementations (apply, add, diff)
│   └── utils/           # Utilities (command runner, etc.)
├── macup.toml           # Your personal config
├── bootstrap.sh         # Initial setup script
└── README.md
```

## Design Philosophy

- **Thin orchestrator**: Wraps existing tools (brew, mas, npm), doesn't reimplement
- **Declarative config**: Single source of truth in TOML
- **Idempotent**: Safe to run repeatedly
- **Fast**: Parallel installations where possible
- **Practical**: Built for real-world daily use, not academic perfection

## What macup is NOT

- ❌ Not a package manager (it calls brew/npm/cargo)
- ❌ Not a full system state manager (like Nix)
- ❌ Not a dotfiles manager (use chezmoi, stow, etc.)
- ❌ Not a window manager configurator

## Roadmap / Future Ideas

- [ ] `macup diff` - Show drift between config and system
- [ ] `macup remove <manager> <package>` - Uninstall and remove from config
- [ ] `macup doctor` - Health check (brew doctor, etc.)
- [ ] `macup cleanup` - Remove packages not in config
- [ ] Shell completions (bash, zsh, fish)
- [ ] Better error messages and suggestions
- [ ] Progress bars for installations

## Contributing

This is a personal tool, but contributions welcome! Feel free to fork for your own setup.

## License

MIT

## Acknowledgments

Inspired by:
- [Homebrew Bundle](https://github.com/Homebrew/homebrew-bundle)
- [mas-cli](https://github.com/mas-cli/mas)
- [mackup](https://github.com/lra/mackup)
- Nix/Home Manager (concept, not implementation)

---

**Built with Rust 🦀 | For macOS 🍎 | By developers, for developers**
