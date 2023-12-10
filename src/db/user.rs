use crate::session::{Session, SessionError};

use argon2::{
    self,
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request, State,
};
use sqlx::PgPool;

/// A hashed password
pub struct HashedPassword(String);

/// Represents all the information about a user
pub struct User {
    /// id row in the database
    pub id: i32,
    // The email associated with the account
    pub email: String,
    // Hashed password, stored as a string
    password: HashedPassword,
}

impl User {
    /// Constructs a user with the provided details.
    ///
    /// ### Arguments
    /// * `id` - the id of the user
    /// * `email` - the user's email
    /// * `password` - the user's plaintext password
    ///
    /// ### Returns
    /// A user record
    pub fn new(id: i32, email: String, password: HashedPassword) -> User {
        User {
            id,
            email,
            password,
        }
    }

    /// Gets the user with the given id from the database
    ///
    /// ### Arguments
    ///
    /// * `conn` - a connection to the database containing the user
    /// * `id` - the id of the given user
    ///
    /// ### Returns
    ///
    /// A result of Error on failure to access the database, otherwise Ok
    /// containing either the User if we found one, or None if no such user exists
    pub async fn get_by_id(
        conn: &mut sqlx::PgConnection,
        id: i32,
    ) -> Result<Option<User>, sqlx::Error> {
        // Grab the user from the database
        let user = sqlx::query!("SELECT id, email, password FROM users WHERE id = $1", id)
            .fetch_optional(conn)
            .await?;

        // Convert the fetched user into a User struct
        Ok(match user {
            Some(user) => Some(User::new(
                user.id,
                user.email,
                HashedPassword(user.password),
            )),
            None => None,
        })
    }

    /// Gets a user with the given email
    ///
    /// ### Arguments
    ///
    /// * `conn` - A connection to the database containing the user
    /// * `email` - The email of the user we're obtaining
    ///
    /// ### Returns
    ///
    /// A result of Error on failure to access the database, otherwise Ok
    /// containing either the User if we found one, or None if no such user exists
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
            Some(user) => Some(User::new(
                user.id,
                user.email,
                HashedPassword(user.password),
            )),
        })
    }

    /// Verify that the provided password matches our stored hashed password
    ///
    /// ### Arguments
    ///
    /// * `password` - the plaintext password we're verifying
    ///
    /// ### Returns
    ///
    /// true if the provided password matches our stored one, false otherwise
    pub fn verify_password(&self, password: &str) -> bool {
        // Get our stored password into an internal format we can use
        let password_hash = match argon2::PasswordHash::new(&self.password.0) {
            Ok(password_hash) => password_hash,
            Err(_) => return false,
        };

        // Compare the two, do they match?
        let password = password.as_bytes();
        Argon2::default()
            .verify_password(password, &password_hash)
            .is_ok()
    }

    /// Checks if the given email is already taken (if a user with the email exists)
    ///
    /// ### Arguments
    ///
    /// * `conn` - A connection to the database storing our users
    /// * `email` - Check if there's an associated user with this email
    ///
    /// ### Returns
    ///
    /// Error if we failed to access the database, or a `true` if the email is taken, `false` otherwise
    pub async fn email_taken(
        conn: &mut sqlx::PgConnection,
        email: &str,
    ) -> Result<bool, sqlx::Error> {
        let record = sqlx::query!("SELECT id FROM users WHERE email = $1", email)
            .fetch_optional(conn)
            .await?;

        Ok(match record {
            Some(_) => true,
            None => false,
        })
    }

    /// Attempts to creates a new user with the provided details
    ///
    /// ### Arguments
    ///
    /// * `conn` - a connection to the database where we want to store the new user
    /// * `email` - the email of the new user
    /// * `password` - the plaintext password for the new user (we'll hash it here)
    ///
    /// ### Returns
    ///
    /// Error if we failed to access the database or Ok(true if we created the user, false if we failed to create the user)
    pub async fn create(
        conn: &mut sqlx::PgConnection,
        email: &str,
        password: &str,
    ) -> Result<bool, sqlx::Error> {
        let password = match Self::hash_password(password).await {
            Ok(password) => password,
            Err(_) => return Ok(false),
        };

        let res = sqlx::query!(
            "INSERT INTO users (email, password) VALUES ($1, $2)",
            email,
            password.0
        )
        .execute(conn)
        .await?;

        // did we effect any rows? (was the user actually created?)
        return Ok(res.rows_affected() != 0);
    }

    /// Hashes the password into a hashed password string
    ///
    /// ### Arguments
    ///
    /// * `password` - the plaintext user password
    ///
    /// ### Returns
    ///
    /// Error if something drastic went wrong, or the hashed password
    async fn hash_password(password: &str) -> Result<HashedPassword, argon2::password_hash::Error> {
        let password = password.as_bytes();
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hashed_password = argon2.hash_password(password, &salt)?;

        Ok(HashedPassword(hashed_password.to_string()))
    }
}

/// Errors that can occur when grabbing the user out of a request
#[derive(Debug)]
pub enum UserError {
    NotFound,
    ServerError,
}

#[async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = UserError;

    /// Get the user making the request
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
        let pool: &State<PgPool> = match req.guard().await {
            Outcome::Success(pool) => pool,
            Outcome::Failure(_) => {
                return Outcome::Failure((Status::InternalServerError, UserError::ServerError))
            }
            Outcome::Forward(forward) => return Outcome::Forward(forward),
        };
        let mut conn = match crate::db::acquire_conn(&pool).await {
            Ok(conn) => conn,
            Err(_) => {
                return Outcome::Failure((Status::InternalServerError, UserError::ServerError))
            }
        };

        // Grab the user, send the final outcome here
        match User::get_by_id(&mut conn, session.user_id).await {
            Ok(Some(user)) => Outcome::Success(user),
            _ => Outcome::Failure((Status::Unauthorized, UserError::NotFound)),
        }
    }
}
