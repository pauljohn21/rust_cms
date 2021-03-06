use actix_web::{post, get, web, Responder, HttpResponse};
use diesel::r2d2::{self, ConnectionManager};
use actix_session::Session;


use log::{info, warn};

use common::result::AjaxResult;
use common::sign_util::blake2_sign;
use super::web_util;

use dao::{models::usermod::*, repos::userrepo};

pub type DbConnection = userrepo::DbConnection;
pub type Pool = r2d2::Pool<ConnectionManager<DbConnection>>;

#[get("/admin/test")]
async fn admin_test (session: Session) -> impl Responder {
    let username = web_util::get_username_from_session(&session).unwrap();
   format! ("Hello, {}", username )
}
#[post("/api/register")]
async fn register(
    pool: web::Data<Pool>,
    login_info: web::Json<LoginInfo>
) -> impl Responder {
        let new_login_info = NewLoginInfo {
                username: &login_info.username,
                password:  &login_info.password,
        };
        match userrepo::add_login_info(&pool.get().unwrap(), &new_login_info) {
                            Ok(info) => HttpResponse::Ok().json(AjaxResult::success_with_single(info)),
                            Err(err) =>  HttpResponse::Forbidden().json(AjaxResult::<String>::fail(err.to_string()))
                     }
}

const SESSION_USER_KEY: &str = web_util::SESSION_USER_KEY;
const SESSION_USER_KEY_SIGN: &str = web_util::SESSION_USER_KEY_SIGN;


#[post("/api/login")]
async fn login(
    session: Session, 
    pool: web::Data<Pool>,
    login_info: web::Json<LoginInfo>) -> impl Responder {

    match session.get::<String>(SESSION_USER_KEY) {
        Ok(Some(user_info)) if user_info == login_info.username => {
            info!("already logged in");
            let user_key_sign = blake2_sign(&user_info);
            match session.get::<String>(SESSION_USER_KEY_SIGN) {
                Ok(Some(user_key_sign_session)) if user_key_sign == user_key_sign_session => {
                    HttpResponse::Ok().json(AjaxResult::<bool>::success_without_data())
                }
                _ => {
                    warn!("illegal request username:{}",  login_info.username);
                    session.remove(SESSION_USER_KEY_SIGN);
                    session.remove(SESSION_USER_KEY);
                    HttpResponse::Forbidden().json(AjaxResult::<bool>::fail("Login time expired".to_string()))
                }
            }

        }
        _ => {
            info!("{} login now", login_info.username);
            
            match userrepo::valid_login_info (
                &pool.get().unwrap(),
                &login_info.username, 
                 &login_info.password
                )  {
                        true =>  {
                            let user_key_sign =  blake2_sign(&login_info.username);
                            session.set::<String>(SESSION_USER_KEY_SIGN, user_key_sign).unwrap();
                            session.set::<String>(SESSION_USER_KEY, login_info.username.clone()).unwrap();
                            HttpResponse::Ok().json(AjaxResult::<bool>::success_without_data())

                        }
                        false =>   HttpResponse::Forbidden().json(AjaxResult::<bool>::fail("password does not match username".to_string()))
            }
        }
    }
}

#[get("/api/logout")]
async fn logout(
    session: Session
) -> impl Responder {
    session.remove(SESSION_USER_KEY_SIGN);
    session.remove(SESSION_USER_KEY);
    web_util::ok_without_data()
}
