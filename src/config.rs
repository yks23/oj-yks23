use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};
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
#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Case {
   pub score:f64,
   pub input_file:String,
   pub answer_file:String,
   pub time_limit:u64,
   pub memory_limit:u64,
}
impl  Case {
    pub fn to_new(&self)->Case{
        Case{score:self.score,input_file:self.input_file.clone(),answer_file:self.answer_file.clone(),time_limit:self.time_limit,memory_limit:self.memory_limit}
    }
    pub fn new()->Case{
        Case{score:0.0,input_file:"".to_string(),answer_file:"".to_string(),time_limit:0,memory_limit:0}
    }
}
#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct Misc {
   pub packing:Option<Vec<Vec<usize>>>,
   pub special_judge:Option<Vec<String>>,
}
#[derive(Deserialize)]
pub struct Problem {
    pub id: u64,
    pub name: String,
    #[serde(rename="type")]
    pub problem_type:String,
    pub misc:Misc,
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
