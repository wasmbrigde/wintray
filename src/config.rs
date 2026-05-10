use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Returns the default path for the configuration file (next to the executable).
pub fn get_config_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path.push("config.toml");
    path
}

/// Loads a TOML configuration file into a struct.
///
/// If the file does not exist, it creates it with the default values
/// of the specified type `T`.
pub fn load_config<T>() -> T
where
    T: for<'de> Deserialize<'de> + Serialize + Default,
{
    let path = get_config_path();
    if !path.exists() {
        let default_config = T::default();
        if let Ok(content) = toml::to_string(&default_config) {
            let _ = fs::write(&path, content);
        }
        return default_config;
    }

    match fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).unwrap_or_else(|_| {
            let default_config = T::default();
            if let Ok(content) = toml::to_string(&default_config) {
                let _ = fs::write(&path, content);
            }
            default_config
        }),
        Err(_) => T::default(),
    }
}

/// Saves the specified configuration struct to the TOML configuration file.
pub fn save_config<T>(config: &T) -> Result<(), String>
where
    T: Serialize,
{
    let path = get_config_path();
    let content = toml::to_string(config).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}
