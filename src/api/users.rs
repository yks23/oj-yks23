use crate::models::{HTTPerror, User, USER_LIST,CONTEST_LIST};
use actix_web::{web, HttpResponse};
use std::sync::{Arc, Mutex};

async fn get_users() -> HttpResponse {
    log::info!("Provide users as requested.");
    let users = USER_LIST.lock().unwrap();
    HttpResponse::Ok().json(&*users)
}

async fn post_user(new_user: web::Json<User>) -> HttpResponse {
    let mut users = USER_LIST.lock().unwrap();
    let mut user = new_user.into_inner();

    // 自动生成新的唯一 id
    if user.id.is_none() {
        for his_user in users.iter() {
            if his_user.name == user.name {
                return HttpResponse::BadRequest().json(HTTPerror::new(
                    1,
                    "ERR_INVALID_ARGUMENT".to_string(),
                    format!("User name '{}' already exists.", user.name),
                ));
            }
        }
        let new_id = if let Some(last_user) = users.last() {
            last_user.id.unwrap() + 1
        } else {
            0
        };
        user.id = Some(new_id);
        users.push(user.clone());
        log::info!("{}", format!("Successfully create new user {} !!",user.name));
        {
            let mut contests=CONTEST_LIST.lock().unwrap();
            contests[0].user_ids.push(new_id);
        }
        return HttpResponse::Ok().json(users.last().unwrap());
    } else {
        let mut flag: bool = false;
        for his_user in users.iter_mut() {
            if &his_user.name == &user.name {
                return HttpResponse::BadRequest().json(HTTPerror::new(
                    1,
                    "ERR_INVALID_ARGUMENT".to_string(),
                    format!("User name '{}' already exists.", user.name),
                ));
            }
        }
        for his_user in users.iter_mut() {
            if &his_user.id.unwrap() == &user.id.unwrap() {
                log::info!("Successfully change user {} 's name to {}",user.id.unwrap(),user.name);
                flag = true;
                his_user.name = user.name.clone();
                return HttpResponse::Ok().json(his_user);
            }
        }
        if !flag {
            return HttpResponse::NotFound().json(HTTPerror::new(
                3,
                "ERR_NOT_FOUND".to_string(),
                format!("User {} not found.", user.id.unwrap()),
            ));
        }
    }
    HttpResponse::Ok().json(users.last().unwrap())
    
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/users")
            .route(web::get().to(get_users))
            .route(web::post().to(post_user)),
    );
}
