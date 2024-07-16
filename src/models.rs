use crate::config::Case;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{fmt, result};
use tokio::io::Join;
use tokio::sync::oneshot;
use tokio::time;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Job {
    pub source_code: String,
    pub language: String,
    pub user_id: i32,
    pub contest_id: i32,
    pub problem_id: i32,
}
impl Job {
    pub fn new() -> Job {
        Job {
            source_code: "".to_string(),
            language: "".to_string(),
            user_id: 0,
            contest_id: 0,
            problem_id: 0,
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobResponse {
    pub id: u64,
    pub created_time: String,
    pub updated_time: String,
    pub submission: Job,
    pub state: String,
    pub result: String,
    pub score: f64,
    pub cases: Vec<PointState>,
}
impl JobResponse {
    pub fn from_Jobstate(jbs: &JobState) -> JobResponse {
        JobResponse {
            id: jbs.id,
            created_time: jbs.created_time.clone(),
            updated_time: jbs.updated_time.clone(),
            submission: jbs.submission.clone(),
            state: jbs.state.clone(),
            result: jbs.result.clone(),
            score: {
                let mut sc: f64 = 0.0;
                for (c, p) in &jbs.cases {
                    if p.info == "Success" {
                        sc += c.score;
                    }
                }
                sc
            },
            cases:jbs.cases.iter().map(|(_,p)| p.clone()).collect()
        }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HTTPerror {
    code: u64,
    reason: String,
    message: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PointState {
    pub id: u64,
    pub result: String,
    pub time: u64,
    pub memory: u64,
    pub info: String,
}
impl PointState {
    pub fn new(id: u64, result: String, time: u64, memory: u64, info: String) -> PointState {
        PointState {
            id,
            result,
            time,
            memory,
            info,
        }
    }
}
pub struct JobState {
    pub id: u64,
    pub created_time: String,
    pub updated_time: String,
    pub submission: Job,
    pub state: String,
    pub result: String,
    pub score: f64,
    pub cases: Vec<(Case, PointState)>,
    pub sd: Option<oneshot::Sender<JobResponse>>,
}
impl JobState {
    pub fn new() -> JobState {
        JobState {
            id: 0,
            created_time: "".to_string(),
            updated_time: "".to_string(),
            submission: Job::new(),
            state: "".to_string(),
            result: "".to_string(),
            score: 0.0,
            cases: Vec::new(),
            sd: None,
        }
    }
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
    problem_ids: Vec<u64>,
    user_ids: Vec<u64>,
    submission_limit: u64,
}
lazy_static! {
    pub static ref JOB_LIST: Arc<Mutex<Vec<JobState>>> = Arc::new(Mutex::new(Vec::new()));
    pub static ref USER_LIST: Arc<Mutex<Vec<User>>> = Arc::new(Mutex::new(Vec::new()));
    pub static ref LANGUAGE_CONFIG: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("rust", "rustc");
        m.insert("c", "gcc");
        m.insert("c++", "g++");
        m
    };
}
