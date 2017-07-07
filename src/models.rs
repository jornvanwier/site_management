use super::schema::*;
use super::argon2;
use super::rand_str;
use std::time::{Duration, SystemTime};
use diesel::pg::PgConnection;
use diesel::result::Error;
use diesel::prelude::*;
use SESSION_LENGTH;

#[derive(Debug, Queryable, Identifiable, Associations)]
#[has_many(users)]
pub struct Organization {
    pub id: i32,
    pub name: String
}

impl Organization {
    fn users(&self, connection: &PgConnection) -> Result<Vec<User>, Error> {
        User::belonging_to(self).load(&*connection)
    }
}

#[derive(Insertable)]
#[table_name="organizations"]
pub struct NewOrganization<'a> {
    pub name: &'a str
}

impl<'a> NewOrganization<'a> {
    fn new(name: &'a str) -> NewOrganization {
        NewOrganization { name }
    }
}

#[derive (Debug, Queryable, Identifiable, Associations)]
#[has_many(sessions)]
#[belongs_to(Organization)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub salt: String,
    pub organization_id: i32
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

#[derive(Debug, Queryable, Associations, Identifiable)]
#[belongs_to(User)]
#[primary_key(key)]
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

