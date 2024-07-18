use crate::api::{contests, jobs, users};
use crate::config::Config;
use crate::models::{Contest, JobState, User, CONTEST_LIST, JOB_LIST, USER_LIST};
use actix_cors::Cors;
use actix_web::test::ok_service;
use actix_web::{post, web, App, HttpServer, Responder};
use clap::parser::ValueSource;
use rusqlite::{params, Connection, Result};
use std::error::Error;
use std::path::Path;
mod api;
mod arg_process;
mod config;
mod models;
use env_logger;
use log;
fn save_users() -> Result<(), Box<dyn Error>> {
    let conn = Connection::open("database.db")?;
    conn.execute("DROP TABLE IF EXISTS users", params![])?;
    conn.execute(
        "CREATE TABLE users (
            source TEXT NOT NULL
        )",
        params![],
    )?;
    let users = USER_LIST.lock()?;
    for us in users.iter() {
        conn.execute(
            "INSERT INTO users (source) VALUES (?1)",
            params![serde_json::to_string(us)?],
        )?;
    }
    Ok(())
}
fn save_jobs() -> Result<(), Box<dyn Error>> {
    let conn = Connection::open("database.db")?;
    conn.execute("DROP TABLE IF EXISTS jobs", params![])?;
    conn.execute(
        "CREATE TABLE jobs (
            source TEXT NOT NULL
        )",
        params![],
    )?;
    let users = JOB_LIST.lock()?;
    for us in users.iter() {
        conn.execute(
            "INSERT INTO jobs (source) VALUES (?1)",
            params![serde_json::to_string(us)?],
        )?;
    }
    log::info!("save the jobs: {:?}", *users);
    Ok(())
}
fn save_contests() -> Result<(), Box<dyn Error>> {
    let conn = Connection::open("database.db")?;
    conn.execute("DROP TABLE IF EXISTS contests", params![])?;
    conn.execute(
        "CREATE TABLE contests (
            source TEXT NOT NULL
        )",
        params![],
    )?;
    let users = CONTEST_LIST.lock()?;
    for us in users.iter() {
        conn.execute(
            "INSERT INTO contests (source) VALUES (?1)",
            params![serde_json::to_string(us)?],
        )?;
    }
    Ok(())
}
// DO NOT REMOVE: used in automatic testing
#[post("/internal/exit")]
#[allow(unreachable_code)]
async fn exit() -> impl Responder {
    log::info!("Shutdown as requested");
    let _ = std::process::Command::new("rm").arg("main*");
    save_users().expect("User save error");
    save_jobs().expect("Job save error");
    save_contests().expect("Contest save error");

    std::process::exit(0);
    format!("Exited")
}
fn query_all_users(conn: &Connection) -> Result<Vec<User>> {
    let mut stmt = conn.prepare("SELECT source FROM users")?;
    let user_iter = stmt.query_map([], |row| {
        let source: String = row.get(0)?;
        Ok(serde_json::from_str(&source).unwrap())
    })?;

    let mut users = Vec::new();
    for user in user_iter {
        users.push(user?);
    }
    Ok(users)
}
fn query_all_contests(conn: &Connection) -> Result<Vec<Contest>> {
    let mut stmt = conn.prepare("SELECT source FROM contests")?;
    let user_iter = stmt.query_map([], |row| {
        let source: String = row.get(0)?;
        Ok(serde_json::from_str(&source).unwrap())
    })?;

    let mut users = Vec::new();
    for user in user_iter {
        users.push(user?);
    }
    Ok(users)
}
fn query_all_jobs(conn: &Connection) -> Result<Vec<JobState>> {
    let mut stmt = conn.prepare("SELECT source FROM jobs")?;
    let user_iter = stmt.query_map([], |row| {
        let source: String = row.get(0)?;
        Ok(serde_json::from_str(&source).unwrap())
    })?;

    let mut users = Vec::new();
    for user in user_iter {
        users.push(user?);
    }
    Ok(users)
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let matched = arg_process::read_command();
    if matched.value_source("config").unwrap() == ValueSource::DefaultValue {
        eprintln!("Usage: -c [path_to_config.json]");
        std::process::exit(1);
    }
    let config = Config::from_file(matched.get_one::<String>("config").unwrap().as_str());
    let config_data = web::Data::new(config);
    // Initialize default root user
    let root_user = User {
        id: Some(0),
        name: "root".to_string(),
    };

    if !matched.get_flag("flush-data") && Path::new("database.db").exists() {
        let conn = Connection::open("database.db").unwrap();
        {
            let mut users = USER_LIST.lock().unwrap();
            *users = query_all_users(&conn).expect("Not a User list");
            let mut jobs = JOB_LIST.lock().unwrap();
            *jobs = query_all_jobs(&conn).expect("Not a Job list");
            let mut contests = CONTEST_LIST.lock().unwrap();
            *contests = query_all_contests(&conn).expect("Not a Contest list");
        }
    } else {
        let mut users = USER_LIST.lock().unwrap();
        users.push(root_user);
        let mut contests = CONTEST_LIST.lock().unwrap();
        let mut problem_vec: Vec<u64> = Vec::new();
        for i in config_data.clone().problems.iter() {
            problem_vec.push(i.id);
        }
        contests.push(Contest {
            user_ids: vec![0],
            problem_ids: problem_vec,
            id: Some(0),
            name: "Global Contest".to_string(),
            from: "".to_string(),
            to: "".to_string(),
            submission_limit: 10000,
        });
    }
    let copy_config = config_data.clone();
    tokio::spawn(async move {
        jobs::process_tasks(copy_config).await;
    });
    let copy_config = config_data.clone();
    HttpServer::new(move || {
        App::new()
            .app_data(config_data.clone())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .configure(jobs::init_routes)
            .configure(users::init_routes)
            .configure(contests::init_routes)
            .service(exit)
    })
    .bind((
        copy_config.server.bind_address.clone(),
        copy_config.server.bind_port,
    ))?
    .run()
    .await
}
