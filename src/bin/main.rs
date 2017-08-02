#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate diesel;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;
extern crate site_management;

mod session_worker;
mod routes;

use rocket_contrib::Template;
use self::site_management::*;


fn main() {
    let pool = establish_connection_pool();
    session_worker::init(pool.clone());

    rocket::ignite()
        .mount("/", routes![
            routes::manage_home,
            routes::manage_website,
            routes::manage_website_admin,
            routes::new_website,
            routes::new_website_submit,
            routes::login,
            routes::login_reason,
            routes::logout,
            routes::auth,
            routes::javascript_files,
            routes::css_files])
        .catch(errors![
            routes::unauthorized
        ])
        .attach(Template::fairing())
        .manage(pool)
        .launch();
}