use std::path::PathBuf;

pub fn project_dir(add: &str) -> PathBuf {
    let mut output = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    output.push(add);
    output
}
