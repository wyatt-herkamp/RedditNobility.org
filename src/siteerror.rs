use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

use actix_web::{dev::HttpResponseBuilder, error, get, http::header, http::StatusCode, App, HttpResponse, HttpServer, web};
use derive_more::{Display, Error};
use serde_json;
use error::ResponseError;
use log::{error, info, warn};
use serde_json::Value;
use std::collections::HashMap;
use tera::Tera;
use crate::websiteerror::WebsiteError;

/// Error type that occurs when an API request fails for some reason.
#[derive(Debug, Display, Error)]
pub enum SiteError {
    #[display(fmt = "An internal error occurred. Please try again later.")]
    JSONError(serde_json::Error),
    #[display(fmt = "An internal error occurred. Please try again later.")]
    DBError(diesel::result::Error),
    #[display(fmt = "An internal error occurred. Please try again later.")]
    Other(String),
}
impl SiteError{
    fn error(&self) {
        error!("{}", self)
    }
}
impl WebsiteError for SiteError {
    fn site_error(&self, tera: web::Data<Tera>) -> HttpResponse {
        let mut ctx = tera::Context::new();
        let x = self.json_error_message();
        ctx.insert("error", x["user_message"].as_str().unwrap());
        let result = tera.get_ref().render("error.html", &ctx);
        if result.is_err() {
            let error = result.err().unwrap();
            error!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
        self.error();
        HttpResponse::Ok().status(self.status_code()).content_type("text/html").body(&result.unwrap())
    }
    fn api_error(&self) -> HttpResponse {
        self.error();

        HttpResponse::Ok().status(self.status_code()).content_type("application/json").body(serde_json::to_string(self.json_error_message()).unwrap())
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn user_message(&self) -> &str {
        match *self {
            _ => "An internal error has occurred",
        }
    }
}


impl From<diesel::result::Error> for SiteError {
    fn from(err: diesel::result::Error) -> SiteError {
        SiteError::DBError(err)
    }
}


impl From<serde_json::Error> for SiteError {
    fn from(err: serde_json::Error) -> SiteError {
        SiteError::JSONError(err)
    }
} 