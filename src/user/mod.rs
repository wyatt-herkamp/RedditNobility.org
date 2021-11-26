pub mod action;
mod controllers;
mod login;
pub mod models;
pub mod utils;
use controllers::*;
use actix_web::web;
use log::debug;

pub fn init(cfg: &mut web::ServiceConfig) {
    debug!("Loading Login Controllers");
    cfg.service(login::login)
        .service(login::me)
        .service(login::one_time_password)
        .service(login::one_time_password_create);
    debug!("Loading User Controllers");
    cfg.service(change_property)
        .service(submit_user)
        .service(update_password);
}
