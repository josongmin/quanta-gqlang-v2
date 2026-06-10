#![forbid(unsafe_code)]

pub mod ast;
pub mod declarations;
pub mod diagnostics;
pub mod error;
pub mod formatter;
pub mod interactive;
pub mod lexer;
pub mod parser;
pub mod root;
pub mod roots;
pub mod span;
pub mod token;

pub const CRATE_ID_V1: &str = "quanta-gqlang-v2-syntax";

pub use ast::{AstRootV1, DocumentAstV1};
pub use diagnostics::{SyntaxDiagnosticPageV1, SyntaxDiagnosticV1};
pub use error::ParseErrorV1;
pub use formatter::format_document_v1;
pub use lexer::tokenize_v1;
pub use parser::{document_statements_v1, parse_document_v1};
pub use root::RootKindV1;
pub use token::{TokenKindV1, TokenV1};

#[cfg(test)]
mod lib_tests;
