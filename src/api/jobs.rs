use crate::config::{Case, Config, LanguageConfig};
use crate::models::{
    HTTPerror, Job, JobResponse, JobState, PointState, JOB_LIST, LANGUAGE_CONFIG, USER_LIST,
};
use actix_web::{web, HttpResponse};
use chrono::Utc;
use std::fs::File;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::sync::oneshot;
use wait_timeout::ChildExt;
async fn post_job(config: web::Data<Config>, new_job: web::Json<Job>) -> HttpResponse {
    let mut job = new_job.into_inner();
    let mut jobs = JOB_LIST.lock().unwrap();
    let users = USER_LIST.lock().unwrap();
    let now = Utc::now();
    let created_time = now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let mut jobstats = JobState::new();
    jobstats.created_time = created_time.clone();
    jobstats.updated_time = created_time;
    jobstats.id = match jobs.last() {
        None => 0,
        Some(job) => job.id + 1,
    };
    jobstats.submission = job.clone();
    jobstats.score = 0.0;
    for p in &config.problems {
        if p.id as i32 == job.problem_id {
            jobstats.cases = p
                .cases
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    (
                        c.to_new(),
                        PointState::new(i as u64 + 1, "Waiting".to_string(), 0, 0, "".to_string()),
                    )
                })
                .collect();
            jobstats.cases.insert(
                0,
                (
                    Case::new(),
                    PointState::new(0, "Waiting".to_string(), 0, 0, "".to_string()),
                ),
            );
        }
    }
    {
        //handle language
        let mut flag1: bool = false;
        for i in LANGUAGE_CONFIG.keys() {
            if *i == job.language {
                flag1 = true;
            }
        }
        //handle user
        let mut flag2: bool = false;
        for i in users.iter() {
            if i.id.unwrap() as i32 == job.user_id {
                flag2 = true;
            }
        }
        if !flag1 || !flag2 {
            return HttpResponse::NotFound()
                .json(HTTPerror::new_none(3, "ERR_NOT_FOUND".to_string()));
        }
    }
    let (r, w) = oneshot::channel();
    jobstats.sd = Some(r);
    jobs.push(jobstats);
    let jbrs = w.await.unwrap();
    /*
    let compile_cmd:LanguageConfig;
    for &i in &config.languages{
        if i.name==job.language{
            compile_cmd=i;
        }
    }
    // 将代码写入临时文件
    let mut file = File::create(compile_cmd.file_name).expect("Failed to create temp file");
    file.write_all(job.source_code.as_bytes()).expect("Failed to write to temp file");
    // Compile
    let output = Command::new(compile_cmd.command[0])
        .args(&compile_cmd.command[1..])
        .output()
        .expect("Failed to compile");

    if !output.status.success() {
        return HttpResponse::Ok().json("Compilation Error");
    }
    let run_command="./".to_string()+exe_path;
    // Run
    let mut child = Command::new(run_command)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run");

    let timeout = Duration::from_millis(500);
    let status_code = match child.wait_timeout(timeout).unwrap() {
        Some(status) => status.code().unwrap_or(-1),
        None => {
            child.kill().unwrap();
            -1
        }
    };

    let job_result = match status_code {
        0 => "Accepted",
        -1 => "Time Limit Exceeded",
        _ => "Runtime Error",
    };

    let mut jobs = JOB_LIST.lock().unwrap();
    jobs.push(job);
    let mut out_put_message=String::new();
    if let Some(mut outputs)=child.stdout.take(){
        outputs.read_to_string(&mut out_put_message).expect("Failed to read stdout");
    }
    */
    HttpResponse::Ok().json(jbrs)
}
/*
async fn get_jobs() -> HttpResponse {
    let jobs = JOB_LIST.lock().unwrap();
    HttpResponse::Ok().json(&*jobs)
}
*/
pub async fn process_tasks(config: web::Data<Config>) {
    loop {
        let mut jobs = JOB_LIST.lock().unwrap();
        for job in jobs.iter_mut() {
            //queuing->running
            if job.state == "Queuing" {
                job.state = "Raiting".to_string();
                let now = Utc::now();
                let created_time = now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
                job.updated_time = created_time;
                job.result = "Running".to_string();
                job.cases[0].1.result = "Running".to_string();
                let now_job = job.submission.clone();
                let mut filename: String = String::new();
                let mut commands: Vec<String> = Vec::new();
                for i in &config.languages {
                    if i.name == now_job.language {
                        filename = i.file_name.clone();
                        commands = i.command.clone();
                    }
                }
                let sourcecode = now_job.source_code.clone();
                // 将代码写入临时文件
                let mut file = File::create(filename).expect("Failed to create temp file");
                file.write_all(sourcecode.as_bytes())
                    .expect("Failed to write to temp file");
                // Compile
                let output = Command::new(&commands[0]).args(&commands[1..]).output();
                if output.is_err() {
                    job.state = "Finished".to_string();
                    job.cases[0].1.result = "Compilation Error".to_string();
                    job.result = "Compilation Error".to_string();
                    let now = Utc::now();
                    let created_time = now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
                    job.updated_time = created_time;
                    if let Some(sd) = job.sd.take() {
                        let _=sd.send(JobResponse::from_Jobstate(&job));
                    }
                    continue;
                } else {
                    job.state = "Running".to_string();
                    job.cases[0].1.result = "Compilation Success".to_string();
                    job.result = "Running".to_string();
                    let now = Utc::now();
                    let created_time = now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
                    job.updated_time = created_time;
                    // run every point
                    for (case, pt) in job.cases.iter() {}
                }
            }
        }
    }
}
/*
async fn get_job_by_id(id: web::Path<u64>) -> HttpResponse {
    let jobs = JOB_LIST.lock().unwrap();
    let job = jobs.iter().find(|&job| job.id == *id);
    if let Some(job) = job {
        HttpResponse::Ok().json(job)
    } else {
        HttpResponse::NotFound().finish()
    }
}
 */
/*
async fn put_job_by_id(id: web::Path<u64>) -> HttpResponse {
    let mut jobs = JOB_LIST.lock().unwrap();
    let job = jobs.iter_mut().find(|job| job.id == *id);
    if let Some(job) = job {
        // Re-evaluate the job
        job.status = "Reevaluated".to_string();
        HttpResponse::Ok().json(job)
    } else {
        HttpResponse::NotFound().finish()
    }
}
*/
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/jobs").route(web::post().to(post_job)), //.route(web::get().to(get_jobs))
    );
    //cfg.service(web::resource("/jobs/{id}")
    //  .route(web::get().to(get_job_by_id))
    //.route(web::put().to(put_job_by_id))
    //);
}
