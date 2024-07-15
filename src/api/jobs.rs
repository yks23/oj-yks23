use actix_web::{web, HttpResponse};
use std::process::{Command, Stdio};
use wait_timeout::ChildExt;
use std::time::Duration;
use crate::models::{HTTPerror, Job, JOB_LIST, USER_LIST};
use crate::config::Config;
use std::fs::File;
use std::io::{Read, Write};
async fn post_job(config: web::Data<Config>, new_job: web::Json<Job>) -> HttpResponse {
    let mut job = new_job.into_inner();
    let compile_cmd = &config.languages[&job.language].compile;
    let run_cmd = &config.languages[&job.language].run;
    let temp_file_path = "temp_code.rs";//change then
    let exe_path="temp_code";
    let users=USER_LIST.lock().unwrap();
    {//handle language
    let mut flag1:bool=false;
    for i in config.languages.keys(){
        if *i==job.language{
            flag1=true;
        }
    }
    //handle user
    let mut flag2:bool=false;
    for i in users.iter(){
        if i.id.unwrap() as i32==job.user_id{
            flag2=true;
        }
    }
    if !flag1||!flag2{
        return HttpResponse::NotFound().json(HTTPerror::new_none(3, "ERR_NOT_FOUND".to_string()));
    }
    }
    // 将代码写入临时文件
    let mut file = File::create(temp_file_path).expect("Failed to create temp file");
    file.write_all(job.source_code.as_bytes()).expect("Failed to write to temp file");
    // Compile
    let output = Command::new(compile_cmd)
        .arg(&temp_file_path)
        .arg("-o")
        .arg(exe_path)
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

    let timeout = Duration::from_millis(config.timeout);
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
    HttpResponse::Ok().body(out_put_message)
}

async fn get_jobs() -> HttpResponse {
    let jobs = JOB_LIST.lock().unwrap();
    HttpResponse::Ok().json(&*jobs)
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
    cfg.service(web::resource("/jobs")
        .route(web::post().to(post_job))
        //.route(web::get().to(get_jobs))
    );
    //cfg.service(web::resource("/jobs/{id}")
      //  .route(web::get().to(get_job_by_id))
        //.route(web::put().to(put_job_by_id))
    //);
}
