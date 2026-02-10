use super::Config;
use anyhow::Result;
use std::collections::{HashMap, HashSet};

/// Validate config for correctness
pub fn validate_config(config: &Config) -> Result<()> {
    // Check for dependency cycles
    check_dependency_cycles(config)?;

    // Validate manager names
    validate_manager_names(config)?;

    Ok(())
}

/// Check for circular dependencies in depends_on
fn check_dependency_cycles(config: &Config) -> Result<()> {
    let mut deps = HashMap::new();

    // Build dependency graph
    if let Some(brew) = &config.brew {
        deps.insert("brew", brew.depends_on.clone());
    }
    if let Some(mas) = &config.mas {
        deps.insert("mas", mas.depends_on.clone());
    }
    if let Some(npm) = &config.npm {
        deps.insert("npm", npm.depends_on.clone());
    }
    if let Some(cargo) = &config.cargo {
        deps.insert("cargo", cargo.depends_on.clone());
    }
    if let Some(install) = &config.install {
        deps.insert("install", install.depends_on.clone());
    }
    if let Some(system) = &config.system {
        deps.insert("system", system.depends_on.clone());
    }

    // Check each node for cycles using DFS
    for &node in deps.keys() {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        if has_cycle(node, &deps, &mut visited, &mut stack) {
            anyhow::bail!("Dependency cycle detected involving: {}", node);
        }
    }

    Ok(())
}

fn has_cycle(
    node: &str,
    deps: &HashMap<&str, Vec<String>>,
    visited: &mut HashSet<String>,
    stack: &mut HashSet<String>,
) -> bool {
    if stack.contains(node) {
        return true;
    }
    if visited.contains(node) {
        return false;
    }

    visited.insert(node.to_string());
    stack.insert(node.to_string());

    if let Some(neighbors) = deps.get(node) {
        for neighbor in neighbors {
            if has_cycle(neighbor, deps, visited, stack) {
                return true;
            }
        }
    }

    stack.remove(node);
    false
}

/// Validate that manager names are recognized
fn validate_manager_names(config: &Config) -> Result<()> {
    let valid_managers = ["brew", "mas", "npm", "cargo"];

    // Check required managers (if explicitly declared)
    for manager in &config.managers.required {
        if !valid_managers.contains(&manager.as_str()) {
            anyhow::bail!("Unknown required manager: {}", manager);
        }
    }

    Ok(())
}
