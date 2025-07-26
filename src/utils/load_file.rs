pub fn load_file_as_vec_u32(file_path: &str) -> Vec<u32> {
    let u8_bytes: Vec<u8> = std::fs::read(file_path).expect("failed to load file");
    let u32_bytes: Vec<u32> = u8_bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().expect("failed to convert u8 to u32")))
        .collect();
    u32_bytes
}
