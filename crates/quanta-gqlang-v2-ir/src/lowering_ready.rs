use crate::planning::{PublicPlanRootKindV1, PublicPlanV1, PublicPlanValidationErrorV1};
use crate::root_ir::PublicIrRootV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoweringReadyIrV1 {
    pub root: PublicIrRootV1,
    pub planning: Option<PublicPlanV1>,
}

impl LoweringReadyIrV1 {
    pub fn new_v1(
        root: PublicIrRootV1,
        planning: Option<PublicPlanV1>,
    ) -> Result<Self, LoweringReadyIrErrorV1> {
        let ready = Self { root, planning };
        ready.validate_v1()?;
        Ok(ready)
    }

    pub fn validate_v1(&self) -> Result<(), LoweringReadyIrErrorV1> {
        let root_kind = self.root_kind_v1();
        match self.planning.as_ref() {
            None if root_kind.requires_planning_metadata_v1() => {
                return Err(LoweringReadyIrErrorV1::new_v1(
                    "missing_planning_metadata",
                    format!(
                        "{root_kind:?} lowering-ready IR must preserve public planning metadata"
                    ),
                ));
            }
            None => return Ok(()),
            Some(planning) => {
                planning.validate_v1()?;
                if planning.root != root_kind {
                    return Err(LoweringReadyIrErrorV1::new_v1(
                        "planning_root_mismatch",
                        format!(
                            "lowering-ready planning root {:?} does not match IR root {:?}",
                            planning.root, root_kind
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn normalized_v1(&self) -> Result<Self, LoweringReadyIrErrorV1> {
        self.validate_v1()?;
        Ok(self.clone())
    }

    #[must_use]
    pub fn root_kind_v1(&self) -> PublicPlanRootKindV1 {
        match &self.root {
            PublicIrRootV1::Select(_) => PublicPlanRootKindV1::Select,
            PublicIrRootV1::Find(_) => PublicPlanRootKindV1::Find,
            PublicIrRootV1::Check(_) => PublicPlanRootKindV1::Check,
            PublicIrRootV1::Explain(_) => PublicPlanRootKindV1::Explain,
            PublicIrRootV1::Retrieval(_) => PublicPlanRootKindV1::Retrieval,
            PublicIrRootV1::Interactive(_) => PublicPlanRootKindV1::Interactive,
            PublicIrRootV1::Declaration(_) => PublicPlanRootKindV1::Declaration,
            PublicIrRootV1::Meta(_) => PublicPlanRootKindV1::Meta,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoweringReadyIrErrorV1 {
    pub code: &'static str,
    pub message: String,
}

impl LoweringReadyIrErrorV1 {
    fn new_v1(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl From<PublicPlanValidationErrorV1> for LoweringReadyIrErrorV1 {
    fn from(value: PublicPlanValidationErrorV1) -> Self {
        Self {
            code: value.code,
            message: value.message,
        }
    }
}
