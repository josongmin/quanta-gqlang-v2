//! Conversion from source order keys to the existing checked order contract.

use super::{NullOrderV2, SortDirectionV2};
use quanta_gqlang_v2_hir::{NullOrderHirV1, SortDirectionHirV1};

impl From<NullOrderHirV1> for NullOrderV2 {
    fn from(order: NullOrderHirV1) -> Self {
        match order {
            NullOrderHirV1::First => Self::First,
            NullOrderHirV1::Last => Self::Last,
        }
    }
}

impl From<SortDirectionHirV1> for SortDirectionV2 {
    fn from(direction: SortDirectionHirV1) -> Self {
        match direction {
            SortDirectionHirV1::Asc => Self::Asc,
            SortDirectionHirV1::Desc => Self::Desc,
        }
    }
}
