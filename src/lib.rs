#[macro_use]
extern crate rocket;

mod db;
mod routes;
mod session;

pub use routes::launch;
