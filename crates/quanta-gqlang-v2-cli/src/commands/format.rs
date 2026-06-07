pub fn run(input: &str) -> Result<String, String> {
    let doc = quanta_gqlang_v2_ir::frontend::parse_document_v1(input)
        .map_err(|page| format!("syntax-error\n{}", render_syntax_page_v1(&page)))?;
    Ok(quanta_gqlang_v2_ir::frontend::format_document_v1(&doc))
}

fn render_syntax_page_v1(page: &quanta_gqlang_v2_ir::frontend::SyntaxDiagnosticPageV1) -> String {
    page.diagnostics
        .iter()
        .enumerate()
        .map(|(index, diagnostic)| {
            let span = diagnostic
                .span
                .as_ref()
                .map_or_else(|| "none".to_string(), |span| format!("{}..{}", span.start, span.end));
            format!(
                "syntax.diagnostic[{index}].code={}\nsyntax.diagnostic[{index}].message={}\nsyntax.diagnostic[{index}].span={span}",
                diagnostic.code, diagnostic.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
