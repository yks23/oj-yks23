use crate::config::{Case, Config};
use crate::models::{
    HTTPerror, Job, JobFilter, JobResponse, JobState, PointState, CONTEST_LIST, JOB_LIST, USER_LIST,
};
use crate::save_jobs;
use actix_web::{web, HttpResponse};
use chrono::DateTime;
use chrono::Utc;
use futures::future::join_all;
use glob::glob;
use std::process::Stdio;
use std::time::Instant;
use tokio::io::ErrorKind;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

// Loose comparison function: compares two strings ignoring trailing whitespaces
fn loose_compare(output: &str, expected: &str) -> bool {
    let output_lines: Vec<&str> = output.lines().map(|line| line.trim_end()).collect();
    let expected_lines: Vec<&str> = expected.lines().map(|line| line.trim_end()).collect();
    output_lines == expected_lines
}

// Strict comparison function: strictly compares two strings
fn strict_compare(output: &str, expected: &str) -> bool {
    output == expected
}

// Handles posting a new job
async fn post_job(config: web::Data<Config>, new_job: web::Json<Job>) -> HttpResponse {
    log::info!("Enter post_job.own joblist");
    let job = new_job.into_inner();

    let mut jobs = JOB_LIST.lock().unwrap();
    let users = USER_LIST.lock().unwrap();
    let now = Utc::now();
    let created_time = now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let mut jobstats = JobState::new();
    jobstats.created_time = created_time.clone();
    jobstats.updated_time = created_time.clone();
    jobstats.problem_id = job.problem_id as usize;
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

    // Handle language validation
    let mut flag1: bool = false;
    for i in &config.languages {
        if *i.name == job.language {
            flag1 = true;
        }
    }
    // Handle user validation
    let mut flag2: bool = false;
    for i in users.iter() {
        if i.id.unwrap() as i32 == job.user_id {
            flag2 = true;
        }
    }
    // Handle contests validation
    let mut cnt_sub: u64 = 0;
    for i in jobs.iter() {
        if i.submission.user_id == job.user_id
            && i.submission.contest_id == job.contest_id
            && i.submission.problem_id == job.problem_id
        {
            cnt_sub += 1;
        }
    }

    let mut flag3: bool = false;
    let mut flag4: bool = false;
    let mut flag5: bool = true;
    {
        let contests = CONTEST_LIST.lock().unwrap();
        for i in contests.iter() {
            if i.id.unwrap() as i32 == job.contest_id {
                if cnt_sub >= i.submission_limit {
                    flag5 = false;
                }
                flag4 = true;
                if i.problem_ids.contains(&(job.problem_id as u64)) {
                    if (i.from <= created_time && i.to >= created_time) || job.contest_id == 0 {
                        flag3 = true;
                    } else {
                        break;
                    }
                }
                if !i.user_ids.contains(&(job.user_id as u64)) {
                    flag3 = false;
                }
            }
        }
    }
    if !flag1 || !flag2 || !flag4 {
        return HttpResponse::NotFound().json(HTTPerror::new_none(3, "ERR_NOT_FOUND".to_string()));
    }
    if !flag3 {
        return HttpResponse::BadRequest()
            .json(HTTPerror::new_none(1, "ERR_INVALID_ARGUMENT".to_string()));
    }
    if !flag5 {
        return HttpResponse::BadRequest()
            .json(HTTPerror::new_none(4, "ERR_RATE_LIMIT".to_string()));
    }
    jobstats.state = "Queueing".to_string();
    jobstats.result = "Waiting".to_string();
    jobs.push(jobstats.clone());
    drop(jobs);
    {
        save_jobs().unwrap();
    }
    HttpResponse::Ok().json(JobResponse::from_jobstate(&jobstats))
}

// Remove files matching a specific pattern
async fn remove_files_with_pattern(pattern: &str) -> Result<(), Box<dyn std::error::Error>> {
    for entry in glob(pattern)? {
        match entry {
            Ok(path) => match tokio::fs::remove_file(&path).await {
                Ok(_) => println!("Successfully deleted {:?}", path),
                Err(e) => match e.kind() {
                    ErrorKind::NotFound => eprintln!("File not found: {:?}", path),
                    _ => eprintln!("Error deleting {:?}: {:?}", path, e),
                },
            },
            Err(e) => eprintln!("Error processing entry: {:?}", e),
        }
    }
    Ok(())
}

// Asynchronously read file contents
async fn read_file_async(file_path: &str) -> std::io::Result<String> {
    let mut file = tokio::fs::File::open(file_path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    Ok(contents)
}

// Process a specific job
pub async fn process_task(config: web::Data<Config>, job_id: usize) {
    log::info!("Processing job {}", job_id);

    let job_opt = {
        let mut jobs = JOB_LIST.lock().unwrap();
        jobs.iter_mut()
            .find(|job| job.id as usize == job_id)
            .map(|job| job.clone())
    };

    if let Some(mut job) = job_opt {
        job.state = "Waiting".to_string();
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
        remove_files_with_pattern("main*").await.unwrap();
        // Write code to temporary file
        let mut file = tokio::fs::File::create(filename.clone()).await.unwrap();
        file.write_all(sourcecode.as_bytes()).await.unwrap();

        // Compile
        let compile_commands: Vec<String> = commands[1..]
            .iter()
            .map(|c| match c {
                c if c == "%INPUT%" => filename.clone(),
                c if c == "%OUTPUT%" => "main".to_string() + job.id.to_string().as_str(),
                _ => c.to_string(),
            })
            .collect();
        let exe_file = "main".to_string() + job.id.to_string().as_str();
        let child = Command::new(&commands[0])
            .args(compile_commands)
            .spawn()
            .unwrap();
        let output = child.wait_with_output().await.unwrap();
        if !output.status.success() {
            job.state = "Finished".to_string();
            job.cases[0].1.result = "Compilation Error".to_string();
            job.result = "Compilation Error".to_string();
            let now = Utc::now();
            let created_time = now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
            job.updated_time = created_time;
        } else {
            job.state = "Running".to_string();
            job.cases[0].1.result = "Compilation Success".to_string();
            job.result = "Running".to_string();
            let now = Utc::now();
            let created_time = now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
            job.updated_time = created_time;
            {
                let mut jobs = JOB_LIST.lock().unwrap();

                if let Some(existing_job) = jobs.iter_mut().find(|j| j.id as usize == job_id) {
                    *existing_job = job.clone();
                }
            }
            // Run every test case
            for (idx, (case, pt)) in job.cases[1..].iter_mut().enumerate() {
                let input_file = case.input_file.clone();
                let expected_output_file = case.answer_file.clone();

                // Asynchronously read input file contents
                let input = read_file_async(&input_file)
                    .await
                    .expect("Unable to read input file");

                // Define time limit
                let time_limit = Duration::from_micros(case.time_limit);

                // Start timer
                let start = Instant::now();

                // Asynchronously execute the compiled file and pass input to stdin
                let mut child = Command::new("./".to_string() + exe_file.as_str())
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("Failed to start process");

                // Write input content to child process's stdin
                if let Some(mut stdin) = child.stdin.take() {
                    stdin
                        .write_all(input.as_bytes())
                        .await
                        .expect("Failed to write to stdin");
                }

                // Wait for child process to complete or timeout
                let result = timeout(time_limit, child.wait()).await;

                // Check if child process exited or timed out
                match result {
                    Ok(Ok(status)) => {
                        // Child process executed successfully
                        if status.success() {
                            // Stop timer
                            let duration = start.elapsed();
                            pt.time = duration.as_micros() as u64;

                            // Get child process output
                            let output = child
                                .wait_with_output()
                                .await
                                .expect("Failed to read stdout");

                            // Asynchronously read expected output file contents
                            let expected_output = read_file_async(&expected_output_file)
                                .await
                                .expect("Unable to read expected output file");

                            // Compare actual output with expected output
                            let output_stdout = String::from_utf8_lossy(&output.stdout);
                            log::info!("{} {}", expected_output, output_stdout);
                            let mut prom_type: String = "".to_string();
                            for pp in &config.problems {
                                if pp.id == job.problem_id as u64 {
                                    prom_type = pp.problem_type.clone();
                                }
                            }
                            let flag: bool = match prom_type {
                                s if s == "strict" => {
                                    strict_compare(&output_stdout, &expected_output)
                                }
                                _ => loose_compare(&output_stdout, &expected_output),
                            };
                            if flag {
                                pt.result = "Accepted".to_string();
                            } else {
                                pt.result = "Wrong Answer".to_string();
                                if job.result == "Running" {
                                    job.result = "Wrong Answer".to_string();
                                }
                            }
                        } else {
                            pt.result = "Runtime Error".to_string();
                            if job.result == "Running" {
                                job.result = "Runtime Error".to_string();
                            }
                        }
                    }
                    Ok(Err(_)) => {
                        // Child process failed to execute
                        pt.result = "Runtime Error".to_string();
                        if job.result == "Running" {
                            job.result = "Runtime Error".to_string();
                        }
                    }
                    Err(_) => {
                        // Child process timed out and was forcefully killed
                        pt.result = "Time Limit Exceeded".to_string();
                        child.kill().await.expect("Failed to kill process");
                        if job.result == "Running" {
                            job.result = "Time Limit Exceeded".to_string();
                        }
                    }
                }
                let now = Utc::now();
                let created_time = now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
                job.updated_time = created_time;
                {
                    {
                        let mut jobs = JOB_LIST.lock().unwrap();

                        if let Some(existing_job) =
                            jobs.iter_mut().find(|j| j.id as usize == job_id)
                        {
                            existing_job.updated_time = job.updated_time.clone();
                            existing_job.result = job.result.clone();
                            existing_job.state = job.state.clone();
                            existing_job.cases[idx].1.result = pt.result.clone();
                            existing_job.cases[idx].1.time = pt.time.clone();
                            existing_job.update_score();
                        }
                    }
                    save_jobs().unwrap();
                }
            }
            job.state = "Finished".to_string();
            if job.result == "Running" {
                job.result = "Accepted".to_string();
            }
        }
        job.update_score();
        // Update job in JOB_LIST
        {
            {
                let mut jobs = JOB_LIST.lock().unwrap();

                if let Some(existing_job) = jobs.iter_mut().find(|j| j.id as usize == job_id) {
                    *existing_job = job;
                }
            }
            save_jobs().unwrap();
        }
    }
}

// Continuously process tasks in the queue
pub async fn process_tasks(config: web::Data<Config>) {
    loop {
        let job_ids: Vec<usize> = {
            let jobs = JOB_LIST.lock().unwrap();
            jobs.iter()
                .filter(|job| job.state == "Queueing")
                .map(|job| job.id as usize)
                .collect()
        };
        let mut tasks = vec![];

        for job_id in job_ids {
            let config_clone = config.clone();
            tasks.push(tokio::task::spawn(async move {
                process_task(config_clone, job_id).await;
            }));
        }
        let _ = join_all(tasks).await;
    }
}

// Get job by ID
async fn get_job_by_id(id: web::Path<u64>) -> HttpResponse {
    let jobs = JOB_LIST.lock().unwrap();
    let job = jobs.iter().find(|&job| job.id == *id);
    if let Some(jobn) = job {
        HttpResponse::Ok().json(JobResponse::from_jobstate(jobn))
    } else {
        HttpResponse::NotFound().json(HTTPerror::new(
            3,
            "ERR_NOT_FOUND".to_string(),
            format!("Job {} not found.", *id),
        ))
    }
}

// Update job by ID
async fn put_job_by_id(id: web::Path<u64>) -> HttpResponse {
    let mut flag: bool = false;
    log::info!("response to put_jobs");
    {
        let mut jobs = JOB_LIST.lock().unwrap();

        for job in jobs.iter_mut() {
            if job.id == *id {
                if job.state != "Finished" {
                    return HttpResponse::BadRequest().json(HTTPerror::new(
                        2,
                        "ERR_INVALID_STATE".to_string(),
                        format!("Job {} not finished", job.id),
                    ));
                }
                job.result = "Waiting".to_string();
                job.score = 0.0;
                job.state = "Queueing".to_string();
                let now = Utc::now();
                let created_time = now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

                job.updated_time = created_time;
                for (_, p) in job.cases.iter_mut() {
                    p.result = "Waiting".to_string();
                }
                flag = true;
                let rep = JobResponse::from_jobstate(&job);
                return HttpResponse::Ok().json(rep);
            }
        }
    }
    if !flag {
        return HttpResponse::NotFound().json(HTTPerror::new(
            3,
            "ERR_NOT_FOUND".to_string(),
            format!("Job {} not found.", *id),
        ));
    }
    {
        save_jobs().unwrap();
    }

    HttpResponse::Ok().finish()
}

// Delete job by ID
async fn delete_job_by_id(id: web::Path<u64>) -> HttpResponse {
    log::info!("response to delete_job");
    let mut flag: bool = false;
    {
        let mut jobs = JOB_LIST.lock().unwrap();
        for (idx, job) in jobs.iter_mut().enumerate() {
            if job.id == *id {
                jobs.remove(idx);
                flag = true;
                break;
            }
        }
    }
    {
        save_jobs().unwrap();
    }
    if !flag {
        return HttpResponse::NotFound().json(HTTPerror::new(
            3,
            "ERR_NOT_FOUND".to_string(),
            format!("Job {} not found.", *id),
        ));
    } else {
        HttpResponse::Ok().finish()
    }
}

// Filter jobs based on the filter criteria
fn fil_true(jf: &JobFilter, jb: &JobState) -> bool {
    if jf.state.is_some() && jb.state != jf.state.clone().unwrap() {
        return false;
    }
    if jf.result.is_some() && jb.result != jf.result.clone().unwrap() {
        return false;
    }
    if jf.user_id.is_some() && jb.submission.user_id as u64 != jf.user_id.clone().unwrap() {
        return false;
    }
    if jf.problem_id.is_some() && jb.submission.problem_id as u64 != jf.problem_id.clone().unwrap()
    {
        return false;
    }
    if jf.contest_id.is_some() && jb.submission.contest_id as u64 != jf.contest_id.clone().unwrap()
    {
        return false;
    }
    if jf.language.is_some() && jb.submission.language != jf.language.clone().unwrap() {
        return false;
    }
    if jf.from.is_some() {
        let from_t = DateTime::parse_from_rfc3339(&jf.from.clone().unwrap())
            .expect("Failed to parse datetime1")
            .with_timezone(&Utc);
        let nowt = DateTime::parse_from_rfc3339(&jb.created_time.clone())
            .expect("Failed to parse datetime2")
            .with_timezone(&Utc);
        // Compare time
        if from_t > nowt {
            return false;
        }
    }
    if jf.to.is_some() {
        let to_t = DateTime::parse_from_rfc3339(&jf.to.clone().unwrap())
            .expect("Failed to parse datetime1")
            .with_timezone(&Utc);
        let nowt = DateTime::parse_from_rfc3339(&jb.created_time.clone())
            .expect("Failed to parse datetime2")
            .with_timezone(&Utc);
        // Compare time
        if to_t < nowt {
            return false;
        }
    }
    true
}

// Get jobs based on the filter criteria
async fn get_jobs(filt: web::Query<JobFilter>) -> HttpResponse {
    log::info!("response to get_jobs");
    let filt = filt.into_inner();
    let mut vc: Vec<JobResponse> = Vec::new();
    let jobs = JOB_LIST.lock().unwrap();
    for jb in jobs.iter() {
        if fil_true(&filt, jb) {
            vc.push(JobResponse::from_jobstate(jb));
        }
    }
    HttpResponse::Ok().json(vc)
}

// Initialize routes
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/jobs")
            .route(web::post().to(post_job))
            .route(web::get().to(get_jobs)),
    );
    cfg.service(
        web::resource("/jobs/{id}")
            .route(web::get().to(get_job_by_id))
            .route(web::put().to(put_job_by_id))
            .route(web::delete().to(delete_job_by_id)),
    );
}
