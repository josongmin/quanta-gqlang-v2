//! Resource accounting shared by every phase of one authority build.

use crate::ModuleDiagnosticV2;
use quanta_gqlang_v2_types::AuthorityBudgetProfileV2;

/// Hard limits for one module-authority transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleBudgetV2 {
    profile: AuthorityBudgetProfileV2,
}

impl ModuleBudgetV2 {
    #[must_use]
    pub fn from_profile(profile: AuthorityBudgetProfileV2) -> Self {
        Self { profile }
    }

    #[must_use]
    pub(crate) fn begin_session_v2(&self) -> ModuleBudgetSessionV2 {
        ModuleBudgetSessionV2 {
            budget: self.clone(),
            nodes: 0,
            edges: 0,
            steps: 0,
            authority_bytes: 0,
        }
    }
}

/// Opaque counter owned by a public session-owning builder.
#[derive(Debug)]
pub(crate) struct ModuleBudgetSessionV2 {
    budget: ModuleBudgetV2,
    nodes: usize,
    edges: usize,
    steps: usize,
    authority_bytes: usize,
}

impl ModuleBudgetSessionV2 {
    pub(crate) fn charge_node(&mut self) -> Result<(), ModuleDiagnosticV2> {
        Self::charge(
            &mut self.nodes,
            1,
            self.budget.profile.module_nodes(),
            "nodes",
        )
    }

    pub(crate) fn charge_edge(&mut self) -> Result<(), ModuleDiagnosticV2> {
        Self::charge(
            &mut self.edges,
            1,
            self.budget.profile.module_edges(),
            "edges",
        )
    }

    pub(crate) fn charge_step(&mut self) -> Result<(), ModuleDiagnosticV2> {
        Self::charge(
            &mut self.steps,
            1,
            self.budget.profile.module_steps(),
            "steps",
        )
    }

    pub(crate) fn charge_bytes(&mut self, bytes: usize) -> Result<(), ModuleDiagnosticV2> {
        Self::charge(
            &mut self.authority_bytes,
            bytes,
            self.budget.profile.authority_bytes(),
            "authority_bytes",
        )
    }

    pub(crate) fn charge_string(&mut self, value: &str) -> Result<(), ModuleDiagnosticV2> {
        self.check_string_width(value)?;
        self.charge_bytes(value.len())
    }

    /// Applies the per-string bound when the enclosing raw buffer was already
    /// charged at ingress.
    pub(crate) fn check_string_width(&self, value: &str) -> Result<(), ModuleDiagnosticV2> {
        let observed = value.len();
        if observed > self.budget.profile.authority_string_bytes() {
            return Err(ModuleDiagnosticV2::resource_exhausted(
                "authority_string_bytes",
                self.budget.profile.authority_string_bytes(),
                observed,
            ));
        }
        Ok(())
    }

    fn charge(
        current: &mut usize,
        amount: usize,
        limit: usize,
        dimension: &'static str,
    ) -> Result<(), ModuleDiagnosticV2> {
        let observed = current.checked_add(amount).unwrap_or(usize::MAX);
        if observed > limit {
            return Err(ModuleDiagnosticV2::resource_exhausted(
                dimension, limit, observed,
            ));
        }
        *current = observed;
        Ok(())
    }
}
