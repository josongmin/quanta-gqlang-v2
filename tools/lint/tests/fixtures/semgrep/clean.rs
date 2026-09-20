pub fn clean_control() -> Result<(), CleanDiagnostic> {
    Ok(())
}

pub struct CleanDiagnostic;

pub const QUERY_RESULT_COUNT: &str = "query_result_count";
pub const NO_RESULT_REASON: &str = "no_result_reason";
pub const TABLE_ROWS: &str = "table_rows=5";
