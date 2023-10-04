use crate::{
    db::{self, user::User},
    session::Session,
};
use rocket::{
    form::Form,
    http::{CookieJar, Status},
};

/// Information about an account required to login / sign up
#[derive(FromForm)]
pub struct SignupForm {
    // The user's email
    email: String,
    // The user's plaintext password (not yet hashed)
    password: String,
}

/// Signs up a new user.
///
/// Returns [`Status::Created`] on success (a new user is created)
///
/// # Errors
///
/// Returns [`Status::InternalServerError`] if it fails to access the database
///
/// Returns [`Status::Conflict`] if a user with that email already exists
///
#[post("/user", data = "<user_info>")]
pub async fn signup(
    user_info: Form<SignupForm>,
    mut conn: db::Connection<db::Db>,
    jar: &CookieJar<'_>,
    session: Option<Session>,
) -> Status {
    // If they're currently logged in, tell them NO
    if session.is_some() {
        return Status::BadRequest;
    }

    // Ensure we don't have an existing user with that email
    let existing_user = User::email_taken(&mut conn, &user_info.email).await;
    let existing_user = match existing_user {
        Ok(a) => a,
        Err(_) => return Status::InternalServerError,
    };
    if existing_user {
        return Status::Conflict;
    }

    // Create the user
    if !User::create(&mut conn, &user_info.email, &user_info.password).await {
        return Status::InternalServerError;
    }

    // Get the user's ID so we can make a sessino for them
    let user = User::get_by_email(&mut conn, &user_info.email).await;
    if let Ok(Some(user)) = user {
        Session::init(user.id, jar, &mut conn).await;
    }

    Status::Created
}
