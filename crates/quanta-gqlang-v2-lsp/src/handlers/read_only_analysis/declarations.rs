use super::super::ReadOnlyRootSummaryV1;
use super::common::declaration_summary_v1;

pub(super) fn declaration_summary_from_carrier_v1(
    keyword: &'static str,
    carrier: &str,
) -> ReadOnlyRootSummaryV1 {
    // The carrier already parsed, so its clause structure is valid; split it only to
    // present the declared name and clauses (display formatting, not validation).
    let name = carrier.split(";;").next().unwrap_or(carrier).trim();
    let clauses = carrier
        .split(";;")
        .skip(1)
        .map(|clause| clause.trim().to_string())
        .filter(|clause| !clause.is_empty())
        .collect();
    declaration_summary_v1(keyword, name, clauses)
}
