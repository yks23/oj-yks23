use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct LanguageConfig {
    pub compile: String,
    pub run: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub languages: std::collections::HashMap<String, LanguageConfig>,
    pub timeout: u64,
    pub port: u16,
}

impl Config {
    pub fn from_file(path: &str) -> Self {
         println!("Loading config file from: {}", path); // 添加这行代码调试路径
        let config_str = fs::read_to_string(path).expect("Unable to read config file");
        serde_json::from_str(&config_str).expect("Unable to parse config file")
    }
}
