//! Typed private NDJSON protocol. This is not JSON-RPC or LSP.

use quanta_gqlang_v2_frontend_contract::{FrontendBudgetV2, OpenAnalysisAuthorityV2};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::handlers::v3_diagnostics::{handle_v3_diagnostics_v2, AnalysisPayloadV2};

pub const READ_ONLY_METHOD_NAMES_V2: [&str; 1] = ["v3_diagnostics"];
const RESPONSE_LIMIT_FALLBACK_V2: &[u8] =
    br#"{"id":null,"ok":false,"error":{"code":"response_limit_exceeded","message":"response exceeds configured limit"}}"#;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestV2 {
    pub id: Value,
    pub method: String,
    pub source_id: u32,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WireDiagnosticV2 {
    pub stage: &'static str,
    pub code: &'static str,
    pub severity: &'static str,
    pub message: String,
    pub source_id: u32,
    pub span_start: usize,
    pub span_end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProtocolErrorV2 {
    pub code: &'static str,
    pub message: String,
}

impl ProtocolErrorV2 {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Serialize)]
struct SuccessResponseV2<'a> {
    id: &'a Value,
    ok: bool,
    method: &'static str,
    #[serde(flatten)]
    payload: AnalysisPayloadV2,
}

#[derive(Debug, Serialize)]
struct ErrorResponseV2<'a> {
    id: &'a Value,
    ok: bool,
    error: ProtocolErrorV2,
}

#[derive(Debug)]
pub struct DiagnosticsServerV2 {
    authority: OpenAnalysisAuthorityV2,
}

impl DiagnosticsServerV2 {
    pub fn try_standard() -> Result<Self, ProtocolErrorV2> {
        let budget = FrontendBudgetV2::standard_v2().map_err(|error| {
            ProtocolErrorV2::new(
                "configuration_error",
                format!("{}: {}", error.code(), error.message()),
            )
        })?;
        Self::try_with_budget(budget)
    }

    pub fn try_with_budget(budget: FrontendBudgetV2) -> Result<Self, ProtocolErrorV2> {
        if usize::try_from(budget.max_response_bytes()).unwrap_or(usize::MAX)
            < RESPONSE_LIMIT_FALLBACK_V2.len()
        {
            return Err(ProtocolErrorV2::new(
                "configuration_error",
                format!(
                    "max_response_bytes must be at least {}",
                    RESPONSE_LIMIT_FALLBACK_V2.len()
                ),
            ));
        }
        let authority = OpenAnalysisAuthorityV2::try_empty_v2(budget).map_err(|error| {
            ProtocolErrorV2::new(
                "configuration_error",
                format!("{}: {}", error.code(), error.message()),
            )
        })?;
        Ok(Self { authority })
    }

    #[must_use]
    pub const fn budget(&self) -> &FrontendBudgetV2 {
        self.authority.budget()
    }

    pub fn dispatch(&self, line: &[u8]) -> Result<Vec<u8>, ProtocolErrorV2> {
        let request: RequestV2 = serde_json::from_slice(line)
            .map_err(|error| ProtocolErrorV2::new("invalid_request", error.to_string()))?;
        validate_id(&request.id)?;
        if request.method != READ_ONLY_METHOD_NAMES_V2[0] {
            return self.serialize_error(
                &request.id,
                ProtocolErrorV2::new(
                    "method_not_allowed",
                    format!("method `{}` is not allowed", request.method),
                ),
            );
        }
        let source_limit = usize::try_from(self.budget().max_source_bytes()).unwrap_or(usize::MAX);
        if request.source.len() > source_limit {
            return self.serialize_error(
                &request.id,
                ProtocolErrorV2::new(
                    "source_limit_exceeded",
                    format!("source exceeds max_source_bytes {source_limit}"),
                ),
            );
        }
        match handle_v3_diagnostics_v2(request.source_id, &request.source, &self.authority) {
            Ok(payload) => self.serialize_bounded(&SuccessResponseV2 {
                id: &request.id,
                ok: true,
                method: READ_ONLY_METHOD_NAMES_V2[0],
                payload,
            }),
            Err(error) => self.serialize_error(&request.id, error),
        }
    }

    pub fn render_transport_error(&self, error: ProtocolErrorV2) -> Vec<u8> {
        self.serialize_error(&Value::Null, error)
            .unwrap_or_else(|_| RESPONSE_LIMIT_FALLBACK_V2.to_vec())
    }

    fn serialize_error(
        &self,
        id: &Value,
        error: ProtocolErrorV2,
    ) -> Result<Vec<u8>, ProtocolErrorV2> {
        self.serialize_bounded(&ErrorResponseV2 {
            id,
            ok: false,
            error,
        })
    }

    fn serialize_bounded<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, ProtocolErrorV2> {
        let bytes = serde_json::to_vec(value)
            .map_err(|error| ProtocolErrorV2::new("serialization_error", error.to_string()))?;
        let limit = usize::try_from(self.budget().max_response_bytes()).unwrap_or(usize::MAX);
        if bytes.len() > limit {
            return Err(ProtocolErrorV2::new(
                "response_limit_exceeded",
                format!("serialized response exceeds max_response_bytes {limit}"),
            ));
        }
        Ok(bytes)
    }
}

fn validate_id(id: &Value) -> Result<(), ProtocolErrorV2> {
    if matches!(id, Value::String(_) | Value::Number(_)) {
        Ok(())
    } else {
        Err(ProtocolErrorV2::new(
            "invalid_id",
            "id must be a JSON string or number",
        ))
    }
}
