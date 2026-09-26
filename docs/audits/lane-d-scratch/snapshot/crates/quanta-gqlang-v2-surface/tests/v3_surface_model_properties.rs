//! Generated valid syntax with a test-owned expression model.
//!
//! The model renders source without using the production formatter, then
//! checks the parser's HIR against the model. Every generated case is valid by
//! construction, so a parser rejection cannot silently skip a round trip.

use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use quanta_gqlang_v2_hir::{
    check_tree, HirBuilderV1, HirRootRefV1, LiteralV1, ValueExprHirV1, ValueExprKindV1,
};
use quanta_gqlang_v2_surface::{format_item, parse_expr, parse_program, SurfaceItemV1};

#[derive(Clone, Debug)]
enum ModelExpr {
    Name(&'static str),
    Integer(u16),
    Add(Box<Self>, Box<Self>),
    Multiply(Box<Self>, Box<Self>),
}

impl ModelExpr {
    fn source(&self) -> String {
        match self {
            Self::Name(name) => (*name).to_owned(),
            Self::Integer(value) => value.to_string(),
            Self::Add(left, right) => format!("({} + {})", left.source(), right.source()),
            Self::Multiply(left, right) => format!("({} * {})", left.source(), right.source()),
        }
    }

    fn precedence(&self) -> u8 {
        match self {
            Self::Name(_) | Self::Integer(_) => 3,
            Self::Multiply(_, _) => 2,
            Self::Add(_, _) => 1,
        }
    }

    /// Independent precedence-aware spelling: omit grouping only where the
    /// model tree is preserved by left associativity and `*` above `+`.
    fn minimally_grouped_source(&self, parent_precedence: u8, right_child: bool) -> String {
        let precedence = self.precedence();
        let ungrouped = match self {
            Self::Name(name) => (*name).to_owned(),
            Self::Integer(value) => value.to_string(),
            Self::Add(left, right) | Self::Multiply(left, right) => {
                let operator = if matches!(self, Self::Add(_, _)) {
                    "+"
                } else {
                    "*"
                };
                format!(
                    "{} {operator} {}",
                    left.minimally_grouped_source(precedence, false),
                    right.minimally_grouped_source(precedence, true)
                )
            }
        };
        if precedence < parent_precedence || (right_child && precedence == parent_precedence) {
            format!("({ungrouped})")
        } else {
            ungrouped
        }
    }
}

fn generated_expr() -> impl Strategy<Value = ModelExpr> {
    let atom = prop_oneof![
        prop::sample::select(vec!["a", "bb", "record_1"]).prop_map(ModelExpr::Name),
        (0u16..1000).prop_map(ModelExpr::Integer),
    ];
    atom.prop_recursive(4, 48, 2, |inner| {
        (inner.clone(), inner, any::<bool>()).prop_map(|(left, right, multiply)| {
            if multiply {
                ModelExpr::Multiply(Box::new(left), Box::new(right))
            } else {
                ModelExpr::Add(Box::new(left), Box::new(right))
            }
        })
    })
}

/// Keep a binary root in every case, including after shrinking.
fn generated_binary_expr() -> impl Strategy<Value = ModelExpr> {
    (generated_expr(), generated_expr(), any::<bool>()).prop_map(|(left, right, multiply)| {
        if multiply {
            ModelExpr::Multiply(Box::new(left), Box::new(right))
        } else {
            ModelExpr::Add(Box::new(left), Box::new(right))
        }
    })
}

fn matches_model(model: &ModelExpr, parsed: &ValueExprHirV1) -> Result<(), TestCaseError> {
    // Parentheses are an authored syntax node, while the model represents the
    // expression's semantic grouping. Inspect every non-parenthesis payload.
    if let ValueExprKindV1::Paren(inner) = &parsed.kind {
        return matches_model(model, inner);
    }
    match (model, &parsed.kind) {
        (ModelExpr::Name(expected), ValueExprKindV1::Symbol(actual)) => {
            prop_assert_eq!(actual.name.as_str(), *expected);
        }
        (ModelExpr::Integer(expected), ValueExprKindV1::Literal(LiteralV1::Int(actual))) => {
            prop_assert_eq!(actual, &expected.to_string());
        }
        (ModelExpr::Add(left, right), ValueExprKindV1::Binary(actual)) => {
            prop_assert_eq!(actual.op.symbol.as_str(), "+");
            matches_model(left, &actual.lhs)?;
            matches_model(right, &actual.rhs)?;
        }
        (ModelExpr::Multiply(left, right), ValueExprKindV1::Binary(actual)) => {
            prop_assert_eq!(actual.op.symbol.as_str(), "*");
            matches_model(left, &actual.lhs)?;
            matches_model(right, &actual.rhs)?;
        }
        _ => prop_assert!(false, "model={model:?}; parsed={parsed:?}"),
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    #[test]
    fn generated_valid_expression_matches_independent_model(model in generated_binary_expr()) {
        for expression_source in [model.source(), model.minimally_grouped_source(0, false)] {
            let expression = parse_expr(&expression_source, &HirBuilderV1::new());
            prop_assert!(expression.is_ok(), "generated valid expression rejected: {expression_source:?}: {expression:?}");
            let expression = expression.map_err(|error| TestCaseError::fail(format!("{error:?}")))?;
            matches_model(&model, &expression)?;
            prop_assert!(check_tree(HirRootRefV1::Value(&expression), expression_source.len()).is_ok());

            let declaration_source = format!("let generated = {expression_source}");
            let parsed = parse_program(&declaration_source, &HirBuilderV1::new());
            prop_assert!(parsed.is_ok(), "generated declaration rejected: {declaration_source:?}: {parsed:?}");
            let parsed = parsed.map_err(|error| TestCaseError::fail(format!("{error:?}")))?;
            let SurfaceItemV1::Decl(declaration) = &parsed.item else {
                prop_assert!(false, "generated declaration changed family: {:?}", parsed.item);
                return Ok(());
            };
            matches_model(&model, &declaration.body)?;

            let formatted = format_item(&parsed.item.to_program_item());
            let reparsed = parse_program(&formatted, &HirBuilderV1::new());
            prop_assert!(reparsed.is_ok(), "formatted declaration rejected: {formatted:?}: {reparsed:?}");
            let reparsed = reparsed.map_err(|error| TestCaseError::fail(format!("{error:?}")))?;
            let SurfaceItemV1::Decl(reparsed_decl) = &reparsed.item else {
                prop_assert!(false, "formatted declaration changed family: {:?}", reparsed.item);
                return Ok(());
            };
            matches_model(&model, &reparsed_decl.body)?;
            prop_assert_eq!(format_item(&reparsed.item.to_program_item()), formatted);
        }
    }
}
