use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

pub fn run(name: &str) -> Result<()> {
    println!("{}", "=".repeat(60).bright_blue());
    println!(
        "{}",
        format!("Removing package manager: {}", name)
            .bright_blue()
            .bold()
    );
    println!("{}", "=".repeat(60).bright_blue());
    println!();

    let name_capitalized = capitalize(name);

    // Validate that the manager exists in registry first
    println!("{} Checking if manager exists...", "→".bold());
    if !check_manager_exists(name)? {
        anyhow::bail!(
            "Manager '{}' not found in registry. Nothing to remove.",
            name
        );
    }
    println!("   {} Manager found in registry", "✓".green());
    println!();

    // Step 1: Remove from registry
    println!("{} Removing from registry...", "1.".bold());
    remove_from_registry(name, &name_capitalized)?;
    println!("   {} {}", "✓".green(), "src/managers/registry.rs".dimmed());
    println!();

    // Step 2: Remove from SectionType enum
    println!("{} Removing from SectionType enum...", "2.".bold());
    remove_from_section_type(name, &name_capitalized)?;
    println!("   {} {}", "✓".green(), "src/executor/planner.rs".dimmed());
    println!();

    // Step 3: Remove Config struct and implementation
    println!("{} Removing config struct...", "3.".bold());
    remove_config_struct(name, &name_capitalized)?;
    println!("   {} {}", "✓".green(), "src/config/schema.rs".dimmed());
    println!();

    // Step 4: Remove handler function
    println!("{} Removing handler function...", "4.".bold());
    remove_handler_function(name, &name_capitalized)?;
    println!("   {} {}", "✓".green(), "src/executor/apply.rs".dimmed());
    println!();

    // Step 5: Remove manager implementation file
    println!("{} Removing manager implementation...", "5.".bold());
    remove_manager_impl(name)?;
    println!(
        "   {} {}",
        "✓".green(),
        format!("src/managers/{}.rs", name).dimmed()
    );
    println!();

    // Step 6: Update managers/mod.rs
    println!("{} Updating managers module...", "6.".bold());
    remove_from_managers_mod(name)?;
    println!("   {} {}", "✓".green(), "src/managers/mod.rs".dimmed());
    println!();

    // Step 7: Remove from add.rs
    println!("{} Removing from 'macup add' command...", "7.".bold());
    remove_from_add_command(name, &name_capitalized)?;
    println!("   {} {}", "✓".green(), "src/commands/add.rs".dimmed());
    println!();

    // Step 8: Remove from diff.rs
    println!("{} Removing from 'macup diff' command...", "8.".bold());
    remove_from_diff_command(name, &name_capitalized)?;
    println!("   {} {}", "✓".green(), "src/commands/diff.rs".dimmed());
    println!();

    println!("{}", "=".repeat(60).bright_green());
    println!(
        "{}",
        "✅ Package manager removed successfully!"
            .bright_green()
            .bold()
    );
    println!("{}", "=".repeat(60).bright_green());
    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. Run {} to verify compilation", "cargo build".cyan());
    println!(
        "  2. Remove any references to {} in your macup.toml",
        format!("[{}]", name).cyan()
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

fn remove_from_add_command(name: &str, name_cap: &str) -> Result<()> {
    let add_path = Path::new("src/commands/add.rs");
    let content = fs::read_to_string(add_path).context("Failed to read add.rs")?;

    // 1. Remove import with inline marker - handle different formatting
    // Pattern 1: with comment and alignment spaces (existing wrapped imports)
    let import_pattern1 = format!(
        "    {}::{}Manager,             // CODEGEN[{}]: import\n",
        name, name_cap, name
    );
    // Pattern 2: with comment, single space (newly generated imports)
    let import_pattern2 = format!(
        "    {}::{}Manager, // CODEGEN[{}]: import\n",
        name, name_cap, name
    );
    // Pattern 3: fallback without comment
    let import_pattern3 = format!("    {}::{}Manager,\n", name, name_cap);

    let updated_content = if content.contains(&import_pattern1) {
        content.replace(&import_pattern1, "")
    } else if content.contains(&import_pattern2) {
        content.replace(&import_pattern2, "")
    } else {
        content.replace(&import_pattern3, "")
    };

    // 2. Remove match arm using pair markers
    let match_start = format!("                // CODEGEN_START[{}]: match_arm", name);
    let match_end = format!("                // CODEGEN_END[{}]: match_arm", name);

    let match_start_pos = updated_content.find(&match_start).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_START[{}]: match_arm marker in add.rs",
            name
        )
    })?;

    let after_match_start = &updated_content[match_start_pos..];
    let match_end_offset = after_match_start.find(&match_end).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_END[{}]: match_arm marker in add.rs",
            name
        )
    })?;

    // Include the END marker and newline
    let match_end_pos = match_start_pos + match_end_offset + match_end.len() + 1; // +1 for newline

    // Remove match arm
    let mut final_content = String::new();
    final_content.push_str(&updated_content[..match_start_pos]);
    final_content.push_str(&updated_content[match_end_pos..]);

    fs::write(add_path, final_content).context("Failed to write add.rs")?;

    Ok(())
}

fn remove_from_diff_command(name: &str, name_cap: &str) -> Result<()> {
    let diff_path = Path::new("src/commands/diff.rs");
    let content = fs::read_to_string(diff_path).context("Failed to read diff.rs")?;

    // 1. Remove config import from line 1
    // Pattern: ", TestpkgConfig" in the config import
    let config_import_pattern = format!(", {}Config", name_cap);
    let updated_content = content.replace(&config_import_pattern, "");

    // 2. Remove manager import using inline marker
    let import_pattern1 = format!(
        "    {}::{}Manager,     // CODEGEN[{}]: import\n",
        name, name_cap, name
    );
    let import_pattern2 = format!(
        "    {}::{}Manager, // CODEGEN[{}]: import\n",
        name, name_cap, name
    );
    let import_pattern3 = format!("    {}::{}Manager,\n", name, name_cap);

    let updated_content = if updated_content.contains(&import_pattern1) {
        updated_content.replace(&import_pattern1, "")
    } else if updated_content.contains(&import_pattern2) {
        updated_content.replace(&import_pattern2, "")
    } else {
        updated_content.replace(&import_pattern3, "")
    };

    // 3. Remove check function call using pair markers
    let call_start = format!("    // CODEGEN_START[{}]: check_call", name);
    let call_end = format!("    // CODEGEN_END[{}]: check_call", name);

    let call_start_pos = updated_content.find(&call_start).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_START[{}]: check_call marker in diff.rs",
            name
        )
    })?;

    let after_call_start = &updated_content[call_start_pos..];
    let call_end_offset = after_call_start.find(&call_end).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_END[{}]: check_call marker in diff.rs",
            name
        )
    })?;

    // Include the END marker and newlines
    let call_end_pos = call_start_pos + call_end_offset + call_end.len() + 1; // +1 for newline

    // Remove check call block
    let mut updated_content2 = String::new();
    updated_content2.push_str(&updated_content[..call_start_pos]);
    updated_content2.push_str(&updated_content[call_end_pos..]);

    // 4. Remove check function using pair markers
    let fn_start = format!("// CODEGEN_START[{}]: check_function", name);
    let fn_end = format!("// CODEGEN_END[{}]: check_function", name);

    let fn_start_pos = updated_content2.find(&fn_start).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_START[{}]: check_function marker in diff.rs",
            name
        )
    })?;

    let after_fn_start = &updated_content2[fn_start_pos..];
    let fn_end_offset = after_fn_start.find(&fn_end).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_END[{}]: check_function marker in diff.rs",
            name
        )
    })?;

    // Include the END marker and newlines
    let fn_end_pos = fn_start_pos + fn_end_offset + fn_end.len() + 1; // +1 for newline

    // Remove function block
    let mut final_content = String::new();
    final_content.push_str(&updated_content2[..fn_start_pos]);
    final_content.push_str(&updated_content2[fn_end_pos..]);

    fs::write(diff_path, final_content).context("Failed to write diff.rs")?;

    Ok(())
}

fn check_manager_exists(name: &str) -> Result<bool> {
    let registry_path = Path::new("src/managers/registry.rs");
    let content = fs::read_to_string(registry_path).context("Failed to read registry.rs")?;

    // Check if manager entry exists
    let search_pattern = format!(r#"name: "{}","#, name);
    Ok(content.contains(&search_pattern))
}

fn remove_from_registry(name: &str, _name_cap: &str) -> Result<()> {
    let registry_path = Path::new("src/managers/registry.rs");
    let content = fs::read_to_string(registry_path).context("Failed to read registry.rs")?;

    // Find and remove using pair markers
    let start_marker = format!("    // CODEGEN_START: {}", name);
    let end_marker = format!("    // CODEGEN_END: {}", name);

    let start_pos = content.find(&start_marker).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_START marker for {} in registry.rs",
            name
        )
    })?;

    let after_start = &content[start_pos..];
    let end_offset = after_start.find(&end_marker).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_END marker for {} in registry.rs",
            name
        )
    })?;

    // Include the END marker and newline
    let end_pos = start_pos + end_offset + end_marker.len() + 1; // +1 for newline

    // Remove from start to end
    let mut updated_content = String::new();
    updated_content.push_str(&content[..start_pos]);
    updated_content.push_str(&content[end_pos..]);

    fs::write(registry_path, updated_content).context("Failed to write registry.rs")?;

    Ok(())
}

fn remove_from_section_type(name: &str, _name_cap: &str) -> Result<()> {
    let planner_path = Path::new("src/executor/planner.rs");
    let content = fs::read_to_string(planner_path).context("Failed to read planner.rs")?;

    // Find and remove using pair markers
    let start_marker = format!("    // CODEGEN_START: {}", name);
    let end_marker = format!("    // CODEGEN_END: {}", name);

    let start_pos = content.find(&start_marker).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_START marker for {} in planner.rs",
            name
        )
    })?;

    let after_start = &content[start_pos..];
    let end_offset = after_start.find(&end_marker).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_END marker for {} in planner.rs",
            name
        )
    })?;

    // Include the END marker and newline
    let end_pos = start_pos + end_offset + end_marker.len() + 1;

    let mut updated_content = String::new();
    updated_content.push_str(&content[..start_pos]);
    updated_content.push_str(&content[end_pos..]);

    fs::write(planner_path, updated_content).context("Failed to write planner.rs")?;

    Ok(())
}

fn remove_config_struct(name: &str, _name_cap: &str) -> Result<()> {
    let schema_path = Path::new("src/config/schema.rs");
    let content = fs::read_to_string(schema_path).context("Failed to read schema.rs")?;

    // 1. Remove field from Config struct using pair markers
    let field_start = format!("    // CODEGEN_START[{}]: config_field", name);
    let field_end = format!("    // CODEGEN_END[{}]: config_field", name);

    let field_start_pos = content.find(&field_start).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_START[{}]: config_field marker in schema.rs",
            name
        )
    })?;

    let after_field_start = &content[field_start_pos..];
    let field_end_offset = after_field_start.find(&field_end).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_END[{}]: config_field marker in schema.rs",
            name
        )
    })?;

    // Include the END marker and newline
    let field_end_pos = field_start_pos + field_end_offset + field_end.len() + 1; // +1 for newline

    // Remove field
    let mut updated_content = String::new();
    updated_content.push_str(&content[..field_start_pos]);
    updated_content.push_str(&content[field_end_pos..]);

    // 2. Remove config struct definition using pair markers
    let struct_start = format!("// CODEGEN_START[{}]: config_struct", name);
    let struct_end = format!("// CODEGEN_END[{}]: config_struct", name);

    let struct_start_pos = updated_content.find(&struct_start).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_START[{}]: config_struct marker in schema.rs",
            name
        )
    })?;

    let after_struct_start = &updated_content[struct_start_pos..];
    let struct_end_offset = after_struct_start.find(&struct_end).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_END[{}]: config_struct marker in schema.rs",
            name
        )
    })?;

    // Include the END marker and newline
    let struct_end_pos = struct_start_pos + struct_end_offset + struct_end.len() + 1; // +1 for newline

    // Remove struct
    let mut updated_content2 = String::new();
    updated_content2.push_str(&updated_content[..struct_start_pos]);
    updated_content2.push_str(&updated_content[struct_end_pos..]);

    // 3. Remove match arm using pair markers
    let match_start = format!("            // CODEGEN_START[{}]: match_arm", name);
    let match_end = format!("            // CODEGEN_END[{}]: match_arm", name);

    let match_start_pos = updated_content2.find(&match_start).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_START[{}]: match_arm marker in schema.rs",
            name
        )
    })?;

    let after_match_start = &updated_content2[match_start_pos..];
    let match_end_offset = after_match_start.find(&match_end).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_END[{}]: match_arm marker in schema.rs",
            name
        )
    })?;

    // Include the END marker and newline
    let match_end_pos = match_start_pos + match_end_offset + match_end.len() + 1; // +1 for newline

    // Remove match arm
    let mut final_content = String::new();
    final_content.push_str(&updated_content2[..match_start_pos]);
    final_content.push_str(&updated_content2[match_end_pos..]);

    fs::write(schema_path, final_content).context("Failed to write schema.rs")?;

    Ok(())
}

fn remove_handler_function(name: &str, _name_cap: &str) -> Result<()> {
    let apply_path = Path::new("src/executor/apply.rs");
    let content = fs::read_to_string(apply_path).context("Failed to read apply.rs")?;

    // 1. Remove handler function using pair markers
    let start_marker = format!("// CODEGEN_START[{}]: handler_function", name);
    let end_marker = format!("// CODEGEN_END[{}]: handler_function", name);

    let start_pos = content.find(&start_marker).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_START[{}]: handler_function marker in apply.rs",
            name
        )
    })?;

    let after_start = &content[start_pos..];
    let end_offset = after_start.find(&end_marker).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_END[{}]: handler_function marker in apply.rs",
            name
        )
    })?;

    // Include the END marker and newline
    let end_pos = start_pos + end_offset + end_marker.len() + 1; // +1 for newline

    // Remove from start to end
    let mut updated_content = String::new();
    updated_content.push_str(&content[..start_pos]);
    updated_content.push_str(&content[end_pos..]);

    // 2. Remove match arm using pair markers
    let match_start = format!("            // CODEGEN_START[{}]: match_arm", name);
    let match_end = format!("            // CODEGEN_END[{}]: match_arm", name);

    let match_start_pos = updated_content.find(&match_start).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_START[{}]: match_arm marker in apply.rs",
            name
        )
    })?;

    let after_match_start = &updated_content[match_start_pos..];
    let match_end_offset = after_match_start.find(&match_end).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_END[{}]: match_arm marker in apply.rs",
            name
        )
    })?;

    // Include the END marker and newline
    let match_end_pos = match_start_pos + match_end_offset + match_end.len() + 1; // +1 for newline

    // Remove match arm
    let mut final_content = String::new();
    final_content.push_str(&updated_content[..match_start_pos]);
    final_content.push_str(&updated_content[match_end_pos..]);

    // 3. Remove import for manager using inline marker
    let import_pattern_with_comment = format!(
        "    {}::{}Manager, // CODEGEN[{}]: import\n",
        name,
        capitalize(name),
        name
    );
    let import_pattern_fallback1 = format!(
        ", {}::{}Manager, // CODEGEN[{}]: import",
        name,
        capitalize(name),
        name
    );
    let import_pattern_fallback2 = format!(", {}::{}Manager", name, capitalize(name));

    // Try different patterns
    final_content = if final_content.contains(&import_pattern_with_comment) {
        final_content.replace(&import_pattern_with_comment, "")
    } else if final_content.contains(&import_pattern_fallback1) {
        final_content.replace(&import_pattern_fallback1, "")
    } else {
        final_content.replace(&import_pattern_fallback2, "")
    };

    fs::write(apply_path, final_content).context("Failed to write apply.rs")?;

    Ok(())
}

fn remove_manager_impl(name: &str) -> Result<()> {
    let manager_file = format!("src/managers/{}.rs", name);
    let manager_path = Path::new(&manager_file);

    if !manager_path.exists() {
        println!("   {} Manager file not found, skipping", "⚠".yellow());
        return Ok(());
    }

    fs::remove_file(manager_path).context("Failed to remove manager implementation")?;

    Ok(())
}

fn remove_from_managers_mod(name: &str) -> Result<()> {
    let mod_path = Path::new("src/managers/mod.rs");
    let content = fs::read_to_string(mod_path).context("Failed to read managers/mod.rs")?;

    // Remove module declaration using pair markers
    let start_marker = format!("// CODEGEN_START[{}]: module", name);
    let end_marker = format!("// CODEGEN_END[{}]: module", name);

    let start_pos = content.find(&start_marker);

    if start_pos.is_none() {
        println!(
            "   {} Module declaration markers not found, trying fallback",
            "⚠".yellow()
        );

        // Fallback to simple replacement
        let mod_line = format!("pub mod {};\n", name);
        if !content.contains(&mod_line) {
            println!("   {} Module declaration not found, skipping", "⚠".yellow());
            return Ok(());
        }

        let updated_content = content.replace(&mod_line, "");
        fs::write(mod_path, updated_content).context("Failed to write managers/mod.rs")?;
        return Ok(());
    }

    let start_pos = start_pos.unwrap();
    let after_start = &content[start_pos..];
    let end_offset = after_start.find(&end_marker).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not find CODEGEN_END[{}]: module marker in managers/mod.rs",
            name
        )
    })?;

    // Include the END marker and newline
    let end_pos = start_pos + end_offset + end_marker.len() + 1; // +1 for newline

    // Remove from start to end
    let mut updated_content = String::new();
    updated_content.push_str(&content[..start_pos]);
    updated_content.push_str(&content[end_pos..]);

    fs::write(mod_path, updated_content).context("Failed to write managers/mod.rs")?;

    Ok(())
}
