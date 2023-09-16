use crate::db::{self, Connection, Db};
use base64::{engine::general_purpose, Engine as _};
use rocket_db_pools::sqlx;

use rocket::{
    http::{Cookie, CookieJar, Status},
    request::{FromRequest, Outcome},
    time::{Duration, OffsetDateTime},
    Request,
};

const SESSION_COOKIE_NAME: &str = "session";
const SESSION_KEY_LEN: usize = 32;
const SESSION_DEFAULT_EXPIRY_WEEKS: i64 = 4;

pub struct Session {
    key: String,
    pub user_id: i32,
}

impl Session {
    pub fn new(user_id: i32, key: String) -> Session {
        Session { user_id, key }
    }

    pub fn init(user_id: i32) -> Session {
        Session {
            user_id,
            key: Self::generate_key(),
        }
    }

    /// save the given session into the sessions table
    pub async fn save(&self, conn: &mut Connection<Db>) -> bool {
        let conn = conn.as_mut();
        let result = sqlx::query!(
            "INSERT INTO sessions (id, user_id) VALUES ($1, $2)",
            &self.key,
            self.user_id
        )
        .execute(conn)
        .await;

        result.is_ok()
    }

    /// attaches this session to the given cooke jar
    pub fn attach(&self, jar: &CookieJar) {
        let mut session_cookie = Cookie::new(SESSION_COOKIE_NAME, self.key.clone());

        let mut now = OffsetDateTime::now_utc();
        now += Duration::weeks(SESSION_DEFAULT_EXPIRY_WEEKS);

        session_cookie.set_expires(now);
        jar.add_private(session_cookie);
    }

    /// removes the session cookie (good for a logout route)
    pub fn remove(&self, jar: &CookieJar) {
        jar.remove_private(Cookie::named(SESSION_COOKIE_NAME));
    }

    /// deletes the session from the db
    pub async fn delete(&self, conn: &mut Connection<Db>) -> bool {
        let conn = conn.as_mut();
        let result = sqlx::query!("DELETE FROM sessions WHERE id = $1", &self.key)
            .execute(conn)
            .await;

        result.is_ok()
    }

    /// generates a randomly generated session key
    fn generate_key() -> String {
        let mut buf = [0; SESSION_KEY_LEN];
        openssl::rand::rand_bytes(&mut buf).unwrap();
        general_purpose::STANDARD_NO_PAD.encode(buf)
    }
}

#[derive(Debug)]
pub enum SessionError {
    NoCookie,
    DBError,
    NotFound,
}

#[async_trait]
impl<'r> FromRequest<'r> for Session {
    type Error = SessionError;

    // grabs the session from the request cookie, finds a matching one in the db
    async fn from_request(req: &'r Request<'_>) -> Outcome<Session, SessionError> {
        // get the cookie out of the request
        let cookies = req.cookies();
        let session = match cookies.get_private(SESSION_COOKIE_NAME) {
            Some(cookie) => cookie,
            None => return Outcome::Failure((Status::Unauthorized, SessionError::NoCookie)),
        };

        // we have the key now, make sure it's in the db
        let key = session.value();

        // get a db connection
        let mut conn: db::Connection<db::Db> = if let Outcome::Success(conn) = req.guard().await {
            conn
        } else {
            return Outcome::Failure((Status::InternalServerError, SessionError::DBError));
        };

        // Grab the session, return success if we found one, failure otherwise
        let session = sqlx::query!("SELECT user_id, id FROM sessions WHERE id = $1", key)
            .fetch_one(conn.as_mut())
            .await;
        match session {
            Ok(session) => Outcome::Success(Session::new(session.user_id, session.id)),
            Err(_) => Outcome::Failure((Status::Unauthorized, SessionError::NotFound)),
        }
    }
}
