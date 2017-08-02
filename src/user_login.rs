use super::models::{Session, User};

use rocket::Outcome::{Failure, Success};
use rocket::http::Status;
use rocket::request::{self, Request, FromRequest};
use diesel::pg::PgConnection;
use diesel::prelude::*;

use connection_from_pool::ConnectionFromPool;
use SESSION_COOKIE;

#[derive(Debug)]
pub struct UserLogin {
    pub session: Session,
    pub user: User,
}

impl UserLogin {
    pub fn from_key(key: String, conn: &ConnectionFromPool) -> request::Outcome<Self, String> {
        use schema::sessions;
        if let Ok(session) = sessions::table
            .filter(sessions::key.eq(key))
            .first::<Session>(&**conn)
        {
            if let Ok(user) = session.user(&*conn) {
                return Success(UserLogin { session, user });
            }
        }
        Failure((
            Status::Unauthorized,
            "Couldn't parse session key".to_string(),
        ))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for UserLogin {
    type Error = String;

    fn from_request(req: &Request) -> request::Outcome<Self, String> {
        if let Some(session_key) =
            req.cookies().get(SESSION_COOKIE).map(
                |c| c.value().to_string(),
            )
        {
            if let Success(connection) = ConnectionFromPool::from_request(req) {
                return UserLogin::from_key(session_key, &connection);
            }
        }

        Failure((
            Status::Unauthorized,
            "Couldn't parse session key".to_string(),
        ))
    }
}
