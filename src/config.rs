use serde::Deserialize;
use std::{collections::HashMap, fs, net::Ipv4Addr};
use serde_json;
#[derive(Deserialize)]
pub struct Server {
    pub bind_port: u16,
    pub bind_address: String,
}
#[derive(Deserialize)]
pub struct LanguageConfig {
    pub name: String,
    pub file_name: String,
    pub command:Vec<String>,
}
#[derive(Deserialize)]
pub struct Case {
   score:u64,
   input_file:String,
   answer_file:String,
   time_limit:u64,
   memory_limit:u64,
}
#[derive(Deserialize)]
pub struct Problem {
    pub id: u64,
    pub name: String,
    #[serde(rename="type")]
    pub problem_type:String,
    pub misc:HashMap<String,String>,
    pub cases:Vec<Case>,

}
#[derive(Deserialize)]
pub struct Config {
    pub languages: Vec<LanguageConfig>,
    pub problems: Vec<Problem>,
    pub server: Server,

}

impl Config {
    pub fn from_file(path: &str) -> Self {
        let config_str = fs::read_to_string(path).expect("Unable to read config file");
        serde_json::from_str(&config_str).expect("Unable to parse config file")
    }
}
