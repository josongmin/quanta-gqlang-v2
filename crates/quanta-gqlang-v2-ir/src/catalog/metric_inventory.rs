/// E11C external metric vocabulary, re-exported from sema so the catalog plane
/// exposes the named inventory alongside the internal projection counters.
pub use quanta_gqlang_v2_sema::metric_vocabulary::{
    metric_descriptor_v1 as external_metric_descriptor_v1,
    MetricDescriptorV1 as ExternalMetricDescriptorV1, MetricFamilyV1 as ExternalMetricFamilyV1,
    EXTERNAL_METRIC_VOCABULARY_V1,
};
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MetricIdV1 {
    QueryResultCount,
    CandidateSetCount,
    EvidenceSetCount,
    CatalogCoverageRatio,
    CompareAgreementRatio,
    ExternalFindingCount,
    ExternalHighSeverityCount,
    SuppressedFindingCount,
}

impl MetricIdV1 {
    #[must_use]
    pub const fn as_str_v1(self) -> &'static str {
        match self {
            Self::QueryResultCount => "query_result_count",
            Self::CandidateSetCount => "candidate_set_count",
            Self::EvidenceSetCount => "evidence_set_count",
            Self::CatalogCoverageRatio => "catalog_coverage_ratio",
            Self::CompareAgreementRatio => "compare_agreement_ratio",
            Self::ExternalFindingCount => "external_finding_count",
            Self::ExternalHighSeverityCount => "external_high_severity_count",
            Self::SuppressedFindingCount => "suppressed_finding_count",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MetricClassV1 {
    Volume,
    Coverage,
    Quality,
    ExternalSignal,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MetricCategoryV1 {
    QueryResult,
    WorkingSet,
    Compare,
    Catalog,
    ExternalFindings,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MetricScalarKindV1 {
    Count,
    Ratio,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MetricUnitV1 {
    Items,
    Percent,
}

impl MetricUnitV1 {
    #[must_use]
    pub const fn matches_scalar_kind_v1(self, scalar_kind: MetricScalarKindV1) -> bool {
        matches!(
            (self, scalar_kind),
            (Self::Items, MetricScalarKindV1::Count) | (Self::Percent, MetricScalarKindV1::Ratio)
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MetricAggregationHintV1 {
    Sum,
    Latest,
    Average,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MetricProvenanceV1 {
    QueryProjection,
    WorkingSetProjection,
    CompareProjection,
    CatalogSnapshot,
    ExternalFindingCatalog,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MetricDescriptorV1 {
    pub id: MetricIdV1,
    pub class: MetricClassV1,
    pub category: MetricCategoryV1,
    pub scalar_kind: MetricScalarKindV1,
    pub unit: MetricUnitV1,
    pub aggregation_hint: MetricAggregationHintV1,
    pub provenance: MetricProvenanceV1,
}

impl MetricDescriptorV1 {
    fn validate_v1(&self) -> Result<(), MetricInventoryValidationErrorV1> {
        if !self.unit.matches_scalar_kind_v1(self.scalar_kind) {
            return Err(MetricInventoryValidationErrorV1::new_v1(
                "metric_unit_scalar_mismatch",
                "Metric unit must match the declared scalar kind",
            ));
        }

        if matches!(self.category, MetricCategoryV1::ExternalFindings)
            && !matches!(self.class, MetricClassV1::ExternalSignal)
        {
            return Err(MetricInventoryValidationErrorV1::new_v1(
                "metric_external_category_requires_external_class",
                "External finding metrics must use the external-signal class",
            ));
        }

        if matches!(self.provenance, MetricProvenanceV1::ExternalFindingCatalog)
            != matches!(self.category, MetricCategoryV1::ExternalFindings)
        {
            return Err(MetricInventoryValidationErrorV1::new_v1(
                "metric_external_provenance_category_mismatch",
                "External finding provenance must stay paired with the external-findings category",
            ));
        }

        Ok(())
    }
}

const METRIC_INVENTORY_ROWS_V1: [MetricDescriptorV1; 8] = [
    MetricDescriptorV1 {
        id: MetricIdV1::QueryResultCount,
        class: MetricClassV1::Volume,
        category: MetricCategoryV1::QueryResult,
        scalar_kind: MetricScalarKindV1::Count,
        unit: MetricUnitV1::Items,
        aggregation_hint: MetricAggregationHintV1::Sum,
        provenance: MetricProvenanceV1::QueryProjection,
    },
    MetricDescriptorV1 {
        id: MetricIdV1::CandidateSetCount,
        class: MetricClassV1::Volume,
        category: MetricCategoryV1::WorkingSet,
        scalar_kind: MetricScalarKindV1::Count,
        unit: MetricUnitV1::Items,
        aggregation_hint: MetricAggregationHintV1::Latest,
        provenance: MetricProvenanceV1::WorkingSetProjection,
    },
    MetricDescriptorV1 {
        id: MetricIdV1::EvidenceSetCount,
        class: MetricClassV1::Volume,
        category: MetricCategoryV1::WorkingSet,
        scalar_kind: MetricScalarKindV1::Count,
        unit: MetricUnitV1::Items,
        aggregation_hint: MetricAggregationHintV1::Latest,
        provenance: MetricProvenanceV1::WorkingSetProjection,
    },
    MetricDescriptorV1 {
        id: MetricIdV1::CatalogCoverageRatio,
        class: MetricClassV1::Coverage,
        category: MetricCategoryV1::Catalog,
        scalar_kind: MetricScalarKindV1::Ratio,
        unit: MetricUnitV1::Percent,
        aggregation_hint: MetricAggregationHintV1::Average,
        provenance: MetricProvenanceV1::CatalogSnapshot,
    },
    MetricDescriptorV1 {
        id: MetricIdV1::CompareAgreementRatio,
        class: MetricClassV1::Quality,
        category: MetricCategoryV1::Compare,
        scalar_kind: MetricScalarKindV1::Ratio,
        unit: MetricUnitV1::Percent,
        aggregation_hint: MetricAggregationHintV1::Average,
        provenance: MetricProvenanceV1::CompareProjection,
    },
    MetricDescriptorV1 {
        id: MetricIdV1::ExternalFindingCount,
        class: MetricClassV1::ExternalSignal,
        category: MetricCategoryV1::ExternalFindings,
        scalar_kind: MetricScalarKindV1::Count,
        unit: MetricUnitV1::Items,
        aggregation_hint: MetricAggregationHintV1::Sum,
        provenance: MetricProvenanceV1::ExternalFindingCatalog,
    },
    MetricDescriptorV1 {
        id: MetricIdV1::ExternalHighSeverityCount,
        class: MetricClassV1::ExternalSignal,
        category: MetricCategoryV1::ExternalFindings,
        scalar_kind: MetricScalarKindV1::Count,
        unit: MetricUnitV1::Items,
        aggregation_hint: MetricAggregationHintV1::Sum,
        provenance: MetricProvenanceV1::ExternalFindingCatalog,
    },
    MetricDescriptorV1 {
        id: MetricIdV1::SuppressedFindingCount,
        class: MetricClassV1::ExternalSignal,
        category: MetricCategoryV1::ExternalFindings,
        scalar_kind: MetricScalarKindV1::Count,
        unit: MetricUnitV1::Items,
        aggregation_hint: MetricAggregationHintV1::Sum,
        provenance: MetricProvenanceV1::ExternalFindingCatalog,
    },
];

pub const METRIC_INVENTORY_V1: &[MetricDescriptorV1] = &METRIC_INVENTORY_ROWS_V1;

#[must_use]
pub const fn metric_inventory_rows_v1() -> &'static [MetricDescriptorV1] {
    METRIC_INVENTORY_V1
}

#[must_use]
pub fn metric_descriptor_v1(id: MetricIdV1) -> Option<&'static MetricDescriptorV1> {
    let mut index = 0;
    while let Some(row) = METRIC_INVENTORY_ROWS_V1.get(index) {
        if row.id == id {
            return Some(row);
        }
        index += 1;
    }
    None
}

pub fn validate_metric_inventory_v1(
    rows: &[MetricDescriptorV1],
) -> Result<(), MetricInventoryValidationErrorV1> {
    if rows.is_empty() {
        return Err(MetricInventoryValidationErrorV1::new_v1(
            "metric_inventory_empty",
            "Metric inventory must expose at least one public row",
        ));
    }

    let mut left = 0;
    while let Some(left_row) = rows.get(left) {
        left_row.validate_v1()?;
        let mut right = left + 1;
        while let Some(right_row) = rows.get(right) {
            if left_row.id == right_row.id {
                return Err(MetricInventoryValidationErrorV1::new_v1(
                    "metric_inventory_duplicate_id",
                    "Metric inventory ids must remain unique",
                ));
            }
            right += 1;
        }
        left += 1;
    }

    Ok(())
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MetricInventoryValidationErrorV1 {
    pub code: &'static str,
    pub message: &'static str,
}

impl MetricInventoryValidationErrorV1 {
    const fn new_v1(code: &'static str, message: &'static str) -> Self {
        Self { code, message }
    }
}
