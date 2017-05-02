#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate diesel;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate site_management;

use self::site_management::*;
use std::sync::Arc;

mod session_worker;
mod routes;

fn main() {
    let pool = Arc::new(establish_connection_pool());
    session_worker::init(pool.clone());

    rocket::ignite()
        .mount("/", routes![
            routes::pages,
            routes::auth,
            routes::javascript_files,
            routes::css_files])
        .manage(pool)
        .launch();
}