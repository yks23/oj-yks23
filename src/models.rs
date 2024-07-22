use crate::config::Case;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
};
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
    pub fn from_jobstate(jbs: &JobState) -> JobResponse {
        JobResponse {
            id: jbs.id,
            created_time: jbs.created_time.clone(),
            updated_time: jbs.updated_time.clone(),
            submission: jbs.submission.clone(),
            state: jbs.state.clone(),
            result: jbs.result.clone(),
            score: jbs.score,
            cases: jbs.cases.iter().map(|(_, p)| p.clone()).collect(),
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
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobState {
    pub id: u64,
    pub problem_id: usize,
    pub created_time: String,
    pub updated_time: String,
    pub submission: Job,
    pub state: String,
    pub result: String,
    pub score: f64,
    pub cases: Vec<(Case, PointState)>,
}
impl JobState {
    pub fn new() -> JobState {
        JobState {
            id: 0,
            problem_id: 0,
            created_time: "".to_string(),
            updated_time: "".to_string(),
            submission: Job::new(),
            state: "".to_string(),
            result: "".to_string(),
            score: 0.0,
            cases: Vec::new(),
        }
    }
    pub fn update_score(&mut self, packing: &Option<Vec<Vec<usize>>>) {
        let mut score: f64 = 0.0;
        if packing.is_none() {
            log::info!("Actually nothing");
            for (c, p) in &self.cases {
                if p.result == "Accepted" {
                    score += c.score;
                }
            }
        } else {
            log::info!("{:?}", packing);
            let vc = packing.clone().unwrap();
            for packs in vc.iter() {
                let mut bl = true;
                for i in packs {
                    if self.cases[*i].1.result != "Accepted" {
                        bl = false;
                    }
                }
                if bl {
                    for i in packs {
                        score += self.cases[*i].0.score;
                    }
                }
            }
        }
        self.score = score;
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Contest {
    pub id: Option<u64>,
    pub name: String,
    pub from: String,
    pub to: String,
    pub problem_ids: Vec<u64>,
    pub user_ids: Vec<u64>,
    pub submission_limit: u64,
}
lazy_static! {
    pub static ref JOB_LIST: Arc<Mutex<Vec<JobState>>> = Arc::new(Mutex::new(Vec::new()));
    pub static ref USER_LIST: Arc<Mutex<Vec<User>>> = Arc::new(Mutex::new(Vec::new()));
    pub static ref CONTEST_LIST: Arc<Mutex<Vec<Contest>>> = Arc::new(Mutex::new(Vec::new()));
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobFilter {
    pub user_id: Option<u64>,
    pub user_name: Option<String>,
    pub contest_id: Option<u64>,
    pub problem_id: Option<u64>,
    pub language: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub state: Option<String>,
    pub result: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContestConfig {
    pub scoring_rule: Option<String>,
    pub tie_breaker: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserRank {
    pub user: User,
    pub rank: u64,
    pub scores: Vec<f64>,
}

impl UserRank {
    pub fn new() -> UserRank {
        UserRank {
            user: User {
                name: "".to_string(),
                id: None,
            },
            rank: 0,
            scores: Vec::new(),
        }
    }
}
