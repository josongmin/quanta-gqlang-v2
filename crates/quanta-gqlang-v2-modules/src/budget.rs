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

/// Typed, allocation-free budget refusal used before request predeclaration
/// creates an ID, output vector, or rich diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ModuleRequestBudgetRefusalV3 {
    pub(crate) dimension: &'static str,
    pub(crate) limit: usize,
    pub(crate) observed: usize,
}

impl ModuleBudgetSessionV2 {
    /// Read-only gate for the exact charges made by one reference lookup.
    pub(crate) fn preflight_reference_v3(
        &self,
        name: &str,
        steps: usize,
    ) -> Result<(), ModuleRequestBudgetRefusalV3> {
        let width = self.budget.profile.authority_string_bytes();
        if name.len() > width {
            return Err(ModuleRequestBudgetRefusalV3 {
                dimension: "authority_string_bytes",
                limit: width,
                observed: name.len(),
            });
        }
        for (current, amount, limit, dimension) in [
            (
                self.authority_bytes,
                name.len(),
                self.budget.profile.authority_bytes(),
                "authority_bytes",
            ),
            (
                self.steps,
                steps,
                self.budget.profile.module_steps(),
                "steps",
            ),
        ] {
            let observed = current.saturating_add(amount);
            if observed > limit {
                return Err(ModuleRequestBudgetRefusalV3 {
                    dimension,
                    limit,
                    observed,
                });
            }
        }
        Ok(())
    }

    pub(crate) fn preflight_predeclaration_v3<'a>(
        &mut self,
        names: impl Iterator<Item = (&'a str, &'a str)>,
        count: usize,
    ) -> Result<(), ModuleRequestBudgetRefusalV3> {
        fn within(
            current: usize,
            amount: usize,
            limit: usize,
            dimension: &'static str,
        ) -> Result<usize, ModuleRequestBudgetRefusalV3> {
            let observed = current.saturating_add(amount);
            if observed > limit {
                Err(ModuleRequestBudgetRefusalV3 {
                    dimension,
                    limit,
                    observed,
                })
            } else {
                Ok(observed)
            }
        }

        let mut nodes = self.nodes;
        let mut bytes = self.authority_bytes;
        for (module_id, local_name) in names {
            nodes = within(nodes, 1, self.budget.profile.module_nodes(), "nodes")?;
            for value in [module_id, local_name] {
                let width = self.budget.profile.authority_string_bytes();
                if value.len() > width {
                    return Err(ModuleRequestBudgetRefusalV3 {
                        dimension: "authority_string_bytes",
                        limit: width,
                        observed: value.len(),
                    });
                }
                bytes = within(
                    bytes,
                    value.len(),
                    self.budget.profile.authority_bytes(),
                    "authority_bytes",
                )?;
            }
        }
        let steps = within(
            self.steps,
            count,
            self.budget.profile.module_steps(),
            "steps",
        )?;
        self.nodes = nodes;
        self.authority_bytes = bytes;
        self.steps = steps;
        Ok(())
    }

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
        self.charge_steps(1)
    }

    /// Charges several logical authority steps without making the accounting
    /// operation itself linear in `amount`.
    ///
    /// The failure payload reports the first rejected step (`limit + 1`),
    /// exactly as repeated calls to [`Self::charge_step`] would. The counter is
    /// left unchanged on failure.
    pub(crate) fn charge_steps(&mut self, amount: usize) -> Result<(), ModuleDiagnosticV2> {
        let limit = self.budget.profile.module_steps();
        let remaining = limit.saturating_sub(self.steps);
        if amount > remaining {
            let observed = limit.saturating_add(1);
            return Err(ModuleDiagnosticV2::resource_exhausted(
                "steps", limit, observed,
            ));
        }
        self.steps += amount;
        Ok(())
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
        let observed = current.saturating_add(amount);
        if observed > limit {
            return Err(ModuleDiagnosticV2::resource_exhausted(
                dimension, limit, observed,
            ));
        }
        *current = observed;
        Ok(())
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "owner-local budget boundary assertions")]
mod tests {
    use super::*;

    fn session(step_limit: usize) -> ModuleBudgetSessionV2 {
        ModuleBudgetV2::from_profile(
            AuthorityBudgetProfileV2::try_new(1, 1, step_limit, 1, 1).unwrap(),
        )
        .begin_session_v2()
    }

    #[test]
    fn bulk_step_charge_accepts_exact_limit() {
        let mut session = session(3);
        session.charge_steps(3).unwrap();
        assert_eq!(session.steps, 3);
    }

    #[test]
    fn bulk_step_charge_reports_first_rejected_step_without_mutating() {
        let mut session = session(3);
        session.charge_step().unwrap();

        let error = session.charge_steps(usize::MAX).unwrap_err();
        let exhaustion = error.resource_exhaustion().unwrap();
        assert_eq!(exhaustion.dimension(), "steps");
        assert_eq!(exhaustion.limit(), 3);
        assert_eq!(exhaustion.observed(), 4);
        assert_eq!(session.steps, 1);
    }
}
