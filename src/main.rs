#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate bcrypt;
extern crate dotenv;
extern crate strum;
extern crate strum_macros;

use std::collections::HashMap;
use std::ops::Sub;
use std::path::Path;

use std::sync::{Arc, Mutex};
use std::thread::sleep;

use actix_cors::Cors;
use std::{env, thread};

use actix_files as fs;
use actix_files::Files;

use actix_web::web::{Form, PayloadConfig};
use actix_web::{
    get, http, middleware, post, web, App, HttpMessage, HttpRequest, HttpResponse, HttpServer,
};
use bcrypt::{hash, DEFAULT_COST};
use chrono::{DateTime, Duration, Local};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

use log::{info, warn};
use new_rawr::auth::PasswordAuthenticator;
use new_rawr::client::RedditClient;

use crate::install::install::Installed;
use new_rawr::traits::{Content, Votable};
use nitro_log::config::Config;
use nitro_log::NitroLogger;
use serde::{Deserialize, Serialize};
use crate::user::models::User;

use crate::utils::Resources;

mod admin;
mod api_response;
mod error;
mod install;
mod moderator;
mod recaptcha;
pub mod schema;
mod settings;
pub mod user;
mod utils;

type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;
pub type Database = web::Data<DbPool>;
pub type RN = web::Data<RedditRoyalty>;
pub struct RedditRoyalty {
    pub users_being_worked_on: HashMap<i64, DateTime<Local>>,
    pub reddit: RedditClient,
}

impl RedditRoyalty {
    fn new(client: RedditClient) -> RedditRoyalty {
        RedditRoyalty {
            users_being_worked_on: HashMap::new(),
            reddit: client,
        }
    }
    pub fn add_id(&mut self, id: i64) {
        self.users_being_worked_on.insert(id, Local::now());
    }
    fn remove_id(&mut self, i: &i64) {
        self.users_being_worked_on.remove(i);
    }
}

embed_migrations!();
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(error) = dotenv::dotenv() {
        println!("Unable to load dotenv {}", error);
        return Ok(());
    }
    let file = match std::env::var("MODE")
        .unwrap_or("DEBUG".to_string())
        .as_str()
    {
        "DEBUG" => "log-debug.json",
        "RELEASE" => "log-release.json",
        _ => {
            panic!("Must be Release or Debug")
        }
    };
    let config: Config = serde_json::from_str(Resources::file_get_string(file).as_str()).unwrap();
    NitroLogger::load(config, None).unwrap();
    info!("Initializing Database");
    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<MysqlConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    let connection = pool.get().unwrap();
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout()).unwrap();
    info!("Initializing Reddit Controller");

    let arc = PasswordAuthenticator::new(
        std::env::var("CLIENT_KEY").unwrap().as_str(),
        std::env::var("CLIENT_SECRET").unwrap().as_str(),
        std::env::var("REDDIT_USER").unwrap().as_str(),
        std::env::var("PASSWORD").unwrap().as_str(),
    );

    let client = RedditClient::new("RedditNobility bot(by u/KingTuxWH)", arc);
    let reddit_royalty = Arc::new(Mutex::new(RedditRoyalty::new(client)));
    let arc2 = reddit_royalty.clone();
    thread::spawn(move || loop {
        {
            let arc1 = arc2.clone();
            let result = arc1.lock();
            if result.is_err() {
                panic!("The Site Core has been poisoned. Tux you dumb fuck!")
            }
            let mut rr = result.unwrap();
            for x in rr.users_being_worked_on.clone() {
                let x1: Duration = Local::now().sub(x.1.clone());
                if x1.num_minutes() > 5 {
                    rr.remove_id(&x.0);
                }
            }
        }
        sleep(Duration::minutes(5).to_std().unwrap())
    });
    info!("Initializing Web Server");

    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin(),
            )
            .wrap(middleware::Logger::default())
            .data(pool.clone())
            .wrap(Installed)
            .data(PayloadConfig::new(1 * 1024 * 1024 * 1024))
            .configure(error::handlers::init)
            // TODO Make sure this is the correct way of handling vue and actix together. Also learn about packaging the website.
            .service(Files::new("/", std::env::var("SITE_DIR").unwrap()).show_files_listing())
    })
    .workers(2);

    // I am pretty sure this is correctly working
    // If I am correct this will only be available if the feature ssl is added
    #[cfg(feature = "ssl")]
    {
        if std::env::var("PRIVATE_KEY").is_ok() {
            use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

            let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
            builder
                .set_private_key_file(std::env::var("PRIVATE_KEY").unwrap(), SslFiletype::PEM)
                .unwrap();
            builder
                .set_certificate_chain_file(std::env::var("CERT_KEY").unwrap())
                .unwrap();
            return server
                .bind_openssl(std::env::var("ADDRESS").unwrap(), builder)?
                .run()
                .await;
        }
    }

    return server.bind(std::env::var("ADDRESS").unwrap())?.run().await;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Moderator {
    pub user: User,
    pub avatar: String,
}

#[derive(Serialize, Deserialize)]
pub struct InstallRequest {
    pub username: String,
    pub password: String,
}

#[get("/favicon.ico")]
async fn favicon() -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("site/static/favicon.ico")?)
}
