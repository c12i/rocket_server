#![feature(proc_macro_hygiene, decl_macro)]

mod error;

use crate::error::WebError;
use diesel::prelude::*;
use diesel_patches::models::User;
use diesel_patches::schema::users;
use rocket::{
    request::Form,
    response::{NamedFile, Responder},
};
use std::path::PathBuf;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_contrib;

#[get("/healthz")]
fn healthz() -> &'static str {
    "Ok"
}

#[get("/")]
fn root() -> Result<impl Responder<'static>, failure::Error> {
    NamedFile::open("site/static/index.html").map_err(|e| e.into())
}

#[derive(Debug, FromForm)]
pub struct LoginDto {
    name: String,
    pass: String,
}

#[post("/login", data = "<dt>")]
fn login(dt: Form<LoginDto>, db: DPool) -> Result<impl Responder<'static>, WebError> {
    let ld_form = dt.into_inner();
    let vals =
        users::table::filter(users::table, users::name.eq(ld_form.name)).load::<User>(&db.0)?;

    let user = vals.iter().next().ok_or(WebError::UserNotFound)?;

    if !user.verify_password(&ld_form.pass) {
        return Err(WebError::InvalidCredentials);
    }

    Ok("Password passed")
}

#[get("/<path..>")]
fn static_files(path: PathBuf) -> Result<impl Responder<'static>, WebError> {
    let path = PathBuf::from("site/static").join(path);
    NamedFile::open(path).map_err(|e| e.into())
}

#[database("rocket_db")]
pub struct DPool(diesel::pg::PgConnection);

fn main() {
    rocket::ignite()
        .mount("/", routes![healthz, root, login, static_files])
        .attach(DPool::fairing())
        .launch();
}
