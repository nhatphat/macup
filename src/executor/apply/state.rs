use std::collections::HashSet;

/// Tracks execution context and state.
#[derive(Debug, Default)]
pub(super) struct ExecutionContext {
    pub(super) available_managers: HashSet<String>,
    pub(super) skipped_phases: Vec<SkippedPhase>,
}

#[derive(Debug)]
pub(super) struct SkippedPhase {
    pub(super) name: String,
    pub(super) reason: String,
}

/// Tracks failures during apply execution.
#[derive(Debug, Default)]
pub(super) struct ApplyErrors {
    pub(super) manager_failures: Vec<ManagerFailure>,
    pub(super) package_failures: Vec<PackageFailure>,
}

#[derive(Debug)]
pub(super) struct ManagerFailure {
    pub(super) name: String,
    pub(super) reason: String,
}

#[derive(Debug)]
pub(super) struct PackageFailure {
    pub(super) package: String,
    pub(super) manager: String,
    pub(super) reason: String,
}

impl ApplyErrors {
    pub(super) fn has_failures(&self) -> bool {
        !self.manager_failures.is_empty() || !self.package_failures.is_empty()
    }
}
