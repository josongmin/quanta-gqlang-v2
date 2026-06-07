pub use crate::doctor::DoctorReportV1;

pub fn run(input: &str) -> Result<DoctorReportV1, String> {
    crate::doctor::run_v1(input)
}
