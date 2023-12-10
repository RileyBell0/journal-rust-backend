#[macro_use]
extern crate rocket;

#[launch]
async fn rocket() -> _ {
    rust_back::launch()
}
