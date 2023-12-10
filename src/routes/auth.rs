use crate::{
    db::{self, user::User},
    session::Session,
};
use rocket::{
    form::Form,
    http::{CookieJar, Status},
    State,
};
use sqlx::PgPool;

/// Information about an account required to login
#[derive(FromForm)]
pub struct LoginForm {
    // The user's email
    email: String,
    // The user's plaintext password (not yet hashed)
    password: String,
}

/// Attempts to login the user with the provided details
///
/// Returns [`Status::Ok`] if the login was successful
///
/// # Errors
///
/// Returns [`Status::InternalServerError`] if we couldn't connect to the database,
/// or the stored password for the user was corrput and could not be parsed
///
/// Returns [`Status::Unauthorized`] Incorrect email or password
///
/// Returns [`Status::`]
#[post("/login", data = "<login_details>")]
pub async fn login(
    login_details: Form<LoginForm>,
    pool: &State<PgPool>,
    jar: &CookieJar<'_>,
    session: Option<Session>,
) -> Result<Status, Status> {
    // If they're currently logged in, tell them NO
    if session.is_some() {
        return Err(Status::BadRequest);
    }

    // Find a matching user in the db
    let mut conn = db::acquire_conn(pool).await?;
    let user = match User::get_by_email(&mut conn, &login_details.email).await {
        Ok(user) => match user {
            Some(a) => a,
            None => return Err(Status::NotFound),
        },
        Err(_) => return Err(Status::InternalServerError),
    };

    // Check they got the password right for the user associated with the email
    if !user.verify_password(&login_details.password) {
        return Err(Status::Unauthorized);
    }

    Session::init(user.id, jar, &mut conn).await;

    Ok(Status::Ok)
}

/// Logs the user out. Probably unnecessary as we should likely be able to just un-set the cookie on the client side
#[post("/logout")]
pub async fn logout(
    session: Option<Session>,
    jar: &CookieJar<'_>,
    pool: &State<PgPool>,
) -> Result<Status, Status> {
    let mut conn = db::acquire_conn(pool).await?;
    match session {
        Some(session) => {
            // Might fail to remove the session from the db
            if !session.delete(jar, &mut conn).await {
                return Err(Status::InternalServerError);
            }

            Ok(Status::Ok)
        }
        None => Err(Status::BadRequest),
    }
}

/// Checks if the session cookie is valid, and therefore that the user is signed in.
#[get("/")]
pub async fn check(user: Option<User>) -> Result<Status, Status> {
    match user {
        Some(_) => Ok(Status::Ok),
        None => Err(Status::Unauthorized),
    }
}
