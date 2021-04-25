use failure_derive::Fail;
use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{Responder, Response, Result as RocketResult};
use std::io::Cursor;

#[derive(Debug, Fail)]
pub enum WebError {
    #[fail(display = "Could not find {}", 0)]
    IOError(std::io::Error),
    #[fail(display = "Database error: {}", 0)]
    DatabaseError(diesel::result::Error),
    #[fail(display = "User not found")]
    UserNotFound,
    #[fail(display = "Invalid credentials")]
    InvalidCredentials,
    #[fail(display = "NoCookie")]
    NoCookie,
    #[fail(display = "InvalidSession")]
    InvalidSession,
}

impl From<std::io::Error> for WebError {
    fn from(e: std::io::Error) -> Self {
        WebError::IOError(e)
    }
}

impl From<diesel::result::Error> for WebError {
    fn from(e: diesel::result::Error) -> Self {
        WebError::DatabaseError(e)
    }
}

impl<'r> Responder<'r> for WebError {
    fn respond_to(self, _: &Request) -> RocketResult<'r> {
        let res = Response::build()
            .status(Status::NotFound)
            .header(ContentType::Plain)
            .sized_body(Cursor::new(format!("Error loading page: {}", self)))
            .finalize();
        Ok(res)
    }
}
