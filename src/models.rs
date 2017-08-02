use super::schema::*;
use super::argon2;
use super::rand_str;
use std::time::{Duration, SystemTime};
use diesel;
use diesel::pg::PgConnection;
use diesel::result::Error;
use diesel::prelude::*;
use SESSION_LENGTH;

#[derive(Debug, Queryable, Identifiable, Associations)]
pub struct Website {
    pub id: i32,
    pub name: String,
}

impl Website {
    pub fn get_users(&self, conn: &PgConnection) -> Result<Vec<(bool, User)>, Error> {
        Ok(
            UserWebsite::belonging_to(self)
                .inner_join(users::table)
                .load(conn)?
                .into_iter()
                .map(|(uw, u)| {
                    let uw: UserWebsite = uw;
                    (uw.admin, u)
                })
                .collect(),
        )
    }

    pub fn get_by_id(id: i32, conn: &PgConnection) -> Result<Website, Error> {
        Ok(websites::table.find(id).first(conn)?)
    }

    pub fn add_user(&self, user: &User, admin: bool, conn: &PgConnection) -> Result<(), Error> {
        diesel::insert(&UserWebsite {
            user_id: user.id,
            website_id: self.id,
            admin: admin,
        }).into(userwebsites::table)
            .execute(conn)?;

        Ok(())
    }
}

#[derive(Insertable)]
#[table_name = "websites"]
pub struct NewWebsite<'a> {
    pub name: &'a str,
}

#[derive(Debug, Queryable, Identifiable, Associations)]
#[has_many(sessions)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub salt: String,
    pub superadmin: bool,
}

impl User {
    pub fn get_by_name<'a>(name: &'a str, conn: &PgConnection) -> Result<User, Error> {
        use super::schema::users::dsl::*;
        users.filter(username.eq(name)).first(conn)
    }

    pub fn get_websites(&self, conn: &PgConnection) -> Result<Vec<(bool, Website)>, Error> {
        Ok(
            UserWebsite::belonging_to(self)
                .inner_join(websites::table)
                .load(conn)?
                .into_iter()
                .map(|(uw, w)| {
                    let uw: UserWebsite = uw;
                    (uw.admin, w)
                })
                .collect(),
        )
    }

    pub fn get_website(
        &self,
        website_id: i32,
        conn: &PgConnection,
    ) -> Result<(bool, Website), Error> {
        let (uw, w): (UserWebsite, Website) = userwebsites::table
            .find((self.id, website_id))
            .inner_join(websites::table)
            .first(conn)?;

        Ok((uw.admin, w))
    }

    pub fn is_admin_of(&self, website_id: i32, conn: &PgConnection) -> bool {
        if self.superadmin {
            return true;
        }

        let result = self.get_website(website_id, conn).map(
            |(admin, _)| {
                admin
            }
        );
        if result.is_err() {
            return false;
        }

        result.unwrap()
    }

    pub fn is_member_of(&self, website_id: i32, conn: &PgConnection) -> bool {
        self.get_website(website_id, conn).is_ok()
    }
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password: String,
    pub salt: String,
}

impl<'a> NewUser<'a> {
    pub fn new(username: &'a str, password_plain: &'a str) -> NewUser<'a> {
        let salt = rand_str(64).to_uppercase();
        let password_hash = argon2::hash_argon2(&password_plain, &salt);
        NewUser {
            username: username,
            password: password_hash,
            salt: salt,
        }
    }
}

#[derive(Debug, Queryable, Identifiable, Associations, Insertable)]
#[primary_key(user_id, website_id)]
#[table_name = "userwebsites"]
#[belongs_to(User)]
#[belongs_to(Website)]
pub struct UserWebsite {
    pub user_id: i32,
    pub website_id: i32,
    pub admin: bool,
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
        Ok(users::table
            .filter(users::id.eq(self.user_id))
            .first::<User>(conn)?)
    }
}

#[derive(Insertable)]
#[table_name = "sessions"]
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

#[derive(Debug, Queryable, Identifiable, Associations)]
#[belongs_to(Website)]
#[belongs_to(User, foreign_key = "uploaded_by")]
pub struct Image {
    id: i32,
    website_id: i32,
    uploaded_by: i32,
    filename: String,
    upload_date: SystemTime,
}

#[derive(Insertable)]
#[table_name = "images"]
pub struct NewImage<'a> {
    website_id: i32,
    uploaded_by: i32,
    filename: &'a str,
    upload_date: SystemTime,
}

impl<'a> NewImage<'a> {
    pub fn new(website_id: i32, user_id: i32, filename: &'a str) -> NewImage {
        NewImage {
            website_id: website_id,
            uploaded_by: user_id,
            filename: filename,
            upload_date: SystemTime::now(),
        }
    }
}
