# macup

A thin orchestrator for Mac bootstrap and setup. Declaratively configure your macOS setup with Homebrew, npm, cargo, custom scripts, and system settings.

## Features

- 🍺 **Homebrew**: Install formulae, casks, and taps
- 📱 **Mac App Store**: Install apps via mas-cli
- 📦 **Package Managers**: Support for npm, cargo, pip, gem
- 🔧 **Custom Scripts**: Run curl installers (rustup, oh-my-zsh, etc.)
- ⚙️ **System Settings**: Apply macOS defaults and configurations
- 🚀 **Parallel Installation**: Install packages concurrently for speed
- ✅ **Idempotent**: Safe to run multiple times, only installs what's missing
- 🎯 **Dependency Resolution**: Automatic execution order based on dependencies
- ➕ **Easy Adding**: `macup add npm pnpm` to install and save to config
- 📥 **Import Existing Setup**: `macup import` to scan and import currently installed packages
- 🤖 **Auto-Install**: Automatically installs required managers and runtimes (Homebrew, mas-cli, Node.js, Rust, Python, Ruby)
- 🔄 **Error Recovery**: Continue on failures and retry with idempotent re-runs
- 🔌 **Extensible**: Easily add new package managers with code generation

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
macup apply                            # Install packages only (skip system settings)
macup apply --dry-run                  # Preview changes without applying
macup apply --with-system-settings     # Install packages AND apply system settings
```

**Note:** System settings (macOS defaults commands) are **skipped by default** and only run when you explicitly use `--with-system-settings`. This prevents accidentally modifying system preferences on every run.

### Add packages dynamically

```bash
# Add and install packages
macup add brew ripgrep bat eza
macup add cask ghostty arc
macup add npm pnpm typescript
macup add cargo tokei sd
macup add pip requests flask
macup add gem bundler rails

# Only add to config, skip install
macup add npm eslint --no-install
```

When you use `macup add`:
1. Packages are installed first
2. Only successfully installed packages are saved to config
3. Config file is updated automatically

**Supported managers**: `brew`, `cask`, `mas`, `npm`, `cargo`, `pip`, `gem`

### Import existing packages

Already have tools installed? Import them into your config:

```bash
macup import
```

This will:
1. 🔍 Scan your system for installed packages (Homebrew, npm, cargo, MAS, pipx)
2. ✅ Mark packages already in your config
3. 🎯 Show interactive selection (use Space to toggle, Enter to confirm)
4. 👀 Preview changes before writing
5. 📝 Merge selected packages into your `macup.toml`

**Example workflow:**
```bash
# You have tons of brew packages installed
# Import them to track in config
macup import

# Interactive UI shows:
# 🍺 neovim
# 🍺 ripgrep [existing]  ← Already in config
# 📦 visual-studio-code
# 🦀 cargo-edit
# ...

# Select packages with Space, confirm with Enter
# Preview shows what will be added
# Confirm and done!

# Verify
macup diff
```

**Supported managers:**
- 🍺 Homebrew (formulae + casks)
- 📦 npm global packages
- 🦀 Cargo packages
- 📱 Mac App Store apps (with IDs)
- 🐍 pipx packages

### Check differences

```bash
macup diff    # Show what's missing or changed
```

Shows installed vs missing packages for all configured managers:

```
🍺 Homebrew Formulae
  ✓ git
  ✓ neovim
  ❌ ripgrep      ← Not installed yet
  Summary: 2/3

📦 Homebrew Casks
  ✓ visual-studio-code
  ❌ iterm2       ← Not installed yet
  Summary: 1/2

Overall Summary
  ✓ Installed: 3
  ❌ Missing: 2

Run 'macup apply' to install missing packages.
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
- `[pip]` section with packages → auto-installs Python if missing
- `[gem]` section with packages → auto-installs Ruby if missing

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
packages = ["pnpm", "typescript", "eslint"]

[cargo]
# Rust will be auto-installed via brew if needed
packages = ["ripgrep", "bat", "fd-find"]

[pip]
# Python will be auto-installed via brew if needed
packages = ["requests", "flask", "black"]

[gem]
# Ruby will be auto-installed via brew if needed
packages = ["bundler", "rails", "jekyll"]

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
- `packages`: npm global packages

#### `[cargo]`
Requires Rust (auto-installed via brew if needed, or uses existing rustup)
- `packages`: Cargo packages

#### `[pip]`
Requires Python (auto-installed via brew if needed, or uses system Python)
- `packages`: Python packages (installed with pip)

#### `[gem]`
Requires Ruby (auto-installed via brew if needed, or uses system Ruby)
- `packages`: Ruby gems

#### `[[install.scripts]]`
For custom curl installers:
- `name`: Script identifier
- `check`: Command to check if already installed (optional)
- `command`: Install command
- `required`: If false, continues on error (default: true)

#### `[system]`
- `commands`: Array of shell commands (defaults, killall, etc.)
- Executed sequentially after all packages are installed
- **Only runs when `--with-system-settings` flag is provided**

## How It Works

### Execution Flow

1. **Parse & Validate Config**: Load TOML and check for dependency cycles
2. **Pre-flight Checks**: Verify Homebrew is installed (foundation requirement)
3. **Build Execution Plan**: Topological sort based on `depends_on`
4. **Install Packages by Section**: Each section installs its packages in parallel
   - Brew: Install formulae/casks
   - Mas: Auto-install mas-cli if needed, then install apps
   - Npm: Auto-install Node.js if needed, then install packages
   - Cargo: Auto-install Rust if needed, then install packages
   - Pip: Auto-install Python if needed, then install packages
   - Gem: Auto-install Ruby if needed, then install packages
5. **Run Install Scripts**: Sequential, with idempotency checks
6. **Apply System Settings** (optional): Execute commands sequentially
   - Only runs with `--with-system-settings` flag
   - Skipped by default to avoid unintended system changes

### Idempotency

macup checks before installing:
- **Brew**: `brew list --formula` / `brew list --cask`
- **mas**: `mas list`
- **npm**: `npm list -g`
- **cargo**: `cargo install --list`
- **pip**: `pip list`
- **gem**: `gem list`
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

## Developer Guide

### Adding New Package Managers

macup makes it easy to add support for new package managers using code generation. All the boilerplate is generated automatically!

#### Quick Start: Add a New Manager

```bash
# Create a new package manager
./macup new manager <name> \
  --display "Display Name" \
  --icon "🎨" \
  --runtime-cmd "command-name" \
  --runtime-name "Runtime Name" \
  --brew-formula "brew-formula-name"

# Example: Add support for pipx (Python CLI tools)
./macup new manager pipx \
  --display "Python CLI Apps" \
  --icon "🐍" \
  --runtime-cmd "pipx" \
  --runtime-name "pipx" \
  --brew-formula "pipx"
```

This generates:
- ✅ Manager implementation template in `src/managers/<name>.rs`
- ✅ Config schema (TOML section support)
- ✅ Registry entry with metadata
- ✅ Integration with `macup add` command
- ✅ Handler function for installation
- ✅ All required boilerplate code

#### What Gets Generated

After running `macup new manager pipx`, you'll have:

1. **Manager Implementation** (`src/managers/pipx.rs`):
   ```rust
   pub struct PipxManager {
       max_parallel: usize,
   }
   
   impl Manager for PipxManager {
       fn name(&self) -> &str { "pipx" }
       fn install_packages(&self, packages: &[String]) -> Result<InstallResult> {
           // TODO: Implement your installation logic
       }
       // ... other methods with TODOs
   }
   ```

2. **Config Support** - Users can now add to `macup.toml`:
   ```toml
   [pipx]
   packages = ["poetry", "black", "ruff"]
   ```

3. **CLI Integration** - `macup add` now supports your manager:
   ```bash
   macup add pipx poetry black ruff
   ```

4. **Auto-Installation** - Runtime auto-installs via Homebrew if missing

#### Implementation Steps

1. **Generate the manager**:
   ```bash
   ./macup new manager pipx --display "Python CLI Apps" \
     --icon "🐍" --runtime-cmd "pipx" --runtime-name "pipx" \
     --brew-formula "pipx"
   ```

2. **Implement the Manager trait** in `src/managers/pipx.rs`:
   - `list_installed()` - Query currently installed packages
   - `is_package_installed()` - Check if a specific package exists
   - `install_package()` - Install a single package
   - `install_packages()` - Already implemented with parallel support

3. **Build and test**:
   ```bash
   cargo build
   ./macup add pipx poetry
   ./macup apply
   ```

4. **Commit your changes**:
   ```bash
   git add .
   git commit -m "Add pipx package manager support"
   ```

#### Example: Implementing list_installed()

```rust
fn list_installed(&self) -> Result<HashSet<String>> {
    let output = Command::new("pipx")
        .args(&["list", "--short"])
        .output()
        .context("Failed to list pipx packages")?;

    if !output.status.success() {
        anyhow::bail!("Failed to list pipx packages");
    }

    let installed = String::from_utf8(output.stdout)?
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    Ok(installed)
}
```

#### Removing a Manager

If you need to remove a manager:

```bash
./macup remove manager <name>

# Example
./macup remove manager pipx
```

This removes all generated code:
- ✅ Manager implementation file
- ✅ Config schema entries
- ✅ Registry entry
- ✅ CLI integration
- ✅ All boilerplate code

The project will still compile after removal!

#### Manager Requirements

For a package manager to work with macup, implement:

1. **Check if installed**: `is_installed()` - Check if the manager's CLI exists
2. **List packages**: `list_installed()` - Get currently installed packages
3. **Install package**: `install_package()` - Install a single package
4. **Check single package**: `is_package_installed()` - Verify if specific package is installed

The parallel installation logic is handled automatically by the base implementation.

#### Code Generation Architecture

macup uses a marker-based code generation system:

- **CODEGEN_MARKER** comments mark insertion points
- **CODEGEN_START/END** pairs wrap generated code
- `macup new manager` inserts code at markers
- `macup remove manager` removes code between START/END pairs
- Indent-aware generation preserves code formatting

Example markers:
```rust
// CODEGEN_START[pipx]: manager_metadata
ManagerMetadata { ... },
// CODEGEN_END[pipx]: manager_metadata
// CODEGEN_MARKER: insert_manager_metadata_here
```

This allows you to safely add/remove managers without manual code editing!

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
