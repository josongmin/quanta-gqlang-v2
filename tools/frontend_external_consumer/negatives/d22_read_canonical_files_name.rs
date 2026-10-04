//! A borrowed source observation has no public construction fields.
use quanta_gqlang_v2_frontend_contract::AdmittedCanonicalFilesScalarReadV3;

fn probe(read: &AdmittedCanonicalFilesScalarReadV3<'_>) {
    let _ = read.source_name;
}

fn main() {
    let _ = probe as fn(&AdmittedCanonicalFilesScalarReadV3<'_>);
}
