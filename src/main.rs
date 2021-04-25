#![feature(proc_macro_hygiene, decl_macro)]

mod error;
mod session;

use crate::error::WebError;
use crate::session::Session;
use diesel::prelude::*;
use diesel_patches::models::{NewPoll, Poll, User};
use diesel_patches::schema::{polls, users};
use maud::{html, DOCTYPE};
use rocket::{
    http::{Cookie, Cookies},
    request::Form,
    response::{NamedFile, Responder},
    State,
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
fn login(
    dt: Form<LoginDto>,
    db: DPool,
    state: State<Session>,
    mut cookies: Cookies,
) -> Result<impl Responder<'static>, WebError> {
    let ld_form = dt.into_inner();
    let vals =
        users::table::filter(users::table, users::name.eq(ld_form.name)).load::<User>(&db.0)?;

    let user = vals.iter().next().ok_or(WebError::UserNotFound)?;

    if !user.verify_password(&ld_form.pass) {
        return Err(WebError::InvalidCredentials);
    }

    let session_id = state.put(user.clone());
    cookies.add(Cookie::new("login", session_id.to_string()));

    Ok(html! {
        (DOCTYPE)
        head {
            meta charset = "utf-8";
        }
        body {
            h1 {"Welcome " (user.name)}
            h2 {"Ask a question"}
            div style = "border:1px solid black;" {
                form action = "question" method = "POST" {
                    "Question" input type = "text" name = "question";
                    "Options" input type = "text" name = "options";
                    input type = "submit" value = "Ask a question";
                }
            }
        }
    })
}

#[derive(Debug, FromForm)]
pub struct QuestionDto {
    question: String,
    options: String,
}

#[post("/question", data = "<dt>")]
pub fn ask_question(
    dt: Form<QuestionDto>,
    db: DPool,
    state: State<Session>,
    cookies: Cookies,
) -> Result<impl Responder<'static>, WebError> {
    let login = cookies.get("login").ok_or(WebError::NoCookie)?.value();
    let user = state
        .get(login.parse().map_err(|_| WebError::InvalidSession)?)
        .ok_or(WebError::UserNotFound)?;

    let poll = NewPoll::new(&dt.question, &dt.options, Some(user.id));

    let added_poll = diesel::insert_into(polls::table)
        .values(&poll)
        .get_result::<Poll>(&db.0)?;
    Ok(html! {
        (DOCTYPE)
        head {
            meta charset = "utf-8";
        }
        body {
            h1 {"Interesting question " (user.name)}
            (added_poll.question) "??" br;
            (added_poll.options)
        }
    })
}

#[get("/<path..>")]
fn static_files(path: PathBuf) -> Result<impl Responder<'static>, WebError> {
    let path = PathBuf::from("site/static").join(path);
    NamedFile::open(path).map_err(|e| e.into())
}

#[database("rocket_db")]
pub struct DPool(diesel::pg::PgConnection);

fn main() {
    let sesh = Session::new();

    rocket::ignite()
        .mount(
            "/",
            routes![healthz, root, login, ask_question, static_files],
        )
        .attach(DPool::fairing())
        .manage(sesh)
        .launch();
}
