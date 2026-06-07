use crate::entity_schema::{
    validate_name_v1, validate_path_segments_v1, validate_unique_field_names_v1, PublicEntityRefV1,
    PublicFactSchemaAvailabilityV1, PublicFactSchemaSourceProvenanceV1,
    PublicFactSchemaValidationErrorV1, PublicFieldArityV1, PublicFieldTypeV1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicRelationRefV1 {
    pub path: Vec<String>,
}

impl PublicRelationRefV1 {
    #[must_use]
    pub fn canonical_name_v1(&self) -> String {
        self.path.join(".")
    }

    pub fn validate_v1(&self) -> Result<(), PublicFactSchemaValidationErrorV1> {
        validate_path_segments_v1(&self.path, "relation_path")
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PublicRelationDirectionV1 {
    Directed,
    Bidirectional,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PublicRelationEndpointCardinalityV1 {
    One,
    Many,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicRelationEndpointSchemaV1 {
    pub entity: PublicEntityRefV1,
    pub role: String,
    pub cardinality: PublicRelationEndpointCardinalityV1,
}

impl PublicRelationEndpointSchemaV1 {
    pub fn validate_v1(&self) -> Result<(), PublicFactSchemaValidationErrorV1> {
        self.entity.validate_v1()?;
        validate_name_v1(&self.role, "relation_endpoint_role")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicRelationFieldSchemaV1 {
    pub name: String,
    pub value_type: PublicFieldTypeV1,
    pub arity: PublicFieldArityV1,
    pub provenance: PublicFactSchemaSourceProvenanceV1,
    pub availability: PublicFactSchemaAvailabilityV1,
}

impl PublicRelationFieldSchemaV1 {
    pub fn validate_v1(&self) -> Result<(), PublicFactSchemaValidationErrorV1> {
        validate_name_v1(&self.name, "relation_field_name")?;
        self.value_type.validate_v1()?;
        self.provenance.validate_v1()?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicRelationSchemaV1 {
    pub relation: PublicRelationRefV1,
    pub source: PublicRelationEndpointSchemaV1,
    pub target: PublicRelationEndpointSchemaV1,
    pub direction: PublicRelationDirectionV1,
    pub fields: Vec<PublicRelationFieldSchemaV1>,
    pub provenance: PublicFactSchemaSourceProvenanceV1,
    pub availability: PublicFactSchemaAvailabilityV1,
}

impl PublicRelationSchemaV1 {
    pub fn validate_v1(&self) -> Result<(), PublicFactSchemaValidationErrorV1> {
        self.relation.validate_v1()?;
        self.source.validate_v1()?;
        self.target.validate_v1()?;
        self.provenance.validate_v1()?;

        for field in &self.fields {
            field.validate_v1()?;
        }

        validate_unique_field_names_v1(
            &self
                .fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
            "relation_field_name_duplicate",
            "relation field names must be unique",
        )
    }
}
