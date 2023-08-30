#[macro_use]
extern crate rocket;

use rust_back::launch;

#[launch]
fn rocket() -> _ {
    launch()
}
