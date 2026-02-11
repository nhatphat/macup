use super::Config;
use crate::managers::PACKAGE_MANAGERS;
use anyhow::Result;
use std::collections::{HashMap, HashSet};

/// Validate config for correctness
pub fn validate_config(config: &Config) -> Result<()> {
    // Check for dependency cycles
    check_dependency_cycles(config)?;

    // Validate install scripts have binary OR check
    validate_install_scripts(config)?;

    Ok(())
}

/// Validate that install scripts have at least binary or check defined
fn validate_install_scripts(config: &Config) -> Result<()> {
    if let Some(install) = &config.install {
        for script in &install.scripts {
            if script.binary.is_none() && script.check.is_none() {
                anyhow::bail!(
                    "Install script '{}' must have either 'binary' or 'check' field defined",
                    script.name
                );
            }
        }
    }
    Ok(())
}

/// Check for circular dependencies in depends_on
fn check_dependency_cycles(config: &Config) -> Result<()> {
    let mut deps = HashMap::new();

    // Build dependency graph
    if let Some(brew) = &config.brew {
        deps.insert("brew", brew.depends_on.clone());
    }

    // Use registry to iterate over package managers
    for meta in PACKAGE_MANAGERS {
        if let Some(manager_config) = config.get_manager_config(meta.name) {
            deps.insert(meta.name, manager_config.get_depends_on().clone());
        }
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
