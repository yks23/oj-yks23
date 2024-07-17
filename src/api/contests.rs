use crate::config::Config;
use crate::models::{Contest, HTTPerror, User, CONTEST_LIST, USER_LIST};
use actix_web::{web, HttpResponse};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
async fn post_contest(config: web::Data<Config>, contest: web::Json<Contest>) -> HttpResponse {
    let mut contests = CONTEST_LIST.lock().unwrap();
    let mut contest = contest.into_inner();
    if contest.submission_limit == 0 {
        contest.submission_limit = 10086;
    }

    {
        let mut tb: HashSet<u64> = HashSet::new();
        for i in &contest.user_ids {
            if tb.contains(&i) {
                return HttpResponse::BadRequest().json(HTTPerror::new(
                    1,
                    "ERR_INVALID_ARGUMENT".to_string(),
                    format!("Invalid argument user_ids"),
                ));
            } else {
                tb.insert(*i);
            }
            let mut flag: bool = false;
            {
                for j in USER_LIST.lock().unwrap().iter() {
                    if j.id.unwrap() == *i {
                        flag = true;
                    }
                }
            }
            if !flag {
                return HttpResponse::NotFound().json(HTTPerror::new(
                    3,
                    "ERR_NOT_FOUND".to_string(),
                    format!("Contest {} not found.", contest.id.unwrap()),
                ));
            }
        }
        tb.clear();
        for i in &contest.problem_ids {
            if tb.contains(i) {
                return HttpResponse::BadRequest().json(HTTPerror::new(
                    1,
                    "ERR_INVALID_ARGUMENT".to_string(),
                    format!("Invalid argument problem_ids"),
                ));
            } else {
                tb.insert(*i);
            }
            let mut flag: bool = false;
            for j in &config.problems {
                if j.id == *i {
                    flag = true;
                }
            }
            if !flag {
                return HttpResponse::NotFound().json(HTTPerror::new(
                    3,
                    "ERR_NOT_FOUND".to_string(),
                    format!("Contest {} not found.", contest.id.unwrap()),
                ));
            }
        }
        //check duplication
    }
    // 自动生成新的唯一 id
    if contest.id.is_none() {
        for his_con in contests.iter() {
            if his_con.name == contest.name {
                return HttpResponse::BadRequest().json(HTTPerror::new(
                    1,
                    "ERR_INVALID_ARGUMENT".to_string(),
                    format!("Contest name '{}' already exists.", contest.name),
                ));
            }
        }
        let new_id = if let Some(last_user) = contests.last() {
            last_user.id.unwrap() + 1
        } else {
            0
        };
        contest.id = Some(new_id);
        contests.push(contest.clone());
        log::info!(
            "{}",
            format!("Successfully create new Contest {} !!", contest.name)
        );
        return HttpResponse::Ok().json(contests.last().unwrap());
    } else {
        if contest.id.unwrap() == 0 {
            return HttpResponse::BadRequest().json(HTTPerror::new(
                1,
                "ERR_INVALID_ARGUMENT".to_string(),
                "Invalid contest id".to_string(),
            ));
        }
        let mut flag: bool = false;
        for his_con in contests.iter_mut() {
            if &his_con.name == &contest.name {
                return HttpResponse::BadRequest().json(HTTPerror::new(
                    1,
                    "ERR_INVALID_ARGUMENT".to_string(),
                    format!("Contest name '{}' already exists.", contest.name),
                ));
            }
        }
        for his_con in contests.iter_mut() {
            if &his_con.id.unwrap() == &contest.id.unwrap() {
                log::info!(
                    "Successfully change Contest {} 's name to {}",
                    contest.id.unwrap(),
                    contest.name
                );
                flag = true;
                his_con.name = contest.name.clone();
                return HttpResponse::Ok().json(his_con);
            }
        }
        if !flag {
            return HttpResponse::NotFound().json(HTTPerror::new(
                3,
                "ERR_NOT_FOUND".to_string(),
                format!("Contest {} not found.", contest.id.unwrap()),
            ));
        }
    }
    HttpResponse::Ok().json(contests.last().unwrap())
}
async fn get_ranklist() -> HttpResponse {
    let users = USER_LIST.lock().unwrap();
    let mut ranklist: Vec<_> = users.iter().collect();
    // ranklist.sort_by(|a, b| b.score.cmp(&a.score));
    HttpResponse::Ok().json(&ranklist)
}
async fn get_contest(contest: web::Json<Contest>) -> HttpResponse {
    let vc:Vec<Contest>=CONTEST_LIST.lock().unwrap().to_vec();
HttpResponse::Ok().json(vc)
}
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/contests/0/ranklist").route(web::get().to(get_ranklist)));
    cfg.service(
        web::resource("/contests")
            .route(web::post().to(post_contest))
            .route(web::get().to(get_contest)),
    );
}
