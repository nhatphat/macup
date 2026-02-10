use crate::executor::SectionType;

/// Metadata for a package manager
#[derive(Debug, Clone)]
pub struct ManagerMetadata {
    /// Manager name (used in config sections)
    pub name: &'static str,

    /// Display name for user-facing messages
    pub display_name: &'static str,

    /// Icon emoji for terminal output
    pub icon: &'static str,

    /// Command to check if runtime is installed
    pub runtime_command: &'static str,

    /// Human-readable runtime name
    pub runtime_name: &'static str,

    /// Brew formula name to install runtime
    pub brew_formula: &'static str,

    /// Corresponding section type in execution plan
    pub section_type: SectionType,
}

/// Registry of all supported package managers (excluding brew, install, system)
pub static PACKAGE_MANAGERS: &[ManagerMetadata] = &[
    // CODEGEN_START: mas
    ManagerMetadata {
        name: "mas",
        display_name: "Mac App Store apps",
        icon: "📱",
        runtime_command: "mas",
        runtime_name: "mas-cli",
        brew_formula: "mas",
        section_type: SectionType::Mas,
    },
    // CODEGEN_END: mas
    // CODEGEN_START: npm
    ManagerMetadata {
        name: "npm",
        display_name: "npm packages",
        icon: "📦",
        runtime_command: "npm",
        runtime_name: "node",
        brew_formula: "node",
        section_type: SectionType::Npm,
    },
    // CODEGEN_END: npm
    // CODEGEN_START: cargo
    ManagerMetadata {
        name: "cargo",
        display_name: "cargo packages",
        icon: "🦀",
        runtime_command: "cargo",
        runtime_name: "rust",
        brew_formula: "rust",
        section_type: SectionType::Cargo,
    },
    // CODEGEN_END: cargo
    // CODEGEN_MARKER: insert_manager_metadata_here
];

impl ManagerMetadata {
    /// Get manager metadata by name
    pub fn get_by_name(name: &str) -> Option<&'static ManagerMetadata> {
        PACKAGE_MANAGERS.iter().find(|m| m.name == name)
    }

    /// Get manager metadata by section type
    #[allow(dead_code)]
    pub fn get_by_section_type(section_type: &SectionType) -> Option<&'static ManagerMetadata> {
        PACKAGE_MANAGERS
            .iter()
            .find(|m| &m.section_type == section_type)
    }

    /// Get all manager names
    #[allow(dead_code)]
    pub fn all_names() -> Vec<&'static str> {
        PACKAGE_MANAGERS.iter().map(|m| m.name).collect()
    }
}
