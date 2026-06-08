#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicFactSchemaValidationErrorV1 {
    pub code: &'static str,
    pub message: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PublicFactSchemaSourceKindV1 {
    Declared,
    Imported,
    Observed,
    ExampleDerived,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicFactSchemaSourceProvenanceV1 {
    pub kind: PublicFactSchemaSourceKindV1,
    pub source_ref: String,
}

impl PublicFactSchemaSourceProvenanceV1 {
    pub fn validate_v1(&self) -> Result<(), PublicFactSchemaValidationErrorV1> {
        if self.source_ref.trim().is_empty() {
            return Err(PublicFactSchemaValidationErrorV1 {
                code: "fact_schema_source_ref_empty",
                message: "fact schema provenance source_ref must not be empty".to_string(),
            });
        }

        validate_no_runtime_rowset_marker_v1(
            &self.source_ref,
            "fact_schema_source_ref_runtime_local",
            "fact schema provenance source_ref",
        )?;

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PublicFactSchemaAvailabilityV1 {
    Available,
    Degraded {
        reason: PublicFactSchemaDegradedReasonV1,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PublicFactSchemaDegradedReasonV1 {
    PartialCoverage,
    PermissionFiltered,
    SourceUnavailable,
    ExampleDerivedOnly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicEntityRefV1 {
    pub path: Vec<String>,
}

impl PublicEntityRefV1 {
    #[must_use]
    pub fn canonical_name_v1(&self) -> String {
        self.path.join(".")
    }

    pub fn validate_v1(&self) -> Result<(), PublicFactSchemaValidationErrorV1> {
        validate_path_segments_v1(&self.path, "entity_path")?;
        for segment in &self.path {
            validate_no_runtime_rowset_marker_v1(
                segment,
                "entity_path_runtime_local",
                "entity path segment",
            )?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PublicScalarTypeV1 {
    Boolean,
    Integer,
    Float,
    String,
    Bytes,
    Date,
    Timestamp,
    Json,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PublicFieldTypeV1 {
    Scalar(PublicScalarTypeV1),
    EntityRef(PublicEntityRefV1),
}

impl PublicFieldTypeV1 {
    pub fn validate_v1(&self) -> Result<(), PublicFactSchemaValidationErrorV1> {
        match self {
            Self::Scalar(_) => Ok(()),
            Self::EntityRef(entity) => entity.validate_v1(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PublicFieldArityV1 {
    Required,
    Optional,
    Repeated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicEntityFieldSchemaV1 {
    pub name: String,
    pub value_type: PublicFieldTypeV1,
    pub arity: PublicFieldArityV1,
    pub provenance: PublicFactSchemaSourceProvenanceV1,
    pub availability: PublicFactSchemaAvailabilityV1,
}

impl PublicEntityFieldSchemaV1 {
    pub fn validate_v1(&self) -> Result<(), PublicFactSchemaValidationErrorV1> {
        validate_name_v1(&self.name, "entity_field_name")?;
        validate_no_runtime_rowset_marker_v1(
            &self.name,
            "entity_field_name_runtime_local",
            "entity field name",
        )?;
        self.value_type.validate_v1()?;
        self.provenance.validate_v1()?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicEntitySchemaV1 {
    pub entity: PublicEntityRefV1,
    pub fields: Vec<PublicEntityFieldSchemaV1>,
    pub provenance: PublicFactSchemaSourceProvenanceV1,
    pub availability: PublicFactSchemaAvailabilityV1,
}

impl PublicEntitySchemaV1 {
    pub fn validate_v1(&self) -> Result<(), PublicFactSchemaValidationErrorV1> {
        self.entity.validate_v1()?;
        self.provenance.validate_v1()?;

        if self.fields.is_empty() {
            return Err(PublicFactSchemaValidationErrorV1 {
                code: "entity_fields_empty",
                message: format!(
                    "entity `{}` must publish at least one field",
                    self.entity.canonical_name_v1()
                ),
            });
        }

        for field in &self.fields {
            field.validate_v1()?;
        }

        validate_unique_field_names_v1(
            &self
                .fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
            "entity_field_name_duplicate",
            "entity field names must be unique",
        )
    }
}

pub(crate) fn validate_name_v1(
    value: &str,
    code: &'static str,
) -> Result<(), PublicFactSchemaValidationErrorV1> {
    if value.trim().is_empty() {
        return Err(PublicFactSchemaValidationErrorV1 {
            code,
            message: format!("{code} must not be empty"),
        });
    }

    Ok(())
}

pub(crate) fn validate_path_segments_v1(
    path: &[String],
    code: &'static str,
) -> Result<(), PublicFactSchemaValidationErrorV1> {
    if path.is_empty() {
        return Err(PublicFactSchemaValidationErrorV1 {
            code,
            message: format!("{code} must include at least one segment"),
        });
    }

    for segment in path {
        if segment.trim().is_empty() {
            return Err(PublicFactSchemaValidationErrorV1 {
                code,
                message: format!("{code} must not contain empty segments"),
            });
        }
    }

    Ok(())
}

pub(crate) fn validate_unique_field_names_v1(
    field_names: &[&str],
    code: &'static str,
    message: &'static str,
) -> Result<(), PublicFactSchemaValidationErrorV1> {
    let mut seen = std::collections::BTreeSet::new();
    for field_name in field_names {
        if !seen.insert(*field_name) {
            return Err(PublicFactSchemaValidationErrorV1 {
                code,
                message: message.to_string(),
            });
        }
    }

    Ok(())
}

pub(crate) fn validate_no_runtime_rowset_marker_v1(
    value: &str,
    code: &'static str,
    subject: &'static str,
) -> Result<(), PublicFactSchemaValidationErrorV1> {
    // Match the `rowset` runtime-transport marker only as a whole token delimited
    // by non-alphanumeric separators (`_`, `.`, `::`, whitespace, …). Collapsing
    // every separator before a substring `contains("rowset")` produced false
    // positives across word boundaries (e.g. `escrow set` -> `escrowset`).
    let leaks_rowset_token = value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .any(|segment| {
            segment.eq_ignore_ascii_case("rowset") || segment.eq_ignore_ascii_case("rowsets")
        });
    if leaks_rowset_token {
        return Err(PublicFactSchemaValidationErrorV1 {
            code,
            message: format!("{subject} must not leak runtime-only rowset transport"),
        });
    }

    Ok(())
}
