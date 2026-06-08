#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectRootAstV1 {
    // Shared parser ownership still passes the raw SELECT root through this field.
    pub from: String,
}

pub const SELECT_ACCEPTED_FORM_V1: &str =
    "SELECT <projection> FROM <table> [WHERE <field> <op> <literal>] [LIMIT <u64>]";

pub const SELECT_REJECTED_FORMS_V1: &[&str] = &[
    "JOIN and multiple-table FROM clauses",
    "table aliases with or without AS",
    "ORDER BY, GROUP BY, HAVING, and OFFSET clauses",
    "boolean-composed filters such as AND/OR",
    "subqueries and projection expressions",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectRootErrorV1 {
    pub code: &'static str,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectQueryShapeV1 {
    pub table: SelectTableRefAstV1,
    pub projection: SelectProjectionAstV1,
    pub filter: Option<SelectFilterAstV1>,
    pub limit: Option<u64>,
}

impl SelectQueryShapeV1 {
    #[must_use]
    pub fn canonical_query_v1(&self) -> String {
        let projection = match &self.projection {
            SelectProjectionAstV1::Wildcard => "*".to_string(),
            SelectProjectionAstV1::Fields(fields) => fields
                .iter()
                .map(SelectFieldRefAstV1::dotted_name_v1)
                .collect::<Vec<_>>()
                .join(", "),
        };
        let mut query = format!("SELECT {projection} FROM {}", self.table.dotted_name_v1());
        if let Some(filter) = &self.filter {
            query.push_str(" WHERE ");
            query.push_str(&filter.field.dotted_name_v1());
            query.push(' ');
            query.push_str(filter.op.as_str_v1());
            query.push(' ');
            query.push_str(&filter.value.to_query_fragment_v1());
        }
        if let Some(limit) = self.limit {
            query.push_str(" LIMIT ");
            query.push_str(&limit.to_string());
        }
        query
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectTableRefAstV1 {
    pub path: Vec<String>,
}

impl SelectTableRefAstV1 {
    #[must_use]
    pub fn dotted_name_v1(&self) -> String {
        self.path.join(".")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectFieldRefAstV1 {
    pub path: Vec<String>,
}

impl SelectFieldRefAstV1 {
    #[must_use]
    pub fn dotted_name_v1(&self) -> String {
        self.path.join(".")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectProjectionAstV1 {
    Wildcard,
    Fields(Vec<SelectFieldRefAstV1>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectFilterAstV1 {
    pub field: SelectFieldRefAstV1,
    pub op: SelectFilterOpV1,
    pub value: SelectLiteralAstV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectFilterOpV1 {
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
}

impl SelectFilterOpV1 {
    #[must_use]
    pub const fn as_str_v1(&self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::NotEq => "!=",
            Self::Lt => "<",
            Self::Lte => "<=",
            Self::Gt => ">",
            Self::Gte => ">=",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectLiteralAstV1 {
    String(String),
    Integer(i64),
    Boolean(bool),
}

impl SelectLiteralAstV1 {
    #[must_use]
    pub fn to_query_fragment_v1(&self) -> String {
        match self {
            Self::String(value) => format!("'{}'", value.replace('\'', "''")),
            Self::Integer(value) => value.to_string(),
            Self::Boolean(value) => value.to_string(),
        }
    }
}

impl SelectRootAstV1 {
    pub fn parse_v1(input: &str) -> Result<Self, SelectRootErrorV1> {
        let shape = parse_select_shape_v1(input)?;
        Ok(Self {
            from: shape.canonical_query_v1(),
        })
    }

    #[must_use]
    pub fn raw_query_v1(&self) -> &str {
        &self.from
    }

    pub fn shape_v1(&self) -> Result<SelectQueryShapeV1, SelectRootErrorV1> {
        parse_select_root_carrier_v1(&self.from)
    }
}

pub fn parse_select_root_v1(input: &str) -> Result<SelectRootAstV1, SelectRootErrorV1> {
    SelectRootAstV1::parse_v1(input)
}

pub fn parse_select_shape_v1(input: &str) -> Result<SelectQueryShapeV1, SelectRootErrorV1> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(select_error_v1("empty_select_root", "SELECT root is empty"));
    }

    let mut cursor = SelectTokenCursorV1::new(tokenize_select_v1(trimmed)?);
    cursor.expect_keyword_v1(
        "SELECT",
        "expected_select_keyword",
        "SELECT root must start with SELECT",
    )?;
    let projection = parse_projection_v1(&mut cursor)?;
    cursor.expect_keyword_v1(
        "FROM",
        "expected_from_keyword",
        "SELECT projection must be followed by FROM",
    )?;
    let table = SelectTableRefAstV1 {
        path: parse_identifier_path_v1(
            &mut cursor,
            "expected_table_name",
            "expected a table name after FROM",
        )?,
    };
    reject_unsupported_table_shape_v1(&cursor)?;

    let filter = if cursor.consume_keyword_v1("WHERE") {
        let predicate = parse_filter_v1(&mut cursor)?;
        reject_boolean_filter_tail_v1(&cursor)?;
        Some(predicate)
    } else {
        None
    };

    let limit = if cursor.consume_keyword_v1("LIMIT") {
        Some(parse_limit_v1(&mut cursor)?)
    } else {
        None
    };

    if cursor.consume_token_v1(&SelectTokenV1::Semicolon) && cursor.is_done_v1() {
        return Ok(SelectQueryShapeV1 {
            table,
            projection,
            filter,
            limit,
        });
    }

    if !cursor.is_done_v1() {
        return Err(unexpected_trailing_tokens_v1(&cursor));
    }

    Ok(SelectQueryShapeV1 {
        table,
        projection,
        filter,
        limit,
    })
}

fn parse_select_root_carrier_v1(input: &str) -> Result<SelectQueryShapeV1, SelectRootErrorV1> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(select_error_v1("empty_select_root", "SELECT root is empty"));
    }

    let Some((head, _tail)) = split_head_tail_v1(trimmed) else {
        return if trimmed.eq_ignore_ascii_case("SELECT") {
            parse_select_shape_v1(trimmed)
        } else {
            parse_select_tail_shape_v1(trimmed)
        };
    };
    if head.eq_ignore_ascii_case("SELECT") {
        parse_select_shape_v1(trimmed)
    } else {
        parse_select_tail_shape_v1(trimmed)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SelectTokenV1 {
    Word(String),
    StringLiteral(String),
    Comma,
    Dot,
    Star,
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
    Minus,
    Semicolon,
}

#[derive(Clone, Debug)]
struct SelectTokenCursorV1 {
    tokens: Vec<SelectTokenV1>,
    offset: usize,
}

impl SelectTokenCursorV1 {
    fn new(tokens: Vec<SelectTokenV1>) -> Self {
        Self { tokens, offset: 0 }
    }

    fn peek_v1(&self) -> Option<&SelectTokenV1> {
        self.tokens.get(self.offset)
    }

    fn next_v1(&mut self) -> Option<SelectTokenV1> {
        let token = self.tokens.get(self.offset).cloned();
        if token.is_some() {
            self.offset += 1;
        }
        token
    }

    fn is_done_v1(&self) -> bool {
        self.peek_v1().is_none()
    }

    fn consume_token_v1(&mut self, expected: &SelectTokenV1) -> bool {
        if self.peek_v1().is_some_and(|token| token == expected) {
            self.offset += 1;
            return true;
        }
        false
    }

    fn consume_keyword_v1(&mut self, keyword: &str) -> bool {
        if self
            .peek_v1()
            .is_some_and(|token| matches!(token, SelectTokenV1::Word(value) if value.eq_ignore_ascii_case(keyword)))
        {
            self.offset += 1;
            return true;
        }
        false
    }

    fn expect_keyword_v1(
        &mut self,
        keyword: &str,
        code: &'static str,
        message: &'static str,
    ) -> Result<(), SelectRootErrorV1> {
        if self.consume_keyword_v1(keyword) {
            return Ok(());
        }
        Err(select_error_v1(code, message))
    }
}

fn tokenize_select_v1(input: &str) -> Result<Vec<SelectTokenV1>, SelectRootErrorV1> {
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();
    while let Some(ch) = chars.next() {
        if ch.is_whitespace() {
            continue;
        }
        match ch {
            ',' => tokens.push(SelectTokenV1::Comma),
            '.' => tokens.push(SelectTokenV1::Dot),
            '*' => tokens.push(SelectTokenV1::Star),
            '=' => tokens.push(SelectTokenV1::Eq),
            ';' => tokens.push(SelectTokenV1::Semicolon),
            '-' => tokens.push(SelectTokenV1::Minus),
            '!' => {
                if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(SelectTokenV1::NotEq);
                } else {
                    return Err(select_error_v1(
                        "invalid_select_character",
                        "only != is supported after ! in SELECT filters",
                    ));
                }
            }
            '<' => {
                if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(SelectTokenV1::Lte);
                } else {
                    tokens.push(SelectTokenV1::Lt);
                }
            }
            '>' => {
                if matches!(chars.peek(), Some('=')) {
                    chars.next();
                    tokens.push(SelectTokenV1::Gte);
                } else {
                    tokens.push(SelectTokenV1::Gt);
                }
            }
            '\'' => {
                let mut value = String::new();
                loop {
                    match chars.next() {
                        Some('\'') if matches!(chars.peek(), Some('\'')) => {
                            chars.next();
                            value.push('\'');
                        }
                        Some('\'') => break,
                        Some(inner) => value.push(inner),
                        None => {
                            return Err(select_error_v1(
                                "unterminated_string_literal",
                                "unterminated single-quoted string in SELECT filter",
                            ));
                        }
                    }
                }
                tokens.push(SelectTokenV1::StringLiteral(value));
            }
            _ if is_identifier_start_v1(ch) => {
                let mut word = String::from(ch);
                while let Some(next) = chars.peek() {
                    if is_identifier_continue_v1(*next) {
                        word.push(*next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(SelectTokenV1::Word(word));
            }
            _ if ch.is_ascii_digit() => {
                let mut digits = String::from(ch);
                while let Some(next) = chars.peek() {
                    if next.is_ascii_digit() {
                        digits.push(*next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(SelectTokenV1::Word(digits));
            }
            _ => {
                return Err(select_error_v1(
                    "invalid_select_character",
                    format!("unsupported character {ch:?} in SELECT root"),
                ));
            }
        }
    }
    Ok(tokens)
}

fn parse_select_tail_shape_v1(input: &str) -> Result<SelectQueryShapeV1, SelectRootErrorV1> {
    parse_select_shape_v1(&format!("SELECT {input}"))
}

fn parse_projection_v1(
    cursor: &mut SelectTokenCursorV1,
) -> Result<SelectProjectionAstV1, SelectRootErrorV1> {
    if cursor.consume_token_v1(&SelectTokenV1::Star) {
        if cursor.consume_token_v1(&SelectTokenV1::Comma) {
            return Err(select_error_v1(
                "unsupported_projection_shape",
                "SELECT wildcard projection cannot be mixed with explicit fields",
            ));
        }
        return Ok(SelectProjectionAstV1::Wildcard);
    }

    let mut fields = Vec::new();
    fields.push(SelectFieldRefAstV1 {
        path: parse_identifier_path_v1(
            cursor,
            "expected_projection",
            "expected at least one projection field after SELECT",
        )?,
    });
    while cursor.consume_token_v1(&SelectTokenV1::Comma) {
        if cursor.consume_token_v1(&SelectTokenV1::Star) {
            return Err(select_error_v1(
                "unsupported_projection_shape",
                "SELECT wildcard projection cannot be mixed with explicit fields",
            ));
        }
        fields.push(SelectFieldRefAstV1 {
            path: parse_identifier_path_v1(
                cursor,
                "expected_projection",
                "expected a projection field after ,",
            )?,
        });
    }
    Ok(SelectProjectionAstV1::Fields(fields))
}

fn parse_identifier_path_v1(
    cursor: &mut SelectTokenCursorV1,
    code: &'static str,
    message: &'static str,
) -> Result<Vec<String>, SelectRootErrorV1> {
    let mut path = Vec::new();
    path.push(parse_identifier_segment_v1(cursor, code, message)?);
    while cursor.consume_token_v1(&SelectTokenV1::Dot) {
        path.push(parse_identifier_segment_v1(
            cursor,
            "invalid_identifier",
            "expected an identifier segment after .",
        )?);
    }
    Ok(path)
}

fn parse_identifier_segment_v1(
    cursor: &mut SelectTokenCursorV1,
    code: &'static str,
    message: &'static str,
) -> Result<String, SelectRootErrorV1> {
    match cursor.next_v1() {
        Some(SelectTokenV1::Word(value))
            if is_identifier_word_v1(&value) && !is_reserved_keyword_v1(&value) =>
        {
            Ok(value)
        }
        Some(SelectTokenV1::Word(value)) if is_reserved_keyword_v1(&value) => Err(select_error_v1(
            "invalid_identifier",
            format!("{value} is reserved in the current SELECT root grammar"),
        )),
        _ => Err(select_error_v1(code, message)),
    }
}

fn parse_filter_v1(
    cursor: &mut SelectTokenCursorV1,
) -> Result<SelectFilterAstV1, SelectRootErrorV1> {
    let field = SelectFieldRefAstV1 {
        path: parse_identifier_path_v1(
            cursor,
            "expected_filter_field",
            "expected a filter field after WHERE",
        )?,
    };
    let op = parse_filter_op_v1(cursor)?;
    let value = parse_filter_literal_v1(cursor)?;
    Ok(SelectFilterAstV1 { field, op, value })
}

fn parse_filter_op_v1(
    cursor: &mut SelectTokenCursorV1,
) -> Result<SelectFilterOpV1, SelectRootErrorV1> {
    match cursor.next_v1() {
        Some(SelectTokenV1::Eq) => Ok(SelectFilterOpV1::Eq),
        Some(SelectTokenV1::NotEq) => Ok(SelectFilterOpV1::NotEq),
        Some(SelectTokenV1::Lt) => Ok(SelectFilterOpV1::Lt),
        Some(SelectTokenV1::Lte) => Ok(SelectFilterOpV1::Lte),
        Some(SelectTokenV1::Gt) => Ok(SelectFilterOpV1::Gt),
        Some(SelectTokenV1::Gte) => Ok(SelectFilterOpV1::Gte),
        _ => Err(select_error_v1(
            "expected_filter_operator",
            "expected one of =, !=, <, <=, >, >= after WHERE field",
        )),
    }
}

fn parse_filter_literal_v1(
    cursor: &mut SelectTokenCursorV1,
) -> Result<SelectLiteralAstV1, SelectRootErrorV1> {
    match cursor.next_v1() {
        Some(SelectTokenV1::StringLiteral(value)) => Ok(SelectLiteralAstV1::String(value)),
        Some(SelectTokenV1::Word(value)) if value.eq_ignore_ascii_case("true") => {
            Ok(SelectLiteralAstV1::Boolean(true))
        }
        Some(SelectTokenV1::Word(value)) if value.eq_ignore_ascii_case("false") => {
            Ok(SelectLiteralAstV1::Boolean(false))
        }
        Some(SelectTokenV1::Word(value)) if value.chars().all(|ch| ch.is_ascii_digit()) => value
            .parse::<i64>()
            .map(SelectLiteralAstV1::Integer)
            .map_err(|_| {
                select_error_v1(
                    "invalid_integer_literal",
                    format!("integer literal {value} does not fit into i64"),
                )
            }),
        Some(SelectTokenV1::Minus) => match cursor.next_v1() {
            Some(SelectTokenV1::Word(value)) if value.chars().all(|ch| ch.is_ascii_digit()) => {
                let signed = format!("-{value}");
                signed
                    .parse::<i64>()
                    .map(SelectLiteralAstV1::Integer)
                    .map_err(|_| {
                        select_error_v1(
                            "invalid_integer_literal",
                            format!("integer literal {signed} does not fit into i64"),
                        )
                    })
            }
            _ => Err(select_error_v1(
                "expected_filter_value",
                "expected digits after - in SELECT filter literal",
            )),
        },
        _ => Err(select_error_v1(
            "expected_filter_value",
            "expected a string, integer, or boolean literal after SELECT filter operator",
        )),
    }
}

fn parse_limit_v1(cursor: &mut SelectTokenCursorV1) -> Result<u64, SelectRootErrorV1> {
    match cursor.next_v1() {
        Some(SelectTokenV1::Word(value)) if value.chars().all(|ch| ch.is_ascii_digit()) => {
            value.parse::<u64>().map_err(|_| {
                select_error_v1(
                    "invalid_limit_value",
                    format!("LIMIT value {value} does not fit into u64"),
                )
            })
        }
        Some(SelectTokenV1::Minus) => Err(select_error_v1(
            "invalid_limit_value",
            "LIMIT must be a non-negative integer",
        )),
        _ => Err(select_error_v1(
            "expected_limit_value",
            "expected a non-negative integer after LIMIT",
        )),
    }
}

fn reject_unsupported_table_shape_v1(
    cursor: &SelectTokenCursorV1,
) -> Result<(), SelectRootErrorV1> {
    match cursor.peek_v1() {
        Some(SelectTokenV1::Word(value)) if value.eq_ignore_ascii_case("JOIN") => {
            Err(select_error_v1(
                "unsupported_table_shape",
                "JOIN is not supported; SELECT roots accept exactly one table in FROM",
            ))
        }
        Some(SelectTokenV1::Word(value)) if value.eq_ignore_ascii_case("AS") => {
            Err(select_error_v1(
                "unsupported_table_shape",
                "table aliases are not supported in SELECT roots",
            ))
        }
        Some(SelectTokenV1::Word(value))
            if !value.eq_ignore_ascii_case("WHERE") && !value.eq_ignore_ascii_case("LIMIT") =>
        {
            Err(select_error_v1(
                "unsupported_table_shape",
                "table aliases are not supported in SELECT roots",
            ))
        }
        _ => Ok(()),
    }
}

fn reject_boolean_filter_tail_v1(cursor: &SelectTokenCursorV1) -> Result<(), SelectRootErrorV1> {
    match cursor.peek_v1() {
        Some(SelectTokenV1::Word(value))
            if value.eq_ignore_ascii_case("AND") || value.eq_ignore_ascii_case("OR") =>
        {
            Err(select_error_v1(
                "unsupported_filter_boolean",
                "SELECT roots support at most one WHERE predicate without AND/OR composition",
            ))
        }
        _ => Ok(()),
    }
}

fn unexpected_trailing_tokens_v1(cursor: &SelectTokenCursorV1) -> SelectRootErrorV1 {
    match cursor.peek_v1() {
        Some(SelectTokenV1::Word(value)) if value.eq_ignore_ascii_case("ORDER") => select_error_v1(
            "unsupported_clause",
            "ORDER BY is not supported in SELECT roots",
        ),
        Some(SelectTokenV1::Word(value)) if value.eq_ignore_ascii_case("GROUP") => select_error_v1(
            "unsupported_clause",
            "GROUP BY is not supported in SELECT roots",
        ),
        Some(SelectTokenV1::Word(value))
            if value.eq_ignore_ascii_case("HAVING") || value.eq_ignore_ascii_case("OFFSET") =>
        {
            select_error_v1(
                "unsupported_clause",
                format!("{value} is not supported in SELECT roots"),
            )
        }
        Some(SelectTokenV1::Word(value)) if value.eq_ignore_ascii_case("JOIN") => select_error_v1(
            "unsupported_table_shape",
            "JOIN is not supported; SELECT roots accept exactly one table in FROM",
        ),
        Some(other) => select_error_v1(
            "unexpected_trailing_tokens",
            format!("unexpected trailing token {other:?} in SELECT root"),
        ),
        None => select_error_v1(
            "unexpected_trailing_tokens",
            "unexpected trailing tokens in SELECT root",
        ),
    }
}

fn is_identifier_start_v1(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_identifier_continue_v1(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn is_identifier_word_v1(word: &str) -> bool {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) if is_identifier_start_v1(first) => chars.all(is_identifier_continue_v1),
        _ => false,
    }
}

fn split_head_tail_v1(input: &str) -> Option<(&str, &str)> {
    input
        .split_once(char::is_whitespace)
        .map(|(head, tail)| (head.trim(), tail.trim()))
}

fn is_reserved_keyword_v1(word: &str) -> bool {
    matches!(
        word.to_ascii_uppercase().as_str(),
        "SELECT"
            | "FROM"
            | "WHERE"
            | "LIMIT"
            | "JOIN"
            | "AS"
            | "ORDER"
            | "BY"
            | "GROUP"
            | "HAVING"
            | "OFFSET"
            | "AND"
            | "OR"
    )
}

fn select_error_v1(code: &'static str, message: impl Into<String>) -> SelectRootErrorV1 {
    SelectRootErrorV1 {
        code,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "SELECT syntax tests assert shape decoding success for validated fixtures"
    )]

    use super::{
        parse_select_root_v1, parse_select_shape_v1, SelectFieldRefAstV1, SelectFilterAstV1,
        SelectFilterOpV1, SelectLiteralAstV1, SelectProjectionAstV1, SelectQueryShapeV1,
        SelectRootAstV1, SelectTableRefAstV1,
    };

    #[test]
    fn select_shape_captures_projection_filter_limit() {
        let root = parse_select_root_v1(
            " SELECT id, profile.name FROM catalog.users WHERE active = true LIMIT 25 ",
        )
        .expect("SELECT root should parse");
        let shape = root.shape_v1().expect("shape should parse");
        assert_eq!(
            shape,
            SelectQueryShapeV1 {
                table: SelectTableRefAstV1 {
                    path: vec!["catalog".to_string(), "users".to_string()],
                },
                projection: SelectProjectionAstV1::Fields(vec![
                    SelectFieldRefAstV1 {
                        path: vec!["id".to_string()],
                    },
                    SelectFieldRefAstV1 {
                        path: vec!["profile".to_string(), "name".to_string()],
                    },
                ]),
                filter: Some(SelectFilterAstV1 {
                    field: SelectFieldRefAstV1 {
                        path: vec!["active".to_string()],
                    },
                    op: SelectFilterOpV1::Eq,
                    value: SelectLiteralAstV1::Boolean(true),
                }),
                limit: Some(25),
            }
        );
        assert_eq!(
            root.raw_query_v1(),
            "SELECT id, profile.name FROM catalog.users WHERE active = true LIMIT 25",
        );
    }

    #[test]
    fn select_shape_rejects_table_aliases() {
        let err = parse_select_shape_v1("SELECT id FROM catalog.users u")
            .expect_err("implicit aliases should be rejected");
        assert_eq!(err.code, "unsupported_table_shape");
    }

    #[test]
    fn select_shape_rejects_boolean_filter_composition() {
        let err = parse_select_shape_v1(
            "SELECT id FROM catalog.users WHERE active = true AND region = 'us'",
        )
        .expect_err("AND/OR filters should be rejected");
        assert_eq!(err.code, "unsupported_filter_boolean");
    }

    #[test]
    fn select_shape_accepts_shared_tail_only_carrier() {
        let root = SelectRootAstV1 {
            from: "id FROM catalog.users LIMIT 3".to_string(),
        };
        let shape = root.shape_v1().expect("tail-only carrier should parse");
        assert_eq!(shape.table.path, vec!["catalog", "users"]);
        assert_eq!(shape.limit, Some(3));
    }
}
