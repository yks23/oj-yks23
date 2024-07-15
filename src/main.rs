use crate::api::{contests, jobs, users};
use crate::config::Config;
use crate::models::{User, USER_LIST};
use actix_web::{web, App, HttpServer,Responder};
use clap::parser::ValueSource;
mod api;
mod arg_process;
mod config;
mod models;
use env_logger;
use log;
// DO NOT REMOVE: used in automatic testing
#[post("/internal/exit")]
#[allow(unreachable_code)]
async fn exit() -> impl Responder {
    log::info!("Shutdown as requested");
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
    }
    let copy_config = config_data.clone();
    HttpServer::new(move || {
        App::new()
            .app_data(config_data.clone())
            .configure(jobs::init_routes)
            .configure(users::init_routes)
            .configure(contests::init_routes)
            .service(exit())
    })
    .bind(("127.0.0.1", copy_config.port))?
    .run()
    .await
}
