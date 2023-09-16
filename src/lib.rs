#[macro_use]
extern crate rocket;

mod db;
mod routes;
mod session;

use db::{Database, Db}; // We need both to get the database hooked up, as Database defines the ::init method
use rocket::{Build, Rocket};
use routes::{account, auth};

pub fn launch() -> Rocket<Build> {
    rocket::build()
        .attach(Db::init())
        .mount("/rust", routes![routes::hello, account::signup])
        .mount(
            "/rust/auth",
            routes![auth::login, auth::check, auth::logout],
        )
}
