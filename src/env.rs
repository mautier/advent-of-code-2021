const ENV_DATA_DIR: &str = "AOC_DATA_DIR";

pub fn get_data_dir() -> std::path::PathBuf {
    match std::env::var(ENV_DATA_DIR) {
        Ok(path) => path.into(),
        Err(err) => panic!("Failed to read env var {}: {:?}", ENV_DATA_DIR, err),
    }
}

pub fn get_puzzle_input_path(filename: &str) -> std::path::PathBuf {
    let mut path = get_data_dir();
    path.push(filename);
    path
}
