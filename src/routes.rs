use rocket::{fairing::AdHoc, Build, Rocket};
use serde::Deserialize;
use tokio::{fs::File, io::AsyncReadExt};

pub mod account;
pub mod auth;
pub mod images;
pub mod notes;

/// Reads the requested file, returning a string of its contents
///
/// ### Arguments
///
/// * `file_path` - the path to the file you're wanting to read
///
/// ### Returns
///
/// The contents of the file on success, or an error otherwise
async fn read_file(file_path: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(file_path).await?;

    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;

    Ok(contents)
}

/// Reads the given json file
///
/// ### Arguments
///
/// * `file_path` - the path to the json file you're wanting to read
///
/// ### Returns
///
/// The parsed JSON of the relevant file
async fn read_json_file<T>(file_path: &str) -> Result<T, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    let data: T = serde_json::from_str(&read_file(file_path).await?)?;

    Ok(data)
}

/// The overall config not related to setting rocket up
/// - atm just stores our db string
#[derive(Debug, Deserialize)]
struct Config {
    db_url: String,
}

pub fn launch() -> Rocket<Build> {
    // A fairing to connect us to the database
    let connect_to_db = AdHoc::try_on_ignite("Connect to DB", |rocket| {
        Box::pin(async {
            // Grab the config (it contains our database connection url)
            let config: Config = read_json_file("config.json")
                .await
                .expect("Failed to read config.json");

            // Connect to the database
            let pool = sqlx::Pool::<sqlx::Postgres>::connect(&config.db_url)
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
