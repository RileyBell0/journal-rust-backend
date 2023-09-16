pub mod user;

pub use rocket_db_pools::Connection;
pub use rocket_db_pools::Database;

// This is our main Db (postgres)
#[derive(Database)]
#[database("rust")]
pub struct Db(sqlx::PgPool);
