# Adding a New Package Manager

macup makes it easy to add support for new package managers with a single command!

## Quick Start

To add a new package manager (e.g., pip, gem, go), run:

```bash
macup new manager <name> \
  --display "<Display Name>" \
  --icon "<emoji>" \
  --runtime-cmd "<command>" \
  --runtime-name "<runtime>" \
  --brew-formula "<formula>"
```

### Example: Adding pip support

```bash
macup new manager pip \
  --display "pip packages" \
  --icon "🐍" \
  --runtime-cmd "pip3" \
  --runtime-name "python" \
  --brew-formula "python"
```

This single command automatically generates:
- ✅ Registry entry with metadata
- ✅ SectionType enum variant
- ✅ Config struct and PipConfig type
- ✅ PackageManagerSection trait implementation
- ✅ Handler function in apply.rs
- ✅ Manager implementation template in managers/pip.rs
- ✅ Module exports

## What Gets Generated

### 1. Registry Entry (`src/managers/registry.rs`)
```rust
ManagerMetadata {
    name: "pip",
    display_name: "pip packages",
    icon: "🐍",
    runtime_command: "pip3",
    runtime_name: "python",
    brew_formula: "python",
    section_type: SectionType::Pip,
},
```

### 2. Config Support (`src/config/schema.rs`)
```rust
pub pip: Option<PipConfig>,

pub struct PipConfig {
    pub depends_on: Vec<String>,
    pub packages: Vec<String>,
}
```

### 3. Handler Function (`src/executor/apply.rs`)
Auto-generates a complete handler that:
- Checks if runtime is installed
- Auto-installs via brew if missing
- Installs packages with parallel execution
- Tracks failures and errors

### 4. Manager Implementation Template (`src/managers/pip.rs`)
Provides a fully-structured template implementing the `Manager` trait with:
- `list_installed()` - TODO: implement package listing
- `install_package()` - TODO: implement single package install
- `install_packages()` - Complete with parallel execution (ready to use)

## Next Steps After Generation

1. **Implement the Manager trait** in `src/managers/<name>.rs`:
   - Fill in `list_installed()` to query installed packages
   - Fill in `install_package()` to install a single package
   - The parallel `install_packages()` is already implemented!

2. **Build and test**:
   ```bash
   cargo build
   ```

3. **Add to your macup.toml**:
   ```toml
   [pip]
   packages = ["requests", "flask", "pytest"]
   ```

4. **Run macup**:
   ```bash
   macup apply --dry-run  # Test first
   macup apply            # Actually install
   ```

## Example: Complete gem Manager

Here's how you'd add Ruby gems support:

```bash
macup new manager gem \
  --display "Ruby gems" \
  --icon "💎" \
  --runtime-cmd "gem" \
  --runtime-name "ruby" \
  --brew-formula "ruby"
```

Then implement in `src/managers/gem.rs`:

```rust
fn list_installed(&self) -> Result<HashSet<String>> {
    let output = Command::new("gem")
        .args(&["list", "--local"])
        .output()?;
    
    // Parse gem list output
    // Format: "gem-name (version)"
    let gems = String::from_utf8(output.stdout)?
        .lines()
        .filter_map(|line| {
            line.split_whitespace()
                .next()
                .map(|s| s.to_string())
        })
        .collect();
    
    Ok(gems)
}

fn install_package(&self, package: &str) -> Result<()> {
    let status = Command::new("gem")
        .args(&["install", package])
        .status()?;
    
    if !status.success() {
        anyhow::bail!("Failed to install {}", package);
    }
    
    Ok(())
}
```

Done! Your new manager is fully integrated.

## Architecture

The generated code follows macup's architecture:

1. **Registry** - Single source of truth for manager metadata
2. **Config** - TOML parsing and validation
3. **Planner** - Dependency resolution and execution order
4. **Apply** - Actual installation with error handling
5. **Manager** - Package manager-specific logic

## Benefits

- **Consistent**: All managers follow the same pattern
- **Maintainable**: Changes to the pattern only need to happen once
- **Type-safe**: Full Rust type checking
- **Documented**: Generated code includes helpful comments
- **Fast**: Parallel package installation out of the box

## Comparison: Before vs After

### Before (Manual - 8 files to edit)
1. Edit `registry.rs` - add metadata
2. Edit `planner.rs` - add SectionType variant
3. Edit `schema.rs` - add PipConfig struct
4. Edit `schema.rs` - implement PackageManagerSection
5. Edit `schema.rs` - add Config field
6. Edit `schema.rs` - add get_manager_config match arm
7. Edit `apply.rs` - add handler function (~80 lines)
8. Create `managers/pip.rs` - implement Manager (~110 lines)

**Total: ~200 lines across 8 locations**

### After (Automated - 1 command)
```bash
macup new manager pip --display "pip packages" --icon "🐍" \
  --runtime-cmd "pip3" --runtime-name "python" --brew-formula "python"
```

**Then**: Just implement 2 functions in the generated template!

**Reduction: 8 manual steps → 1 command + 2 function implementations**
