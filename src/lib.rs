#[macro_use]
extern crate rocket;

mod db;
mod routes;

use db::{Database, Db}; // We need both to get the database hooked up, as Database defines the ::init method
use rocket::{Build, Rocket};

pub fn launch() -> Rocket<Build> {
    rocket::build()
        .attach(Db::init())
        .mount("/rust", routes![routes::hello])
        .mount("/rust/auth", routes![routes::auth::get_user])
}
