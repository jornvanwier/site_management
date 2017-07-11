extern crate diesel;
extern crate site_management;

use site_management::*;
use site_management::models::*;
use site_management::schema::*;
use diesel::prelude::*;
use diesel::result::Error;

#[test]
#[ignore]
fn insert_records() {
    let pool = establish_connection_pool();

    let new_user = NewUser::new("jorn", "salade");

    let user: Result<User, Error> = diesel::insert(&new_user).into(users::table)
        .get_result(&*pool.get().unwrap());

    let new_website = NewWebsite{name: "Test Site"};

    let website: Result<Website, Error> = diesel::insert(&new_website).into(websites::table)
        .get_result(&*pool.get().unwrap());

    let user: User = user.expect("Error saving new user");
    let website: Website = website.expect("Error saving new website");

    let new_user_website = UserWebsite {
        user_id: user.id,
        website_id: website.id,
        admin: true
    };

    let user_website: UserWebsite = diesel::insert(&new_user_website).into(userwebsites::table)
        .get_result(&*pool.get().unwrap())
        .expect("Error saving new user website link");
}

#[test]
fn websites_for_user() {
    let pool = establish_connection_pool();
    let conn = &*pool.get().unwrap();

    let user: User = users::table.limit(1).load(conn).expect("Couldn't get first user").remove(0);

    let websites = user.get_websites(conn);
}