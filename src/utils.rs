use crate::constants::{SHADER_OUTPUT_DIR, SHADER_SOURCE_DIR};

pub fn compiled_spirv_path_for_source(source_path: &str) -> String {
    let prefix = format!("{}/", SHADER_SOURCE_DIR);
    let rel = source_path
        .strip_prefix(prefix.as_str())
        .unwrap_or(source_path);
    format!("{}/{}.spv", SHADER_OUTPUT_DIR, rel)
}
