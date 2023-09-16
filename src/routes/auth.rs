use crate::{
    db::{self, user::User},
    session::Session,
};
use rocket::{
    form::Form,
    http::{CookieJar, Status},
};

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
    mut conn: db::Connection<db::Db>,
    jar: &CookieJar<'_>,
    session: Option<Session>,
) -> Status {
    // If they're currently logged in, tell them NO
    if session.is_some() {
        return Status::BadRequest;
    }

    // Find a matching user in the db
    let user = match User::get_by_email(&mut conn, &login_details.email).await {
        Ok(user) => match user {
            Some(a) => a,
            None => return Status::Unauthorized,
        },
        Err(_) => return Status::InternalServerError,
    };

    // Check they got the password right for the user associated with the email
    if !user.verify_password(&login_details.password) {
        return Status::Unauthorized;
    }

    let session = Session::init(user.id);
    session.attach(jar);
    session.save(&mut conn).await;

    Status::Ok
}

/// Logs the user out. Probably unnecessary as we should likely be able to just un-set the cookie on the client side
#[post("/logout")]
pub async fn logout(
    session: Option<Session>,
    jar: &CookieJar<'_>,
    mut conn: db::Connection<db::Db>,
) -> Status {
    match session {
        Some(session) => {
            session.remove(jar);

            // Might fail to remove the session from the db
            if !session.delete(&mut conn).await {
                return Status::InternalServerError;
            }

            Status::Ok
        }
        None => Status::BadRequest,
    }
}

/// Checks if the session cookie is valid, and therefore that the user is signed in.
#[get("/")]
pub async fn check(user: Option<User>) -> Status {
    match user {
        Some(_) => Status::Ok,
        None => Status::Unauthorized,
    }
}
