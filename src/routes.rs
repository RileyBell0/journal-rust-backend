pub mod account;
pub mod auth;

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
        let response = client.get(uri!("/rust")).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), "Hello, world!");
    }
}
