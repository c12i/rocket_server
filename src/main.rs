#![feature(proc_macro_hygiene, decl_macro)]

use std::path::PathBuf;

use rocket::response::{NamedFile, Responder};

#[macro_use]
extern crate rocket;

#[get("/healthz")]
fn healthz() -> &'static str {
    "OK"
}

#[get("/")]
fn root() -> Result<impl Responder<'static>, failure::Error> {
    NamedFile::open("site/static/index.html").map_err(|e| e.into())
}

#[get("/<path..>")]
fn static_files(path: PathBuf) -> Result<impl Responder<'static>, failure::Error> {
    let path = PathBuf::from("site/static").join(path);
    NamedFile::open(path).map_err(|e| e.into())
}

fn main() {
    rocket::ignite()
        .mount("/", routes![healthz, root, static_files])
        .launch();
}
