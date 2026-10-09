//! Immutable Rename Command intent and projection from the sealed Sema owner.

use quanta_gqlang_v2_sema::{RenamePhaseV3, ResolvedRenameCommandV3};
use std::sync::Arc;

/// Exact host-issued code symbol carried by a compiled rename intent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionSymbolTargetV3 {
    pub(super) selector: Arc<str>,
    pub(super) repo_path: Arc<str>,
    pub(super) symbol_id: Arc<str>,
    pub(super) start_byte: u64,
    pub(super) end_byte: u64,
}
impl ExecutionSymbolTargetV3 {
    #[must_use]
    pub fn selector(&self) -> &str {
        &self.selector
    }
    #[must_use]
    pub fn repo_path(&self) -> &str {
        &self.repo_path
    }
    #[must_use]
    pub fn symbol_id(&self) -> &str {
        &self.symbol_id
    }
    #[must_use]
    pub const fn start_byte(&self) -> u64 {
        self.start_byte
    }
    #[must_use]
    pub const fn end_byte(&self) -> u64 {
        self.end_byte
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionCommandPhaseV3 {
    Preview,
    Apply,
}

/// Source-bound rename intent. `expected_snapshot` and `preview_token` are
/// untrusted preconditions; this DTO never conveys a mutation grant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionRenameCommandV3 {
    pub(super) target: ExecutionSymbolTargetV3,
    pub(super) new_name: String,
    pub(super) phase: ExecutionCommandPhaseV3,
    pub(super) expected_snapshot: String,
    pub(super) preview_token: Option<String>,
}
impl ExecutionRenameCommandV3 {
    #[must_use]
    pub const fn target(&self) -> &ExecutionSymbolTargetV3 {
        &self.target
    }
    #[must_use]
    pub fn new_name(&self) -> &str {
        &self.new_name
    }
    #[must_use]
    pub const fn phase(&self) -> ExecutionCommandPhaseV3 {
        self.phase
    }
    #[must_use]
    pub fn expected_snapshot(&self) -> &str {
        &self.expected_snapshot
    }
    #[must_use]
    pub fn preview_token(&self) -> Option<&str> {
        self.preview_token.as_deref()
    }
}

pub(super) fn project_command(command: &ResolvedRenameCommandV3) -> ExecutionRenameCommandV3 {
    let target = command.target();
    ExecutionRenameCommandV3 {
        target: ExecutionSymbolTargetV3 {
            selector: target.clone_selector_storage_v3(),
            repo_path: target.clone_repo_path_storage_v3(),
            symbol_id: target.clone_symbol_id_storage_v3(),
            start_byte: target.start_byte(),
            end_byte: target.end_byte(),
        },
        new_name: command.new_name().to_owned(),
        phase: match command.phase() {
            RenamePhaseV3::Preview => ExecutionCommandPhaseV3::Preview,
            RenamePhaseV3::Apply => ExecutionCommandPhaseV3::Apply,
        },
        expected_snapshot: command.expected_snapshot().to_owned(),
        preview_token: command.preview_token().map(str::to_owned),
    }
}
