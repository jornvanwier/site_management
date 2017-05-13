use super::schema::users;
use super::schema::sessions;
use super::argon2;
use super::rand_str;
use std::time::{Duration, SystemTime};
use diesel::pg::PgConnection;
use diesel::result::Error;
use diesel::prelude::*;
use SESSION_LENGTH;

#[derive(Debug, Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub salt: String,
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password: String,
    pub salt: String,
}

impl<'a> NewUser<'a> {
    pub fn new(username: &'a str, password_plain: &'a str) -> NewUser<'a> {
        let salt = rand_str(64);
        let password_hash = argon2::hash_argon2(&password_plain, &salt);
        NewUser {
            username: username,
            password: password_hash,
            salt: salt,
        }
    }
}

#[derive(Debug, Queryable)]
pub struct Session {
    pub key: String,
    user_id: i32,
    pub expire_date: SystemTime,
}

impl Session {
    pub fn user(&self, conn: &PgConnection) -> Result<User, Error> {
        use schema::users;
        let mut user = users::table
            .filter(users::id.eq(self.user_id))
            .first::<User>(conn)?;
        
        user.username = user.username.trim().to_string();
        Ok(user)
    }
}

#[derive(Insertable)]
#[table_name="sessions"]
pub struct NewSession {
    pub key: String,
    pub user_id: i32,
    pub expire_date: SystemTime,
}

impl NewSession {
    pub fn new(user_id: i32) -> NewSession {
        let key = rand_str(32);
        // Keep sessions for 48 hours
        let expire_date = SystemTime::now() + Duration::from_secs(SESSION_LENGTH);

        NewSession {
            key: key,
            user_id: user_id,
            expire_date: expire_date,
        }
    }
}

