use rocket::http::Status;
use sqlx::{pool::PoolConnection, PgPool, Postgres};

pub mod note;
pub mod user;

/// A single database connection that can be used for queries (pass in &mut DbConn)
pub type DbConn = PoolConnection<Postgres>;

/// Acquires a connection from the pool, or returns a Status::InternalServerError
///
/// ### Arguments
///
/// * `pool` - The pool you're hoping to retrieve a database connection from
///
/// ### Returns
///
/// A connection from the pool on success, or an error with Status::InternalServerError on failure
pub async fn acquire_conn(pool: &PgPool) -> Result<DbConn, Status> {
    match pool.acquire().await {
        Ok(conn) => Ok(conn),
        Err(_) => Err(Status::InternalServerError),
    }
}
