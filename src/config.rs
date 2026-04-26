use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub fn get_config_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path.push("config.yml");
    path
}

pub fn load_config<T>() -> T
where
    T: for<'de> Deserialize<'de> + Serialize + Default,
{
    let path = get_config_path();
    if !path.exists() {
        let default_config = T::default();
        if let Ok(yaml) = serde_yml::to_string(&default_config) {
            let _ = fs::write(&path, yaml);
        }
        return default_config;
    }

    match fs::read_to_string(&path) {
        Ok(content) => serde_yml::from_str(&content).unwrap_or_else(|_| {
            let default_config = T::default();
            if let Ok(yaml) = serde_yml::to_string(&default_config) {
                let _ = fs::write(&path, yaml);
            }
            default_config
        }),
        Err(_) => T::default(),
    }
}

pub fn save_config<T>(config: &T) -> Result<(), String>
where
    T: Serialize,
{
    let path = get_config_path();
    let yaml = serde_yml::to_string(config).map_err(|e| e.to_string())?;
    fs::write(path, yaml).map_err(|e| e.to_string())
}
