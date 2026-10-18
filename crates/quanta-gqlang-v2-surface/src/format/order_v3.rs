//! Canonical order-stage formatting uses the existing value formatter.

use super::format_value;
use quanta_gqlang_v2_hir::{OrderByHirV1, SortDirectionHirV1};

pub(super) fn render_order_v3(order: &OrderByHirV1, out: &mut String) {
    out.push_str(" | order by ");
    for (index, key) in order.keys().iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(&format_value(&key.value));
        out.push(' ');
        out.push_str(match key.direction {
            SortDirectionHirV1::Asc => "asc",
            SortDirectionHirV1::Desc => "desc",
        });
        if key.null_order == quanta_gqlang_v2_hir::NullOrderHirV1::First {
            out.push_str(" nulls first");
        }
    }
}
