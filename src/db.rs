pub use rocket_db_pools::Connection;
pub use rocket_db_pools::Database;

#[derive(Database)]
#[database("rust")]
pub struct Db(sqlx::PgPool);
