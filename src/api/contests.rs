use actix_web::{web, HttpResponse};
use crate::models::{User, USER_LIST};
use std::sync::{Arc, Mutex};

async fn get_ranklist() -> HttpResponse {
    let users = USER_LIST.lock().unwrap();
    let mut ranklist: Vec<_> = users.iter().collect();
   // ranklist.sort_by(|a, b| b.score.cmp(&a.score));
    HttpResponse::Ok().json(&ranklist)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/contests/0/ranklist")
        .route(web::get().to(get_ranklist))
    );
}
