use quanta_gqlang_v2_cli::doctor;

#[test]
fn doctor_separates_parse_errors() {
    let report = doctor::run_v1("").expect("doctor parse classification");

    assert_eq!(report.status.as_str_v1(), "parse-error");
    assert!(!report.parse_diagnostics.is_empty());
    assert!(report.query_summary.is_none());
}

#[test]
fn doctor_separates_sema_errors() {
    let report = doctor::run_v1("FIND path api.entry -> db.write | depth=0")
        .expect("doctor sema classification");

    assert_eq!(report.status.as_str_v1(), "sema-error");
    assert!(report
        .sema_diagnostics
        .iter()
        .any(|diag| diag.contains("greater than zero")));
    assert!(report.query_summary.is_none());
}

#[test]
fn doctor_reports_no_result_and_degraded_reasons_without_fake_runtime_truth() {
    let report =
        doctor::run_v1("EXPLAIN rule::owasp.sql-injection ;; evidence=trace ;; truth=confirm")
            .expect("doctor no-result classification");

    assert_eq!(report.status.as_str_v1(), "no-result");
    assert_eq!(
        report.no_result_reason.as_deref(),
        Some("read-only-shell-has-no-execution-backend")
    );
    assert!(report
        .degraded_reasons
        .iter()
        .any(|reason| reason == "truth-evidence-materialization-unavailable"));
    assert!(report.query_summary.is_some());
}
