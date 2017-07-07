#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate dotenv;
extern crate argon2rs;
extern crate rand;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;

pub mod models;
pub mod schema;
pub mod user_login;
pub mod connection_from_pool;
mod argon2;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use r2d2_diesel::*;
use diesel::result::Error;
use dotenv::dotenv;
use std::env;
use rand::{Rng};
use self::models::*;
use std::sync::Arc;

pub const SESSION_LENGTH: u64 = 60 * 60 * 48;
pub const SESSION_COOKIE: &'static str = "SESSION_KEY";

pub fn establish_connection_pool() -> Arc<r2d2::Pool<ConnectionManager<PgConnection>>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let config = r2d2::Config::default();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Arc::new(r2d2::Pool::new(config, manager).expect("Failed to create pool."))
}

pub fn create_user<'a>(name: &'a str, password: &'a str, conn: &PgConnection) -> Result<User, Error> {
    use schema::users;

    let new_user = NewUser::new(&name, &password);

    diesel::insert(&new_user).into(users::table)
        .get_result(conn)
}

pub fn authenticate_user<'a>(name: &'a str, password: &'a str, conn: &PgConnection) -> Option<Session> {
    use schema::users;

    let user = users::table.filter(users::username.eq(name))
        .limit(1)
        .first::<User>(conn);

    if let Ok(user) = user {
        if user.password == argon2::hash_argon2(&password, &user.salt) {
            create_session(&user, &conn).ok()
        } 
        else {
            None
        }
    }
    else {
        None
    }
}

pub fn create_session(user: &User, conn: &PgConnection) -> Result<Session, Error> 
{
    use schema::sessions;

    // Remove other sessions for this user
    // let num_deleted = diesel::delete(sessions::table.filter(sessions::user_id.eq(user.id)))
        // .execute(conn)?;

    let num_deleted = diesel::delete(Session::belonging_to(user)).execute(conn)?;

    if cfg!(debug_assertions) {
        println!("Deleted {} rows", num_deleted);
    }

    let new_session = NewSession::new(user.id);

    diesel::insert(&new_session).into(sessions::table)
        .get_result(conn)
}

pub fn rand_str(length: usize) -> String {
    rand::thread_rng().gen_ascii_chars().take(length).collect::<String>()
}