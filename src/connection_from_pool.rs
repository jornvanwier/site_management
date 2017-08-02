use std::ops::Deref;
use std::sync::Arc;
use r2d2::{Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;
use diesel::pg::PgConnection;
use rocket::Outcome::*;
use rocket::request::{self, State, Request, FromRequest};

type ConnectionPool = Arc<Pool<ConnectionManager<PgConnection>>>;

pub struct ConnectionFromPool(PooledConnection<ConnectionManager<PgConnection>>);

impl<'a, 'r> FromRequest<'a, 'r> for ConnectionFromPool {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<ConnectionFromPool, ()> {
        if let Success(pool) = State::<ConnectionPool>::from_request(request) {
            if let Ok(result) = pool.get() {
                return Success(ConnectionFromPool(result));
            }
        }
        Forward(())
    }
}

impl Deref for ConnectionFromPool {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
