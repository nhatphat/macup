use crate::config::Config;
use anyhow::Result;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub phases: Vec<Phase>,
}

#[derive(Debug, Clone)]
pub struct Phase {
    #[allow(dead_code)]
    pub name: String,
    pub section_type: SectionType,
    #[allow(dead_code)]
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum SectionType {
    Managers,
    Brew,
    Mas,
    Npm,
    Cargo,
    Install,
    System,
}

pub fn create_execution_plan(config: &Config) -> Result<ExecutionPlan> {
    let mut phases = vec![];

    // Phase 1: Always check/install managers first
    phases.push(Phase {
        name: "managers".to_string(),
        section_type: SectionType::Managers,
        depends_on: vec![],
    });

    // Build dependency graph
    let mut deps_map = HashMap::new();

    if let Some(install) = &config.install {
        deps_map.insert("install", install.depends_on.clone());
    }

    if let Some(brew) = &config.brew {
        deps_map.insert("brew", brew.depends_on.clone());
    }

    if let Some(mas) = &config.mas {
        deps_map.insert("mas", mas.depends_on.clone());
    }

    if let Some(npm) = &config.npm {
        deps_map.insert("npm", npm.depends_on.clone());
    }

    if let Some(cargo) = &config.cargo {
        deps_map.insert("cargo", cargo.depends_on.clone());
    }

    if let Some(system) = &config.system {
        deps_map.insert("system", system.depends_on.clone());
    }

    // Topological sort to determine execution order
    let mut satisfied = HashSet::new();
    satisfied.insert("brew".to_string()); // Assume brew always available after managers

    let mut remaining: Vec<&str> = deps_map.keys().copied().collect();

    while !remaining.is_empty() {
        let before_len = remaining.len();

        remaining.retain(|&name| {
            let deps = deps_map.get(name).map(|v| v.as_slice()).unwrap_or(&[]);

            if deps.iter().all(|d| satisfied.contains(d)) {
                // All dependencies satisfied, add to phases
                let section_type = match name {
                    "install" => SectionType::Install,
                    "brew" => SectionType::Brew,
                    "mas" => SectionType::Mas,
                    "npm" => SectionType::Npm,
                    "cargo" => SectionType::Cargo,
                    "system" => SectionType::System,
                    _ => return true,
                };

                phases.push(Phase {
                    name: name.to_string(),
                    section_type,
                    depends_on: deps.to_vec(),
                });

                satisfied.insert(name.to_string());
                false // Remove from remaining
            } else {
                true // Keep in remaining
            }
        });

        // Check for cycles
        if remaining.len() == before_len && !remaining.is_empty() {
            anyhow::bail!(
                "Dependency cycle or unsatisfied dependencies: {:?}",
                remaining
            );
        }
    }

    Ok(ExecutionPlan { phases })
}
