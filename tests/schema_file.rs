//! The committed schema is generated, so it has to still be what the types say.

/// A change to the types that forgets to regenerate it is a defect in itself:
/// editors would read one contract while the binary enforced another.
#[test]
fn the_committed_schema_is_what_the_types_say() {
    let generated = qctl::schema::generated().expect("generate");
    let committed = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(qctl::schema::COMMITTED),
    )
    .expect("read committed schema");
    assert_eq!(
        committed, generated,
        "run `qctl schema` and commit the result"
    );
}
