use crate::config::Config;
use crate::models::{
    Contest, ContestConfig, HTTPerror, JobResponse, User, UserRank, CONTEST_LIST, JOB_LIST,
    USER_LIST,
};
use crate::save_contests;
use actix_web::{web, HttpResponse};
use std::cmp::Ordering;
use std::collections::HashSet;
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
        let mut flag: bool=false;
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
    let nt = contests.last().unwrap().clone();
    drop(contests);
    {
        save_contests().unwrap();
    }
    HttpResponse::Ok().json(nt)
}
fn cmp_by_config(cf: &ContestConfig, a: &User, b: &User, contest: &Contest) -> Ordering {
    let mut score1: f64 = 0.0;
    let mut score2: f64 = 0.0;
    let mut submit_id1: Vec<usize> = Vec::new();
    let mut submit_id2: Vec<usize> = Vec::new();
    let mut sub_cnt1: u64 = 0;
    let mut sub_cnt2: u64 = 0;

    let mut joblist = Vec::new();
    {
        let jblist = JOB_LIST.lock().unwrap();
        for jb in jblist.iter() {
            if jb.submission.contest_id == contest.id.unwrap() as i32 {
                joblist.push(JobResponse::from_jobstate(jb));
            }
        }
    }
    if cf.scoring_rule.is_some() {
        if cf.scoring_rule.clone().unwrap() == "highest" {
            for pbid in contest.problem_ids.iter() {
                let mut temp_score2: f64 = 0.0;
                let mut temp_score1: f64 = 0.0;
                let mut cnt_index1: usize = 10000;
                let mut cnt_index2: usize = 10000;
                for (ind, i) in joblist.iter().enumerate() {
                    if *pbid != i.submission.problem_id as u64 {
                        continue;
                    }
                    if i.submission.user_id == a.id.unwrap() as i32 {
                        sub_cnt1 += 1;
                        if temp_score1 < i.score {
                            temp_score1 = i.score;
                            cnt_index1 = ind;
                        }
                    }
                    if i.submission.user_id == b.id.unwrap() as i32 {
                        sub_cnt2 += 1;
                        if temp_score2 < i.score {
                            temp_score2 = i.score;
                            cnt_index2 = ind;
                        }
                    }
                }
                if cnt_index1 != 10000 {
                    submit_id1.push(cnt_index1);
                }
                if cnt_index2 != 10000 {
                    submit_id2.push(cnt_index2);
                }
                score1 += temp_score1;
                score2 += temp_score2;
            }
        } else {
            for pbid in contest.problem_ids.iter() {
                let mut temp_score2: f64 = 0.0;
                let mut temp_score1: f64 = 0.0;
                let mut cnt_index1: usize = 10000;
                let mut cnt_index2: usize = 10000;
                for (ind, i) in joblist.iter().enumerate() {
                    if *pbid != i.submission.problem_id as u64 {
                        continue;
                    }
                    if i.submission.user_id == a.id.unwrap() as i32 {
                        sub_cnt1 += 1;
                        temp_score1 = i.score;
                        cnt_index1 = ind;
                    }
                    if i.submission.user_id == b.id.unwrap() as i32 {
                        sub_cnt2 += 1;
                        temp_score2 = i.score;
                        cnt_index2 = ind;
                    }
                }
                if cnt_index1 != 10000 {
                    submit_id1.push(cnt_index1);
                }
                if cnt_index2 != 10000 {
                    submit_id2.push(cnt_index2);
                }
                score1 += temp_score1;
                score2 += temp_score2;
            }
        }
    } else {
        for pbid in contest.problem_ids.iter() {
            let mut temp_score2: f64 = 0.0;
            let mut temp_score1: f64 = 0.0;
            let mut cnt_index1: usize = 10000;
            let mut cnt_index2: usize = 10000;
            for (ind, i) in joblist.iter().enumerate() {
                if *pbid != i.submission.problem_id as u64 {
                    continue;
                }
                if i.submission.user_id == a.id.unwrap() as i32 {
                    sub_cnt1 += 1;
                    temp_score1 = i.score;
                    cnt_index1 = ind;
                }
                if i.submission.user_id == b.id.unwrap() as i32 {
                    sub_cnt2 += 1;
                    temp_score2 = i.score;
                    cnt_index2 = ind;
                }
            }
            if cnt_index1 != 10000 {
                submit_id1.push(cnt_index1);
            }
            if cnt_index2 != 10000 {
                submit_id2.push(cnt_index2);
            }
            score1 += temp_score1;
            score2 += temp_score2;
        }
    }

    if score1 < score2 {
        return Ordering::Less;
    }
    if score1 > score2 {
        return Ordering::Greater;
    }
    if score1 == score2 {
        if cf.tie_breaker.is_none() {
            return Ordering::Equal;
        } else {
            let s = cf.tie_breaker.clone().unwrap();
            if s == "submission_time" {
                let  latest1: usize;
                let  latest2: usize;
                if submit_id1.len() == 0 {
                    latest1 = 10000;
                } else {
                    latest1 = submit_id1.iter().fold(0, |a, b| usize::max(a, *b));
                }
                if submit_id2.len() == 0 {
                    latest2 = 10000;
                } else {
                    latest2 = submit_id2.iter().fold(0, |a, b| usize::max(a, *b));
                }
                if latest1 < latest2 {
                    return Ordering::Greater;
                }
                if latest1 > latest2 {
                    return Ordering::Less;
                }
                if latest1 == latest2 {
                    return Ordering::Equal;
                }
            }
            if s == "submission_count" {
                if sub_cnt1 < sub_cnt2 {
                    return Ordering::Greater;
                }
                if sub_cnt1 == sub_cnt2 {
                    return Ordering::Equal;
                }
                if sub_cnt1 > sub_cnt2 {
                    return Ordering::Less;
                }
            }
            if s == "user_id" {
                if a.id < b.id {
                    return Ordering::Greater;
                }
                if a.id == b.id {
                    return Ordering::Equal;
                }
                if a.id > b.id {
                    return Ordering::Less;
                }
            }
        }
    }
    Ordering::Equal
}
fn get_score_list(cf: &ContestConfig, a: &User, contest: &Contest) -> Vec<f64> {
    let mut scores: Vec<f64> = Vec::new();

    let mut joblist = Vec::new();
    {
        let jblist = JOB_LIST.lock().unwrap();
        for jb in jblist.iter() {
            joblist.push(JobResponse::from_jobstate(jb));
        }
    }
    if cf.scoring_rule.is_some() {
        if cf.scoring_rule.clone().unwrap() == "highest" {
            for pbid in contest.problem_ids.iter() {
                let mut temp_score1: f64 = 0.0;
                for (_, i) in joblist.iter().enumerate() {
                    if *pbid != i.submission.problem_id as u64 {
                        continue;
                    }
                    if i.submission.user_id == a.id.unwrap() as i32 {
                        if temp_score1 < i.score {
                            temp_score1 = i.score;
                        }
                    }
                }
                scores.push(temp_score1);
            }
        } else {
            for pbid in contest.problem_ids.iter() {
                let mut temp_score1: f64 = 0.0;

                for (_, i) in joblist.iter().enumerate() {
                    if *pbid != i.submission.problem_id as u64 {
                        continue;
                    }
                    if i.submission.user_id == a.id.unwrap() as i32 {
                        temp_score1 = i.score;
                    }
                }
                scores.push(temp_score1);
            }
        }
    } else {
        for pbid in contest.problem_ids.iter() {
            let mut temp_score1: f64 = 0.0;
            for (_, i) in joblist.iter().enumerate() {
                if *pbid != i.submission.problem_id as u64 {
                    continue;
                }
                if i.submission.user_id == a.id.unwrap() as i32 {
                    temp_score1 = i.score;
                }
            }
            scores.push(temp_score1);
        }
    }
    scores
}
async fn get_ranklist(id: web::Path<u64>, contestcfg: web::Query<ContestConfig>) -> HttpResponse {
    let mut ranklist: Vec<_> = Vec::new();
    let mut contest: Contest;
    {
        let users = USER_LIST.lock().unwrap();
        let mut flag: bool = false;
        let contests = CONTEST_LIST.lock().unwrap();
        contest = contests.last().unwrap().clone();
        for i in contests.iter() {
            if i.id.unwrap() == *id {
                contest = i.clone();
                flag = true;
                break;
            }
        }

        if !flag {
            return HttpResponse::NotFound().json(HTTPerror::new(
                3,
                "ERR_NOT_FOUND".to_string(),
                format!("Contest {} not found.", id),
            ));
        }
        for k in contest.user_ids.iter() {
            for us in users.iter() {
                if us.id.unwrap() == *k {
                    ranklist.push(us.clone());
                }
            }
        }
    }
    ranklist.sort_by(|a, b| cmp_by_config(&contestcfg, b, a, &contest));
    let mut final_rank: Vec<UserRank> = Vec::new();
    for us in ranklist.iter() {
        let mut ur = UserRank::new();
        ur.user = us.clone();
        ur.rank = 1;
        for j in ranklist.iter() {
            if cmp_by_config(&contestcfg, us, j, &contest) == Ordering::Less {
                ur.rank += 1;
            }
        }
        ur.scores = get_score_list(&contestcfg, us, &contest);
        final_rank.push(ur.clone());
    }
    HttpResponse::Ok().json(final_rank)
}
async fn get_contest() -> HttpResponse {
    let mut vc: Vec<Contest> = CONTEST_LIST.lock().unwrap().to_vec();
    vc.remove(0);
    HttpResponse::Ok().json(vc)
}
async fn get_contest_by_id(id: web::Path<u64>) -> HttpResponse {
    let contests = CONTEST_LIST.lock().unwrap();
    let id = id.into_inner();
    if id == 0 {
        return HttpResponse::BadRequest().json(HTTPerror::new(
            1,
            "ERR_INVALID_ARGUMENT".to_string(),
            "Invalid contest id".to_string(),
        ));
    }
    let contest = contests.iter().find(|&contest| contest.id.unwrap() == id);
    if let Some(cont) = contest {
        HttpResponse::Ok().json(cont)
    } else {
        HttpResponse::NotFound().json(HTTPerror::new(
            3,
            "ERR_NOT_FOUND".to_string(),
            format!("Contest {} not found.", id),
        ))
    }
}
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/contests/{id}/ranklist").route(web::get().to(get_ranklist)));
    cfg.service(
        web::resource("/contests")
            .route(web::post().to(post_contest))
            .route(web::get().to(get_contest)),
    );
    cfg.service(web::resource("/contests/{id}").route(web::get().to(get_contest_by_id)));
}
