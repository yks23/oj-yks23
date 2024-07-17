use crate::api::{contests, jobs, users};
use crate::config::Config;
use crate::models::{User, USER_LIST,CONTEST_LIST};
use actix_web::{web, App, HttpServer,Responder,post};
use actix_cors::Cors;
use clap::parser::ValueSource;
mod api;
mod arg_process;
mod config;
mod models;
use clap::Command;
use env_logger;
use log;
use models::Contest;
// DO NOT REMOVE: used in automatic testing
#[post("/internal/exit")]
#[allow(unreachable_code)]
async fn exit() -> impl Responder {
    log::info!("Shutdown as requested");
    let _=std::process::Command::new("rm").arg("main*");
    std::process::exit(0);
    format!("Exited")
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
    {
        let mut users = USER_LIST.lock().unwrap();
        users.push(root_user);
        let mut contests=CONTEST_LIST.lock().unwrap();
        let mut problem_vec:Vec<u64>=Vec::new();
        for i in config_data.clone().problems.iter(){
            problem_vec.push(i.id);
        }
        contests.push(Contest{user_ids:vec![0],problem_ids:problem_vec,id:Some(0),name:"Global Contest".to_string(),from:"".to_string(),to:"".to_string(),submission_limit:10000});
    }
    let copy_config = config_data.clone();
    tokio::spawn(async move {
        jobs::process_tasks(copy_config).await;
    });
    let copy_config = config_data.clone();
    HttpServer::new(move || {
        App::new()
            .app_data(config_data.clone())
            .wrap(Cors::default().allow_any_origin().allow_any_method().allow_any_header())
            .configure(jobs::init_routes)
            .configure(users::init_routes)
            .configure(contests::init_routes)
            .service(exit)
    })
    .bind((copy_config.server.bind_address.clone(), copy_config.server.bind_port))?
    .run()
    .await
}
