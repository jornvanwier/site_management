extern crate site_management;

use site_management::*;
use std::sync::Arc;

#[test]
fn test_create_user() {
    let pool = Arc::new(establish_connection_pool());
    let conn = pool.get().unwrap();
    match create_user("jorn", "slurp", &conn) {
        Ok(u) =>  println!("Created user\n{:#?}", u),
        Err(e) => println!("ERROR: Failed to create user: {}", e)
    }
}

#[test]
fn test_authenticate() {
    let pool = Arc::new(establish_connection_pool());
    let conn = pool.get().unwrap();
    match authenticate_user("jorn", "slurp", &conn) {
        Some(u) => println!("Signed in as:\n{:#?}\n\n{:#?}", u, u.user(&conn).unwrap()),
        None => println!("Couldn't sign in")
    }
}
