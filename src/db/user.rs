use crate::session::{Session, SessionError};

use super::{Connection, Db};
use argon2::{
    self,
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use rocket_db_pools::sqlx;

/// Represents all the information about a user
pub struct User {
    /// id row in the database
    pub id: i32,
    // The email associated with the account
    pub email: String,
    // Hashed password, stored as a string
    password: String,
}

impl User {
    /// Constructs a user with the provided details.
    pub fn new(id: i32, email: String, password: String) -> User {
        User {
            id,
            email,
            password,
        }
    }

    /// Gets the user with the given id
    pub async fn get_by_id(
        conn: &mut sqlx::PgConnection,
        id: i32,
    ) -> Result<Option<User>, sqlx::Error> {
        // Grab the user from the database
        let user = sqlx::query!("SELECT id, email, password FROM users WHERE id = $1", id)
            .fetch_optional(conn)
            .await?;

        // Convert the fetched user into a User struct
        match user {
            Some(user) => Ok(Some(User::new(user.id, user.email, user.password))),
            None => Ok(None),
        }
    }

    /// Gets a user with the given email
    pub async fn get_by_email(
        conn: &mut sqlx::PgConnection,
        email: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        // Try and find a user
        let res = sqlx::query!(
            "SELECT id, email, password FROM users WHERE email = $1",
            email
        )
        .fetch_optional(conn)
        .await?;

        // Query was successful, package data
        Ok(match res {
            None => None,
            Some(user) => Some(User::new(user.id, user.email, user.password)),
        })
    }

    /// true means the passwords match
    pub fn verify_password(&self, password: &str) -> bool {
        // Get our stored password into an internal format we can use
        let password_hash = match argon2::PasswordHash::new(&self.password) {
            Ok(password_hash) => password_hash,
            Err(_) => return false,
        };

        // Turn the received password into a byte slice
        let password = password.as_bytes();

        // Compare the two, do they match?
        Argon2::default()
            .verify_password(password, &password_hash)
            .is_ok()
    }

    /// Checks if the given email is already taken (if a user with the email exists)
    pub async fn email_taken(
        conn: &mut sqlx::PgConnection,
        email: &str,
    ) -> Result<bool, sqlx::Error> {
        let record = sqlx::query!("SELECT id FROM users WHERE email = $1", email)
            .fetch_optional(conn)
            .await?;

        match record {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Creates a new user with the provided details
    pub async fn create(conn: &mut sqlx::PgConnection, email: &str, password: &str) -> bool {
        let password = match Self::hash_password(password).await {
            Ok(password) => password,
            Err(_) => return false,
        };

        sqlx::query!(
            "INSERT INTO users (email, password) VALUES ($1, $2)",
            email,
            password
        )
        .execute(conn)
        .await
        .is_ok()
    }

    // Hashes the password into a string
    async fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
        let password = password.as_bytes();
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hashed_password = argon2.hash_password(password, &salt)?;

        Ok(hashed_password.to_string())
    }
}

#[derive(Debug)]
pub enum UserError {
    NotFound,
    ServerError,
}

#[async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = UserError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<User, UserError> {
        // Grab the associated session
        let session: Session = match req.guard().await {
            Outcome::Success(session) => session,
            Outcome::Failure((_, err)) => match err {
                SessionError::NoCookie => {
                    return Outcome::Failure((Status::Unauthorized, UserError::NotFound));
                }
                SessionError::DBError => {
                    return Outcome::Failure((Status::InternalServerError, UserError::ServerError))
                }
                SessionError::NotFound => {
                    return Outcome::Failure((Status::Unauthorized, UserError::NotFound))
                }
            },
            Outcome::Forward(forward) => return Outcome::Forward(forward),
        };

        // Get a DB connection
        let mut conn: Connection<Db> = match req.guard().await {
            Outcome::Success(conn) => conn,
            Outcome::Failure(_) => {
                return Outcome::Failure((Status::InternalServerError, UserError::ServerError))
            }
            Outcome::Forward(forward) => return Outcome::Forward(forward),
        };

        // Grab the user, send the final outcome here
        match User::get_by_id(&mut conn, session.user_id).await {
            Ok(Some(user)) => Outcome::Success(user),
            _ => Outcome::Failure((Status::Unauthorized, UserError::NotFound)),
        }
    }
}
