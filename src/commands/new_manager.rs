use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

pub fn run(
    name: &str,
    display: &str,
    icon: &str,
    runtime_cmd: &str,
    runtime_name: &str,
    brew_formula: &str,
) -> Result<()> {
    println!("{}", "=".repeat(60).bright_blue());
    println!(
        "{}",
        format!("Creating new package manager: {}", name)
            .bright_blue()
            .bold()
    );
    println!("{}", "=".repeat(60).bright_blue());
    println!();

    let name_capitalized = capitalize(name);

    // Step 1: Add to registry
    println!("{} Adding to registry...", "1.".bold());
    add_to_registry(
        name,
        display,
        icon,
        runtime_cmd,
        runtime_name,
        brew_formula,
        &name_capitalized,
    )?;
    println!("   {} {}", "✓".green(), "src/managers/registry.rs".dimmed());
    println!();

    // Step 2: Add to SectionType enum
    println!("{} Adding to SectionType enum...", "2.".bold());
    add_to_section_type(&name_capitalized, name)?;
    println!("   {} {}", "✓".green(), "src/executor/planner.rs".dimmed());
    println!();

    // Step 3: Add Config struct and implementation
    println!("{} Generating config struct...", "3.".bold());
    add_config_struct(name, &name_capitalized)?;
    println!("   {} {}", "✓".green(), "src/config/schema.rs".dimmed());
    println!();

    // Step 4: Add handler function
    println!("{} Generating handler function...", "4.".bold());
    add_handler_function(name, &name_capitalized)?;
    println!("   {} {}", "✓".green(), "src/executor/apply.rs".dimmed());
    println!();

    // Step 5: Create manager implementation template
    println!(
        "{} Creating manager implementation template...",
        "5.".bold()
    );
    create_manager_impl(name, &name_capitalized)?;
    println!(
        "   {} {}",
        "✓".green(),
        format!("src/managers/{}.rs", name).dimmed()
    );
    println!();

    // Step 6: Update managers/mod.rs
    println!("{} Updating managers module...", "6.".bold());
    update_managers_mod(name)?;
    println!("   {} {}", "✓".green(), "src/managers/mod.rs".dimmed());
    println!();

    // Step 7: Update add.rs for 'macup add' support
    println!("{} Adding 'macup add' command support...", "7.".bold());
    add_to_add_command(name, &name_capitalized)?;
    println!("   {} {}", "✓".green(), "src/commands/add.rs".dimmed());
    println!();

    // Step 8: Update diff.rs for 'macup diff' support
    println!("{} Adding 'macup diff' command support...", "8.".bold());
    add_to_diff_command(name, &name_capitalized)?;
    println!("   {} {}", "✓".green(), "src/commands/diff.rs".dimmed());
    println!();

    println!("{}", "=".repeat(60).bright_green());
    println!(
        "{}",
        "✅ Package manager created successfully!"
            .bright_green()
            .bold()
    );
    println!("{}", "=".repeat(60).bright_green());
    println!();
    println!("{}", "Next steps:".bold());
    println!(
        "  1. Implement the Manager trait in {}",
        format!("src/managers/{}.rs", name).cyan()
    );
    println!("  2. Run {} to verify compilation", "cargo build".cyan());
    println!(
        "  3. Test with {} in your macup.toml",
        format!("[{}]", name).cyan()
    );
    println!(
        "  4. Test with {}",
        format!("macup add {} <package>", name).cyan()
    );
    println!();

    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Extract the leading whitespace from a marker line in the content
fn extract_indent(content: &str, marker: &str) -> String {
    content
        .lines()
        .find(|line| line.contains(marker))
        .map(|line| {
            let trimmed = line.trim_start();
            let indent_len = line.len() - trimmed.len();
            line[..indent_len].to_string()
        })
        .unwrap_or_default()
}

fn add_to_add_command(name: &str, name_cap: &str) -> Result<()> {
    let add_path = Path::new("src/commands/add.rs");
    let content = fs::read_to_string(add_path).context("Failed to read add.rs")?;

    // 1. Add import with pair markers
    let import_marker = "// CODEGEN_MARKER: insert_manager_import_here";
    if !content.contains(import_marker) {
        anyhow::bail!("Could not find CODEGEN_MARKER: insert_manager_import_here in add.rs");
    }

    // Extract indent from the marker
    let import_indent = extract_indent(&content, import_marker);

    let new_import = format!(
        "{}{}::{}Manager, // CODEGEN[{}]: import\n{}{}",
        import_indent, name, name_cap, name, import_indent, import_marker
    );
    let mut updated_content =
        content.replace(&format!("{}{}", import_indent, import_marker), &new_import);

    // 2. Add match arm with pair markers
    let match_marker = "// CODEGEN_MARKER: insert_manager_match_arm_here";
    if !updated_content.contains(match_marker) {
        anyhow::bail!("Could not find CODEGEN_MARKER: insert_manager_match_arm_here in add.rs");
    }

    // Extract indent from the marker
    let match_indent = extract_indent(&updated_content, match_marker);

    let new_match_arm = format!(
        r#"{}// CODEGEN_START[{}]: match_arm
{}"{}" => Box::new({}Manager::new(max_parallel)),
{}// CODEGEN_END[{}]: match_arm
{}{}"#,
        match_indent,
        name,
        match_indent,
        name,
        name_cap,
        match_indent,
        name,
        match_indent,
        match_marker
    );
    updated_content =
        updated_content.replace(&format!("{}{}", match_indent, match_marker), &new_match_arm);

    fs::write(add_path, updated_content).context("Failed to write add.rs")?;

    Ok(())
}

fn add_to_registry(
    name: &str,
    display: &str,
    icon: &str,
    runtime_cmd: &str,
    runtime_name: &str,
    brew_formula: &str,
    name_cap: &str,
) -> Result<()> {
    let registry_path = Path::new("src/managers/registry.rs");
    let content = fs::read_to_string(registry_path).context("Failed to read registry.rs")?;

    // Use CODEGEN_MARKER
    let insert_marker = "// CODEGEN_MARKER: insert_manager_metadata_here";

    if !content.contains(insert_marker) {
        anyhow::bail!("Could not find CODEGEN_MARKER: insert_manager_metadata_here in registry.rs");
    }

    // Extract indent from the marker
    let indent = extract_indent(&content, insert_marker);

    // Generate with START/END markers for easy removal
    let new_entry = format!(
        r#"{}// CODEGEN_START: {}
{}ManagerMetadata {{
{}    name: "{}",
{}    display_name: "{}",
{}    icon: "{}",
{}    runtime_command: "{}",
{}    runtime_name: "{}",
{}    brew_formula: "{}",
{}    section_type: SectionType::{},
{}}},
{}// CODEGEN_END: {}
{}{}"#,
        indent,
        name,
        indent,
        indent,
        name,
        indent,
        display,
        indent,
        icon,
        indent,
        runtime_cmd,
        indent,
        runtime_name,
        indent,
        brew_formula,
        indent,
        name_cap,
        indent,
        indent,
        name,
        indent,
        insert_marker
    );

    let updated_content = content.replace(&format!("{}{}", indent, insert_marker), &new_entry);

    fs::write(registry_path, updated_content).context("Failed to write registry.rs")?;

    Ok(())
}

fn add_to_section_type(name_cap: &str, name: &str) -> Result<()> {
    let planner_path = Path::new("src/executor/planner.rs");
    let content = fs::read_to_string(planner_path).context("Failed to read planner.rs")?;

    let marker = "// CODEGEN_MARKER: insert_section_type_here";

    if !content.contains(marker) {
        anyhow::bail!("Could not find CODEGEN_MARKER: insert_section_type_here in planner.rs");
    }

    // Extract indent from the marker
    let indent = extract_indent(&content, marker);

    // Generate with START/END markers
    let new_variant = format!(
        "{}// CODEGEN_START: {}\n{}{},\n{}// CODEGEN_END: {}\n{}{}",
        indent, name, indent, name_cap, indent, name, indent, marker
    );
    let updated_content = content.replace(&format!("{}{}", indent, marker), &new_variant);

    fs::write(planner_path, updated_content).context("Failed to write planner.rs")?;

    Ok(())
}

fn add_config_struct(name: &str, name_cap: &str) -> Result<()> {
    let schema_path = Path::new("src/config/schema.rs");
    let content = fs::read_to_string(schema_path).context("Failed to read schema.rs")?;

    // 1. Add field to Config struct with pair markers
    let config_field_marker = "// CODEGEN_MARKER: insert_config_field_here";
    if !content.contains(config_field_marker) {
        anyhow::bail!("Could not find CODEGEN_MARKER: insert_config_field_here in schema.rs");
    }

    // Extract indent from the marker
    let field_indent = extract_indent(&content, config_field_marker);

    let new_field = format!(
        "{}// CODEGEN_START[{}]: config_field\n{}#[serde(default)]\n{}pub {}: Option<{}Config>,\n{}// CODEGEN_END[{}]: config_field\n\n{}{}",
        field_indent, name, field_indent, field_indent, name, name_cap, field_indent, name, field_indent, config_field_marker
    );
    let mut updated_content = content.replace(
        &format!("{}{}", field_indent, config_field_marker),
        &new_field,
    );

    // 2. Add config struct definition with pair markers
    let struct_insert_marker = "// CODEGEN_MARKER: insert_config_struct_here";
    if !updated_content.contains(struct_insert_marker) {
        anyhow::bail!("Could not find CODEGEN_MARKER: insert_config_struct_here in schema.rs");
    }

    // Extract indent from the marker (usually no indent)
    let struct_indent = extract_indent(&updated_content, struct_insert_marker);

    let new_struct = format!(
        r#"{}// CODEGEN_START[{}]: config_struct
{}#[derive(Debug, Clone, Deserialize, Serialize)]
{}pub struct {}Config {{
{}    #[serde(default)]
{}    pub depends_on: Vec<String>,
{}
{}    #[serde(default)]
{}    pub packages: Vec<String>,
{}}}
{}
{}impl PackageManagerSection for {}Config {{
{}    fn get_depends_on(&self) -> &Vec<String> {{
{}        &self.depends_on
{}    }}
{}
{}    fn has_packages(&self) -> bool {{
{}        !self.packages.is_empty()
{}    }}
{}}}
{}// CODEGEN_END[{}]: config_struct
{}
{}{}"#,
        struct_indent,
        name,
        struct_indent,
        struct_indent,
        name_cap,
        struct_indent,
        struct_indent,
        struct_indent,
        struct_indent,
        struct_indent,
        struct_indent,
        struct_indent,
        struct_indent,
        name_cap,
        struct_indent,
        struct_indent,
        struct_indent,
        struct_indent,
        struct_indent,
        struct_indent,
        struct_indent,
        struct_indent,
        struct_indent,
        name,
        struct_indent,
        struct_indent,
        struct_insert_marker
    );

    updated_content = updated_content.replace(
        &format!("{}{}", struct_indent, struct_insert_marker),
        &new_struct,
    );

    // 3. Add match arm with pair markers
    let match_marker = "// CODEGEN_MARKER: insert_manager_match_arm_here";
    if !updated_content.contains(match_marker) {
        anyhow::bail!("Could not find CODEGEN_MARKER: insert_manager_match_arm_here in schema.rs");
    }

    // Extract indent from the marker
    let match_indent = extract_indent(&updated_content, match_marker);

    let new_match_arm = format!(
        r#"{}// CODEGEN_START[{}]: match_arm
{}"{}" => self.{}.as_ref().map(|c| c as &dyn PackageManagerSection),
{}// CODEGEN_END[{}]: match_arm
{}{}"#,
        match_indent,
        name,
        match_indent,
        name,
        name,
        match_indent,
        name,
        match_indent,
        match_marker
    );
    updated_content =
        updated_content.replace(&format!("{}{}", match_indent, match_marker), &new_match_arm);

    fs::write(schema_path, updated_content).context("Failed to write schema.rs")?;

    Ok(())
}

fn add_handler_function(name: &str, name_cap: &str) -> Result<()> {
    let apply_path = Path::new("src/executor/apply.rs");
    let content = fs::read_to_string(apply_path).context("Failed to read apply.rs")?;

    // 1. Add import with pair markers
    let import_marker = "// CODEGEN_MARKER: insert_manager_import_here";
    if !content.contains(import_marker) {
        anyhow::bail!("Could not find CODEGEN_MARKER: insert_manager_import_here in apply.rs");
    }

    // Extract indent from the marker
    let import_indent = extract_indent(&content, import_marker);

    let new_import = format!(
        "{}{}::{}Manager, // CODEGEN[{}]: import\n{}{}",
        import_indent, name, name_cap, name, import_indent, import_marker
    );
    let mut updated_content =
        content.replace(&format!("{}{}", import_indent, import_marker), &new_import);

    // 2. Add handler function using pair markers
    let handler_marker = "// CODEGEN_MARKER: insert_handler_function_here";
    if !updated_content.contains(handler_marker) {
        anyhow::bail!("Could not find CODEGEN_MARKER: insert_handler_function_here in apply.rs");
    }

    // Extract indent from the marker (usually no indent)
    let i = extract_indent(&updated_content, handler_marker);

    // Build handler function with proper indent
    let handler_function = vec![
        format!("{}// CODEGEN_START[{}]: handler_function", i, name),
        format!("{}/// Handler for {} package manager phase", i, name_cap),
        format!("{}fn apply_{}_phase(", i, name),
        format!("{}    config: &Config,", i),
        format!("{}    dry_run: bool,", i),
        format!("{}    max_parallel: usize,", i),
        format!("{}    fail_fast: bool,", i),
        format!("{}    errors: &mut ApplyErrors,", i),
        format!("{}) -> Result<()> {{", i),
        format!("{}    let {}_config = match &config.{} {{", i, name, name),
        format!("{}        Some(cfg) if !cfg.packages.is_empty() => cfg,", i),
        format!("{}        _ => return Ok(()), // No {} config or no packages", i, name),
        format!("{}    }};", i),
        format!("{}", i),
        format!("{}    let meta = ManagerMetadata::get_by_name(\"{}\").unwrap();", i, name),
        format!("{}    ", i),
        format!("{}    println!(", i),
        format!("{}        \"{{}}\",", i),
        format!("{}        format!(\"{{}} Installing {{}}...\", meta.icon, meta.display_name)", i),
        format!("{}            .bright_cyan()", i),
        format!("{}            .bold()", i),
        format!("{}    );", i),
        format!("{}", i),
        format!("{}    // Auto-install runtime if not found", i),
        format!("{}    if !crate::utils::command_exists(meta.runtime_command) {{", i),
        format!("{}        println!(", i),
        format!("{}            \"  ⚠️  {{}} not found, installing {{}} via brew...\",", i),
        format!("{}            meta.runtime_command.yellow(),", i),
        format!("{}            meta.runtime_name.cyan()", i),
        format!("{}        );", i),
        format!("{}", i),
        format!("{}        if dry_run {{", i),
        format!("{}            println!(\"    → Would run: brew install {{}}\", meta.brew_formula);", i),
        format!("{}        }} else {{", i),
        format!("{}            match install_runtime_via_brew(meta.brew_formula) {{", i),
        format!("{}                Ok(_) => {{", i),
        format!("{}                    println!(\"  ✓ {{}} installed\", meta.runtime_name.green());", i),
        format!("{}                }}", i),
        format!("{}                Err(e) => {{", i),
        format!("{}                    println!(\"  ❌ Failed to install {{}}: {{}}\", meta.runtime_name, e);", i),
        format!("{}", i),
        format!("{}                    // Record failures for all packages", i),
        format!("{}                    for pkg in &{}_config.packages {{", i, name),
        format!("{}                        errors.package_failures.push(PackageFailure {{", i),
        format!("{}                            package: pkg.clone(),", i),
        format!("{}                            manager: meta.name.to_string(),", i),
        format!("{}                            reason: format!(\"{{}} installation failed: {{}}\", meta.runtime_name, e),", i),
        format!("{}                        }});", i),
        format!("{}                    }}", i),
        format!("{}", i),
        format!("{}                    if fail_fast {{", i),
        format!("{}                        bail!(\"Failed to install {{}}\", meta.runtime_name);", i),
        format!("{}                    }}", i),
        format!("{}", i),
        format!("{}                    println!();", i),
        format!("{}                    return Ok(());", i),
        format!("{}                }}", i),
        format!("{}            }}", i),
        format!("{}        }}", i),
        format!("{}    }}", i),
        format!("{}", i),
        format!("{}    // Install packages", i),
        format!("{}    if dry_run {{", i),
        format!("{}        println!(\"  Packages: {{:?}}\", {}_config.packages);", i, name),
        format!("{}    }} else {{", i),
        format!("{}        let {}_mgr = {}Manager::new(max_parallel);", i, name, name_cap),
        format!("{}        match {}_mgr.install_packages(&{}_config.packages) {{", i, name, name),
        format!("{}            Ok(result) => {{", i),
        format!("{}                print_result(\"{} packages\", &result);", i, name_cap),
        format!("{}", i),
        format!("{}                // Track failures", i),
        format!("{}                for (pkg, reason) in &result.failed {{", i),
        format!("{}                    errors.package_failures.push(PackageFailure {{", i),
        format!("{}                        package: pkg.clone(),", i),
        format!("{}                        manager: meta.name.to_string(),", i),
        format!("{}                        reason: reason.clone(),", i),
        format!("{}                    }});", i),
        format!("{}                }}", i),
        format!("{}            }}", i),
        format!("{}            Err(e) => {{", i),
        format!("{}                println!(\"  ❌ {{}} installation failed: {{}}\", meta.name, e);", i),
        format!("{}", i),
        format!("{}                if fail_fast {{", i),
        format!("{}                    bail!(\"{{}} installation failed\", meta.name);", i),
        format!("{}                }}", i),
        format!("{}            }}", i),
        format!("{}        }}", i),
        format!("{}    }}", i),
        format!("{}", i),
        format!("{}    println!();", i),
        format!("{}    Ok(())", i),
        format!("{}}}", i),
        format!("{}// CODEGEN_END[{}]: handler_function", i, name),
        format!("{}", i),
        format!("{}{}", i, handler_marker),
    ].join("\n");

    updated_content =
        updated_content.replace(&format!("{}{}", i, handler_marker), &handler_function);

    // 3. Add match arm in apply_plan using pair markers
    let match_marker = "// CODEGEN_MARKER: insert_section_match_arm_here";
    if !updated_content.contains(match_marker) {
        anyhow::bail!("Could not find CODEGEN_MARKER: insert_section_match_arm_here in apply.rs");
    }

    // Extract indent from the marker
    let match_indent = extract_indent(&updated_content, match_marker);

    let new_match_arm = format!(
        r#"{}// CODEGEN_START[{}]: match_arm
{}SectionType::{} => {{
{}    apply_{}_phase(config, dry_run, max_parallel, fail_fast, &mut errors)?;
{}}}
{}// CODEGEN_END[{}]: match_arm
{}
{}{}"#,
        match_indent,
        name,
        match_indent,
        name_cap,
        match_indent,
        name,
        match_indent,
        match_indent,
        name,
        match_indent,
        match_indent,
        match_marker
    );
    updated_content =
        updated_content.replace(&format!("{}{}", match_indent, match_marker), &new_match_arm);

    fs::write(apply_path, updated_content).context("Failed to write apply.rs")?;

    Ok(())
}

fn create_manager_impl(name: &str, name_cap: &str) -> Result<()> {
    let manager_file = format!("src/managers/{}.rs", name);
    let manager_path = Path::new(&manager_file);

    let template = format!(
        r#"use super::{{InstallResult, Manager}};
use anyhow::{{Context, Result}};
use rayon::prelude::*;
use std::collections::HashSet;
use std::process::Command;

/// Manager for {} packages
pub struct {}Manager {{
    max_parallel: usize,
}}

impl {}Manager {{
    pub fn new(max_parallel: usize) -> Self {{
        Self {{ max_parallel }}
    }}

    /// Parse package name with optional binary mapping
    /// Format: "package:binary" or just "package"
    /// Examples:
    ///   - "typescript:tsc" -> install "typescript", check binary "tsc"
    ///   - "eslint" -> install "eslint", check binary "eslint"
    fn parse_package_name(input: &str) -> (&str, &str) {{
        if let Some((pkg, bin)) = input.split_once(':') {{
            (pkg.trim(), bin.trim())
        }} else {{
            (input.trim(), input.trim())
        }}
    }}
}}

impl Manager for {}Manager {{
    fn name(&self) -> &str {{
        "{}"
    }}

    fn is_installed(&self) -> bool {{
        crate::utils::command_exists("{}")
    }}

    fn install_self(&self) -> Result<()> {{
        // Runtime is installed via brew in apply phase
        Ok(())
    }}

    fn list_installed(&self) -> Result<HashSet<String>> {{
        // Not needed - we use `which` to check if packages are installed
        Ok(HashSet::new())
    }}

    fn is_package_installed(&self, package: &str) -> Result<bool> {{
        // Parse package:binary format
        let (_pkg_name, binary_name) = Self::parse_package_name(package);
        
        // Use `which` to check if the binary exists
        Ok(crate::utils::command_exists(binary_name))
    }}

    fn install_package(&self, package: &str) -> Result<()> {{
        // Parse package:binary format - install using package name
        let (pkg_name, _binary_name) = Self::parse_package_name(package);
        
        println!("  Installing {{}}...", pkg_name);

        // TODO: Adjust the install command for your package manager
        // Example for npm: ["install", "--global", pkg_name]
        // Example for cargo: ["install", pkg_name]
        // Example for pip: ["install", pkg_name]
        let status = Command::new("{}")
            .args(&["install", pkg_name]) // Adjust args as needed
            .status()
            .context(format!("Failed to install {{}}", pkg_name))?;

        if !status.success() {{
            anyhow::bail!("Failed to install {{}}", pkg_name);
        }}

        Ok(())
    }}

    fn install_packages(&self, packages: &[String]) -> Result<InstallResult> {{
        let mut result = InstallResult::default();

        // Check which packages are already installed using `which`
        // The check uses binary name, but we keep the full "package:binary" string for tracking
        let (already_installed, to_install): (Vec<_>, Vec<_>) = packages
            .iter()
            .partition(|pkg| self.is_package_installed(pkg).unwrap_or(false));

        result.skipped.extend(already_installed.into_iter().cloned());

        if to_install.is_empty() {{
            return Ok(result);
        }}

        // Collect owned strings for parallel processing
        let to_install: Vec<String> = to_install.into_iter().cloned().collect();

        // Install packages in parallel
        let install_results: Vec<_> = to_install
            .par_iter()
            .map(|pkg| {{
                (pkg.clone(), self.install_package(pkg))
            }})
            .collect();

        // Separate successes and failures
        for (pkg, res) in install_results {{
            match res {{
                Ok(_) => result.success.push(pkg),
                Err(e) => result.failed.push((pkg, e.to_string())),
            }}
        }}

        Ok(result)
    }}
}}
"#,
        name, name_cap, name_cap, name_cap, name, name, name
    );

    fs::write(manager_path, template).context("Failed to create manager implementation")?;

    Ok(())
}

fn update_managers_mod(name: &str) -> Result<()> {
    let mod_path = Path::new("src/managers/mod.rs");
    let content = fs::read_to_string(mod_path).context("Failed to read managers/mod.rs")?;

    // Use CODEGEN_MARKER with pair markers
    let marker = "// CODEGEN_MARKER: insert_module_declaration_here";

    if !content.contains(marker) {
        anyhow::bail!(
            "Could not find CODEGEN_MARKER: insert_module_declaration_here in managers/mod.rs"
        );
    }

    // Extract indent from the marker (usually no indent)
    let indent = extract_indent(&content, marker);

    let new_mod = format!(
        "{}// CODEGEN_START[{}]: module\n{}pub mod {};\n{}// CODEGEN_END[{}]: module\n{}{}",
        indent, name, indent, name, indent, name, indent, marker
    );
    let updated_content = content.replace(&format!("{}{}", indent, marker), &new_mod);

    fs::write(mod_path, updated_content).context("Failed to write managers/mod.rs")?;

    Ok(())
}

fn add_to_diff_command(name: &str, name_cap: &str) -> Result<()> {
    let diff_path = Path::new("src/commands/diff.rs");
    let content = fs::read_to_string(diff_path).context("Failed to read diff.rs")?;

    // 1. Add config import at the top
    let config_import_pattern = "use crate::config::{load_config_auto,";
    if !content.contains(config_import_pattern) {
        anyhow::bail!("Could not find config import in diff.rs");
    }

    // Find the end of config imports line (after the closing })
    let config_line_start = content.find(config_import_pattern).unwrap();
    let after_import_start = &content[config_line_start..];
    let closing_brace_pos = after_import_start.find("};").unwrap();
    let insert_pos = config_line_start + closing_brace_pos;

    // Insert new config import before the closing }
    let mut updated_content = String::new();
    updated_content.push_str(&content[..insert_pos]);
    updated_content.push_str(&format!(", {}Config", name_cap));
    updated_content.push_str(&content[insert_pos..]);

    // 2. Add import for manager
    let import_marker = "// CODEGEN_MARKER: insert_import_here";
    if !updated_content.contains(import_marker) {
        anyhow::bail!("Could not find CODEGEN_MARKER: insert_import_here in diff.rs");
    }

    let import_indent = extract_indent(&updated_content, import_marker);
    let new_import = format!(
        "{}{}::{}Manager, // CODEGEN[{}]: import\n{}{}",
        import_indent, name, name_cap, name, import_indent, import_marker
    );
    updated_content =
        updated_content.replace(&format!("{}{}", import_indent, import_marker), &new_import);

    // 3. Add check function call
    let call_marker = "// CODEGEN_MARKER: insert_check_call_here";
    if !updated_content.contains(call_marker) {
        anyhow::bail!("Could not find CODEGEN_MARKER: insert_check_call_here in diff.rs");
    }

    let call_indent = extract_indent(&updated_content, call_marker);
    let new_call = vec![
        format!("{}// CODEGEN_START[{}]: check_call", call_indent, name),
        format!(
            "{}if let Some({}_config) = &config.{} {{",
            call_indent, name, name
        ),
        format!(
            "{}    if let Some(result) = check_{}_section({}_config) {{",
            call_indent, name, name
        ),
        format!("{}        results.push(result);", call_indent),
        format!("{}    }}", call_indent),
        format!("{}}}", call_indent),
        format!("{}// CODEGEN_END[{}]: check_call", call_indent, name),
        format!(""),
        format!("{}{}", call_indent, call_marker),
    ]
    .join("\n");

    updated_content =
        updated_content.replace(&format!("{}{}", call_indent, call_marker), &new_call);

    // 3. Add check function implementation
    let func_marker = "// CODEGEN_MARKER: insert_check_function_here";
    if !updated_content.contains(func_marker) {
        anyhow::bail!("Could not find CODEGEN_MARKER: insert_check_function_here in diff.rs");
    }

    let func_indent = extract_indent(&updated_content, func_marker);
    let check_function = generate_diff_check_function(name, name_cap, &func_indent);
    updated_content =
        updated_content.replace(&format!("{}{}", func_indent, func_marker), &check_function);

    fs::write(diff_path, updated_content).context("Failed to write diff.rs")?;

    Ok(())
}

fn generate_diff_check_function(name: &str, name_cap: &str, i: &str) -> String {
    vec![
        format!("{}// CODEGEN_START[{}]: check_function", i, name),
        format!("{}/// Check {} packages", i, name_cap),
        format!("{}fn check_{}_section(config: &{}Config) -> Option<DiffResult> {{", i, name, name_cap),
        format!("{}    if config.packages.is_empty() {{", i),
        format!("{}        return None;", i),
        format!("{}    }}", i),
        format!(""),
        format!("{}    let meta = ManagerMetadata::get_by_name(\"{}\").unwrap();", i, name),
        format!(""),
        format!("{}    // Check if runtime is installed", i),
        format!("{}    if !crate::utils::command_exists(meta.runtime_command) {{", i),
        format!("{}        return Some(DiffResult {{", i),
        format!("{}            manager_name: meta.name.to_string(),", i),
        format!("{}            icon: meta.icon.to_string(),", i),
        format!("{}            display_name: meta.display_name.to_string(),", i),
        format!("{}            installed: vec![],", i),
        format!("{}            missing: vec![],", i),
        format!("{}            skipped_reason: Some(format!(\"{{}} not installed\", meta.runtime_command)),", i),
        format!("{}        }});", i),
        format!("{}    }}", i),
        format!(""),
        format!("{}    // Check each package in parallel", i),
        format!("{}    let mgr = {}Manager::new(1);", i, name_cap),
        format!("{}    let pkg_results: Vec<_> = config", i),
        format!("{}        .packages", i),
        format!("{}        .par_iter()", i),
        format!("{}        .map(|pkg| {{", i),
        format!("{}            // Parse package:binary format - show only package name", i),
        format!("{}            let (pkg_name, _) = parse_package_name(pkg);", i),
        format!("{}            let is_installed = mgr.is_package_installed(pkg).unwrap_or(false);", i),
        format!("{}            (pkg_name.to_string(), is_installed)", i),
        format!("{}        }})", i),
        format!("{}        .collect();", i),
        format!(""),
        format!("{}    let mut installed = vec![];", i),
        format!("{}    let mut missing = vec![];", i),
        format!(""),
        format!("{}    for (pkg, is_installed) in pkg_results {{", i),
        format!("{}        if is_installed {{", i),
        format!("{}            installed.push(pkg);", i),
        format!("{}        }} else {{", i),
        format!("{}            missing.push(pkg);", i),
        format!("{}        }}", i),
        format!("{}    }}", i),
        format!(""),
        format!("{}    Some(DiffResult {{", i),
        format!("{}        manager_name: meta.name.to_string(),", i),
        format!("{}        icon: meta.icon.to_string(),", i),
        format!("{}        display_name: meta.display_name.to_string(),", i),
        format!("{}        installed,", i),
        format!("{}        missing,", i),
        format!("{}        skipped_reason: None,", i),
        format!("{}    }})", i),
        format!("{}}}", i),
        format!("{}// CODEGEN_END[{}]: check_function", i, name),
        format!(""),
        format!("{}// CODEGEN_MARKER: insert_check_function_here", i),
    ].join("\n")
}
