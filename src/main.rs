#[macro_use]
extern crate rocket;

use rocket_db_pools::{Connection, Database};

#[derive(Database)]
#[database("rust")]
struct Logs(sqlx::PgPool);

#[get("/<id>")]
async fn dbtest(id: i32, mut db: Connection<Logs>) -> Option<String> {
    let rows = rocket_db_pools::sqlx::query!("SELECT * FROM account where id = $1", id)
        .fetch_all(&mut *db)
        .await;

    let rows = match rows {
        Ok(r) => r,
        _ => return Some("Failed".to_string()),
    };

    let mut info = Vec::new();

    for row in rows {
        info.push(format!("({},{})", row.id, row.name));
    }

    Some(info.join("\n"))
}

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Logs::init())
        .mount("/rust/", routes![dbtest])
        .mount("/rust/", routes![hello])
}

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    #[test]
    fn hello_world() {
        let client = Client::tracked(rocket()).expect("valid rocket instance");
        let response = client.get(uri!(super::hello)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), "Hello, world!");
    }
}
