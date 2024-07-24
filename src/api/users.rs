use crate::models::{HTTPerror, User, CONTEST_LIST, USER_LIST};
use crate::{save_contests, save_users};
use actix_web::{web, HttpResponse};

// Handler for getting the list of users
async fn get_users() -> HttpResponse {
    log::info!("Provide users as requested.");
    let users = USER_LIST.lock().unwrap();
    HttpResponse::Ok().json(&*users)
}

// Handler for adding a new user or updating an existing one
async fn post_user(new_user: web::Json<User>) -> HttpResponse {
    let mut users = USER_LIST.lock().unwrap();
    let mut user = new_user.into_inner();

    // Automatically generate a new unique ID if not provided
    if user.id.is_none() {
        // Check if the user name already exists
        if users.iter().any(|u| u.name == user.name) {
            return HttpResponse::BadRequest().json(HTTPerror::new(
                1,
                "ERR_INVALID_ARGUMENT".to_string(),
                format!("User name '{}' already exists.", user.name),
            ));
        }

        // Generate new ID
        let new_id = users.last().map_or(0, |last_user| last_user.id.unwrap() + 1);
        user.id = Some(new_id);
        users.push(user.clone());
        log::info!("Successfully created new user {} !!", user.name);

        // Add new user ID to the first contest's user list
        {
            let mut contests = CONTEST_LIST.lock().unwrap();
            contests[0].user_ids.push(new_id);
        }

        // Save users and contests data
        save_users().unwrap();
        save_contests().unwrap();

        return HttpResponse::Ok().json(users.last().unwrap());
    } else {
        // Check if updating an existing user
        if users.iter().any(|u| u.name == user.name && u.id != user.id) {
            return HttpResponse::BadRequest().json(HTTPerror::new(
                1,
                "ERR_INVALID_ARGUMENT".to_string(),
                format!("User name '{}' already exists.", user.name),
            ));
        }

        let mut flag = false;
        for existing_user in users.iter_mut() {
            if existing_user.id == user.id {
                log::info!(
                    "Successfully changed user {}'s name to {}",
                    user.id.unwrap(),
                    user.name
                );
                existing_user.name = user.name.clone();
                flag = true;
                break;
            }
        }

        if !flag {
            return HttpResponse::NotFound().json(HTTPerror::new(
                3,
                "ERR_NOT_FOUND".to_string(),
                format!("User {} not found.", user.id.unwrap()),
            ));
        }

        // Save users and contests data
        save_users().unwrap();
        save_contests().unwrap();

        return HttpResponse::Ok().json(user);
    }
}

// Initialize the routes for the users service
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/users")
            .route(web::get().to(get_users))
            .route(web::post().to(post_user)),
    );
}
