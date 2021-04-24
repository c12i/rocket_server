use failure_derive::Fail;
use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{Responder, Response, Result as RocketResult};
use std::io::Cursor;

#[derive(Debug, Fail)]
pub enum WebError {
    #[fail(display = "Could not find {}", 0)]
    IOError(std::io::Error),
}

impl From<std::io::Error> for WebError {
    fn from(e: std::io::Error) -> Self {
        WebError::IOError(e)
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
