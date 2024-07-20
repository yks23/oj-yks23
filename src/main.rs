use crate::api::{contests, jobs, users};
use crate::config::Config;
use crate::models::{Contest, JobState, User, CONTEST_LIST, JOB_LIST, USER_LIST};
use actix_cors::Cors;
use actix_web::{post, web, App, HttpServer, Responder};
use clap::parser::ValueSource;
use rusqlite::{params, Connection, Result};
use std::error::Error;
use std::path::Path;
use env_logger;
use log;

// Importing modules
mod api;
mod arg_process;
mod config;
mod models;

// General function to save data to the database
fn save_to_db<T: serde::Serialize>(
    table_name: &str,
    data: &std::sync::MutexGuard<Vec<T>>,
) -> Result<(), Box<dyn Error>> {
    let conn = Connection::open("database.db")?;
    conn.execute(&format!("DROP TABLE IF EXISTS {}", table_name), params![])?;
    conn.execute(
        &format!(
            "CREATE TABLE {} (
                source TEXT NOT NULL
            )",
            table_name
        ),
        params![],
    )?;
    for item in data.iter() {
        conn.execute(
            &format!("INSERT INTO {} (source) VALUES (?1)", table_name),
            params![serde_json::to_string(item)?],
        )?;
    }
    Ok(())
}

// Save user data
fn save_users() -> Result<(), Box<dyn Error>> {
    let users = USER_LIST.lock()?;
    save_to_db("users", &users)
}

// Save job data
fn save_jobs() -> Result<(), Box<dyn Error>> {
    let jobs = JOB_LIST.lock()?;
    save_to_db("jobs", &jobs)?;
    log::info!("save the jobs: {:?}", *jobs);
    Ok(())
}

// Save contest data
fn save_contests() -> Result<(), Box<dyn Error>> {
    let contests = CONTEST_LIST.lock()?;
    save_to_db("contests", &contests)
}

// Exit and save all data
#[post("/internal/exit")]
#[allow(unreachable_code)]
async fn exit() -> impl Responder {
    log::info!("Shutdown as requested");
    save_users().expect("User save error");
    save_jobs().expect("Job save error");
    save_contests().expect("Contest save error");
    std::process::exit(0);
    format!("Exited")
}

// General function to query data from the database
fn query_all_from_db<T: serde::de::DeserializeOwned>(
    conn: &Connection,
    table_name: &str,
) -> Result<Vec<T>> {
    let mut stmt = conn.prepare(&format!("SELECT source FROM {}", table_name))?;
    let item_iter = stmt.query_map([], |row| {
        let source: String = row.get(0)?;
        Ok(serde_json::from_str(&source).unwrap())
    })?;

    let mut items = Vec::new();
    for item in item_iter {
        items.push(item?);
    }
    Ok(items)
}

// Query all users
fn query_all_users(conn: &Connection) -> Result<Vec<User>> {
    query_all_from_db(conn, "users")
}

// Query all contests
fn query_all_contests(conn: &Connection) -> Result<Vec<Contest>> {
    query_all_from_db(conn, "contests")
}

// Query all jobs
fn query_all_jobs(conn: &Connection) -> Result<Vec<JobState>> {
    query_all_from_db(conn, "jobs")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Parse command line arguments
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

    // Load or initialize data
    if !matched.get_flag("flush-data") && Path::new("database.db").exists() {
        log::info!("load the save file");
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
        let problem_vec: Vec<u64> = config_data.clone().problems.iter().map(|p| p.id).collect();
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
