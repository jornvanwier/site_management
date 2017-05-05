use super::models::{Session, User};

use rocket::Outcome::{self, Failure, Success};
use rocket::http::Status;
use rocket::request::{self, Request, FromRequest, State};
use super::ConnectionPool;
use diesel::prelude::*;

use SESSION_COOKIE;

pub struct UserLogin {
    pub session: Session,
    pub user: User,
}

impl<'a, 'r> FromRequest<'a, 'r> for UserLogin {
    type Error = String;

    fn from_request(req: &Request) -> request::Outcome<Self, String> {
        if let Some(session_key) =
            req.cookies()
                .find(SESSION_COOKIE)
                .map(|c| c.value().to_string()) {
            if let Outcome::Success(pool) =
                <State<ConnectionPool> as FromRequest>::from_request(req) {
                let conn = pool.get().unwrap();
                use schema::sessions;
                if let Ok(session) = sessions::table
                       .filter(sessions::key.eq(session_key))
                       .first::<Session>(&*conn) {
                    if let Ok(user) = session.user(&*conn) {
                        return Success(UserLogin { session, user });
                    }
                }

            }

        }
        Failure((Status::Unauthorized, "Couldn't parse session key".to_string()))
    }
}

