use base64::{engine::general_purpose, Engine as _};

use rocket::{
    http::{Cookie, CookieJar, Status},
    request::{FromRequest, Outcome},
    time::{Duration, OffsetDateTime},
    Request, State,
};
use sqlx::{PgConnection, PgPool};

const SESSION_COOKIE_NAME: &str = "session";
const SESSION_KEY_LEN: usize = 32;
const SESSION_DEFAULT_EXPIRY_WEEKS: i64 = 4;
const SESSION_PUBLIC_NAME: &str = "session_pub";

/// Represents a single active session
/// `key` - The key that uniquely identifies the session
/// `user_id` - The id of the logged-in user
pub struct Session {
    key: String,
    pub user_id: i32,
}

impl Session {
    /// Creates a new session
    ///
    /// ### Arguments
    ///
    /// * `user_id` - the id of the user who's signed in
    /// * `key` - the key to uniquely idenfity the session
    pub fn new(user_id: i32, key: String) -> Session {
        Session { user_id, key }
    }

    /// Initialises a new session for the given user, storing it in their jar, and in our db
    ///
    /// ### Arguments
    ///
    /// * `user_id` - The id of the user for who we're creating the session
    /// * `jar` - A reference to the cookie jar we're storing the session cookie in
    /// * `conn` - A connection to the database that stores the sessions
    pub async fn init(user_id: i32, jar: &CookieJar<'_>, conn: &mut PgConnection) -> Session {
        let session = Session::new(user_id, Self::generate_key());
        session.attach(jar);
        session.save(conn).await;

        session
    }

    /// Save the given session in the DB
    ///
    /// ### Arguments
    ///
    /// * `conn` - a connection to the DB that stores our sessions
    ///
    /// ### Returns
    ///
    /// true on success, false if we failed to save
    async fn save(&self, conn: &mut PgConnection) -> bool {
        let result = sqlx::query!(
            "INSERT INTO sessions (id, user_id) VALUES ($1, $2)",
            &self.key,
            self.user_id
        )
        .execute(conn)
        .await;

        result.is_ok()
    }

    /// Attaches this session to the given cooke jar
    ///
    /// ### Arguments
    ///
    /// * `jar` - the jar to store the cookie in
    fn attach(&self, jar: &CookieJar) {
        // Craft a cookie to store the session in
        let mut session_cookie = Cookie::new(SESSION_COOKIE_NAME, self.key.clone());
        let expiry = OffsetDateTime::now_utc() + Duration::weeks(SESSION_DEFAULT_EXPIRY_WEEKS);
        session_cookie.set_expires(expiry);

        // Chuck the session cookie into the jar
        jar.add_private(session_cookie);
        jar.add(
            Cookie::build(SESSION_PUBLIC_NAME, "authenticated")
                .http_only(false)
                .same_site(rocket::http::SameSite::Strict)
                .finish(),
        )
    }

    /// Removes the session cookie from the user's cookie jar (good for a logout route)
    ///
    /// ### Arguments
    ///
    /// * `jar` - The jar that contains the cookie
    /// * `conn` - A connection to the db storing the session
    ///
    /// ### Returns
    ///
    /// true if the session was removed from the db, false otherwise
    pub async fn delete(&self, jar: &CookieJar<'_>, conn: &mut PgConnection) -> bool {
        // Remove it from the jar
        jar.remove_private(Cookie::named(SESSION_COOKIE_NAME));
        jar.remove(Cookie::named(SESSION_PUBLIC_NAME));

        // TRY and remove it from the DB
        self.remove_from_db(conn).await
    }

    /// Deletes the session from our database
    ///
    /// ### Arguments
    ///
    /// * `conn` - A connection to the db that stores the session
    ///
    /// ### Returns
    ///
    /// true if the session was successfully deleted, false otherwise
    async fn remove_from_db(&self, conn: &mut PgConnection) -> bool {
        let result = sqlx::query!("DELETE FROM sessions WHERE id = $1", &self.key)
            .execute(conn)
            .await;

        result.is_ok()
    }

    /// Generates a randomly generated session key using like, good random generation
    ///
    /// ### Returns
    /// Returns a random string. In some rare cases this might crash the running webserver TODO
    fn generate_key() -> String {
        let mut buf = [0; SESSION_KEY_LEN];
        openssl::rand::rand_bytes(&mut buf).unwrap();
        general_purpose::STANDARD_NO_PAD.encode(buf)
    }
}

/// Stuff that can go wrong while generating a session with FromRequest
#[derive(Debug)]
pub enum SessionError {
    NoCookie,
    DBError,
    NotFound,
}

/// Allows us to grab the session of the user that's making the request
#[async_trait]
impl<'r> FromRequest<'r> for Session {
    type Error = SessionError;

    /// Gets the session for the request (validating it exists in the database)
    async fn from_request(req: &'r Request<'_>) -> Outcome<Session, SessionError> {
        // get the cookie out of the request
        let cookies = req.cookies();
        let session = match cookies.get_private(SESSION_COOKIE_NAME) {
            Some(cookie) => cookie,
            None => return Outcome::Failure((Status::Unauthorized, SessionError::NoCookie)),
        };

        // Get a DB connection
        let pool: &State<PgPool> = match req.guard().await {
            Outcome::Success(pool) => pool,
            Outcome::Failure(_) => {
                return Outcome::Failure((Status::InternalServerError, SessionError::DBError))
            }
            Outcome::Forward(forward) => return Outcome::Forward(forward),
        };
        let mut conn = match crate::db::acquire_conn(&pool).await {
            Ok(conn) => conn,
            Err(_) => {
                return Outcome::Failure((Status::InternalServerError, SessionError::DBError))
            }
        };

        // Ensure the session exists in the database
        let key = session.value();
        let session = sqlx::query!("SELECT user_id, id FROM sessions WHERE id = $1", key)
            .fetch_one(conn.as_mut())
            .await;
        match session {
            Ok(session) => Outcome::Success(Session::new(session.user_id, session.id)),
            Err(_) => Outcome::Failure((Status::Unauthorized, SessionError::NotFound)),
        }
    }
}
