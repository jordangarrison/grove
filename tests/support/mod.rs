use std::path::PathBuf;

pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

pub fn read_fixture(name: &str) -> std::io::Result<String> {
    std::fs::read_to_string(fixture_path(name))
}
