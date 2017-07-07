#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate diesel;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde_json;
extern crate site_management;

mod session_worker;
mod routes;

use self::site_management::*;


fn main() {
    let pool = establish_connection_pool();
    session_worker::init(pool.clone());

    rocket::ignite()
        .mount("/", routes![
            routes::pages,
            routes::login,
            routes::login_reason,
            routes::logout,
            routes::auth,
            routes::javascript_files,
            routes::css_files])
        .catch(errors![
            routes::unauthorized
        ])
        .manage(pool)
        .launch();
}