use crate::{db::user::User, session::Session};
use rocket::{
    form::Form,
    http::{CookieJar, Status},
    State,
};
use sqlx::PgPool;

/// Information about an account required to login / sign up
#[derive(FromForm)]
pub struct SignupForm {
    // The user's email
    email: String,
    // The user's plaintext password (not yet hashed)
    password: String,
}

/// Signs up a new user with the provided details
///
/// ### Arguments
///
/// * `session` - The session of the currently logged in user - you're not allowed to make a new user whilst logged in
/// * `user_info` - the email and plaintext password of the new user
/// * `jar` - the jar we'll be storing the user's session in once they've been created
/// * `pool` - a pool of connections to the db we're going to store the new user
///
/// ### Returns
///
/// Status::Created on success, Status::InternalServerError if it fails to access
/// the db, or Status::Conflict if a user with the given details already exists
#[post("/user", data = "<user_info>")]
pub async fn signup(
    user_info: Form<SignupForm>,
    jar: &CookieJar<'_>,
    session: Option<Session>,
    pool: &State<PgPool>,
) -> Result<Status, Status> {
    // If they're currently logged in, tell them NO
    if session.is_some() {
        return Err(Status::BadRequest);
    }

    // Ensure we don't have an existing user with that email
    let mut conn = crate::db::acquire_conn(pool.inner()).await?;
    let existing_user = User::email_taken(&mut conn, &user_info.email).await;
    let existing_user = match existing_user {
        Ok(a) => a,
        Err(_) => return Err(Status::InternalServerError),
    };
    if existing_user {
        return Err(Status::Conflict);
    }

    // Create the user
    // if it's Err() or it's Ok(false)
    let res = User::create(&mut conn, &user_info.email, &user_info.password).await;
    if res.is_err() || res.is_ok_and(|x| x == false) {
        return Err(Status::InternalServerError);
    }

    // Get the user's ID so we can make a sessino for them
    let user = User::get_by_email(&mut conn, &user_info.email).await;
    if let Ok(Some(user)) = user {
        Session::init(user.id, jar, &mut conn).await;
    }

    Ok(Status::Created)
}
