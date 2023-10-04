mod account;
mod auth;
mod notes;

use crate::db::{Database, Db}; // We need both to get the database hooked up, as Database defines the ::init method
use rocket::{Build, Rocket};

pub fn launch() -> Rocket<Build> {
    rocket::build()
        .attach(Db::init())
        .mount(
            "/api",
            routes![
                hello,
                account::signup,
                notes::create,
                notes::update,
                notes::get_all,
                notes::get_one,
                notes::delete,
                notes::set_favourite
            ],
        )
        .mount("/api/auth", routes![auth::login, auth::check, auth::logout])
}

/// Example route. Used for testing connection
#[get("/")]
pub async fn hello() -> &'static str {
    "Hello rust!"
}

#[cfg(test)]
mod test {
    use crate::launch;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    //
    #[test]
    fn hello_world() {
        let client = Client::tracked(launch()).expect("valid rocket instance");
        let response = client.get(uri!("/api")).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), "Hello, world!");
    }
}
