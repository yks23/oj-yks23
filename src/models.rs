use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::{Arc, Mutex};
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Job {
    pub source_code: String,
    pub language: String,
    pub user_id: i32,
    pub contest_id: i32,
    pub problem_id: i32,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HTTPerror {
    code: u64,
    reason: String,
    message: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PointState {
     id:u64,
      result:String,
      time: u64,
      memory: u64,
      info: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobState {
    id: u64,
    created_time: String,
    updated_time: String,
    submission: Job,
    state: String,
    result: String,
    score: f64,
    cases: Vec<PointState>,
}
impl HTTPerror {
    pub fn new(code: u64, reason: String, message: String) -> HTTPerror {
        HTTPerror {
            code,
            reason,
            message: Some(message),
        }
    }
    pub fn new_none(code: u64, reason: String) -> HTTPerror {
        HTTPerror {
            code,
            reason,
            message: None,
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: Option<u64>,
    pub name: String,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MyStruct {{ id: {}, name: {} }}",
            self.id.unwrap(),
            self.name
        )?;
        Ok(())
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Contest {
     
  id: Option<u64>,
  name: String,
  from: String,
  to: String,
  problem_ids:Vec<u64>,
  user_ids: Vec<u64>,
  submission_limit: u64,

}
lazy_static! {
    pub static ref JOB_LIST: Arc<Mutex<Vec<Job>>> = Arc::new(Mutex::new(Vec::new()));
    pub static ref USER_LIST: Arc<Mutex<Vec<User>>> = Arc::new(Mutex::new(Vec::new()));
}
