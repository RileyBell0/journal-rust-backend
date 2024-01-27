use rocket::{fairing::AdHoc, Build, Rocket};

pub mod account;
pub mod auth;
pub mod images;
pub mod notes;

pub fn launch() -> Rocket<Build> {
    // A fairing to connect us to the database
    let connect_to_db = AdHoc::try_on_ignite("Connect to DB", |rocket| {
        Box::pin(async {
            let vars = env_file_reader::read_file(".env").expect("Failed to find/parse env file");
            let db_url: &str = &vars["DATABASE_URL"];

            // Connect to the database
            let pool = sqlx::Pool::<sqlx::Postgres>::connect(db_url)
                .await
                .expect("Failed to connect to the DB");

            // Hand off our pool to Rocket
            Ok(rocket.manage(pool))
        })
    });

    rocket::build()
        .attach(connect_to_db)
        .mount("/api", routes![account::signup,])
        .mount(
            "/api/notes",
            routes![
                notes::create,
                notes::get,
                notes::get_many,
                notes::get_overview,
                notes::get_overview_many,
                notes::update,
                notes::delete,
                notes::get_diary_many
            ],
        )
        .mount("/api/images", routes![images::upload, images::get])
        .mount("/api/auth", routes![auth::login, auth::check, auth::logout])
}
