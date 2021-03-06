use actix_web::http::header::HeaderMap;
use chrono::Duration;
use diesel::MysqlConnection;
use log::info;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::ops::Add;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::internal_error::InternalError;
use crate::user::action;
use crate::user::action::{
    add_new_auth_token, add_opt, get_user_by_name, get_user_from_auth_token,
};
use crate::user::models::{AuthToken, Status, User, UserPermissions, UserProperties, OTP};
use crate::utils::{get_current_time, is_valid};
use crate::Titles;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
pub fn get_user_by_header(
    header_map: &HeaderMap,
    conn: &MysqlConnection,
) -> Result<Option<User>, InternalError> {
    let option = header_map.get("Authorization");
    if option.is_none() {
        return Ok(None);
    }
    let x = option.unwrap().to_str();
    if x.is_err() {}
    let header = x.unwrap().to_string();

    let split = header.split(' ').collect::<Vec<&str>>();
    let option = split.get(0);
    if option.is_none() {
        return Ok(None);
    }
    let value = split.get(1);
    if value.is_none() {
        return Ok(None);
    }
    let value = value.unwrap().to_string();
    let key = option.unwrap().to_string();
    if key.eq("Bearer") {
        let result = get_user_from_auth_token(value, conn)?;
        return Ok(result);
    }
    Ok(None)
}

pub fn otp_expiration() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .add(Duration::hours(1).to_std().unwrap())
        .as_millis() as i64
}

pub fn generate_otp(user: &i64, conn: &MysqlConnection) -> Result<String, InternalError> {
    let value = loop {
        let opt = generate_otp_value();
        if !crate::user::action::opt_exist(&opt, conn)? {
            break opt;
        }
    };
    let opt = OTP {
        id: 0,
        user: *user,
        password: value,
        expiration: otp_expiration(),
        created: get_current_time(),
    };
    add_opt(&opt, conn)?;
    Ok(opt.password)
}
pub fn hash(password: String) -> Result<String, InternalError> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), salt.as_ref()).unwrap()
        .to_string();
    Ok(password_hash)
}

fn generate_otp_value() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect()
}

pub fn create_token(user: &User, connection: &MysqlConnection) -> Result<AuthToken, InternalError> {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(25)
        .map(char::from)
        .collect();
    let token = AuthToken {
        id: 0,
        user: user.id,
        token: s,
        created: get_current_time(),
    };
    let _result = add_new_auth_token(&token, connection);

    Ok(token)
}

pub fn quick_add(
    username: &str,
    discoverer: &str,
    conn: &MysqlConnection,
    titles: &Titles,
) -> Result<(), InternalError> {
    info!("Adding user {}", &username);

    let mut status = Status::Found;
    if username.contains("=T") {
        status = Status::Approved;
    } else if username.contains("=F") {
        status = Status::Denied
    }
    let username = username
        .replace("=T", "")
        .replace("=F", "")
        .replace("\r", "");

    if get_user_by_name(&username, conn)?.is_none() {
        let properties = UserProperties {
            avatar: None,
            description: None,
        };
        let user = User {
            id: 0,
            discord_id: 0,
            username: username.clone(),
            password: "".to_string(),
            reviewer: "".to_string(),
            status,
            status_changed: 0,
            created: get_current_time(),
            discoverer: discoverer.to_string(),
            properties,
            title: is_valid(&username, titles).unwrap_or_else(|| "No Title Identified".to_string()),
            permissions: UserPermissions {
                admin: false,
                moderator: false,
                submit: true,
                review_user: false,
                login: true,
            },
            password_changed: get_current_time(),
            birthday: None
        };
        action::add_new_user(&user, conn)?;
    }
    Ok(())
}
