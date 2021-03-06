use std::thread;
use std::time::{SystemTime, Duration};
use std::sync::Arc;
use r2d2::Pool;
use r2d2_diesel::*;
use diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;

pub fn init(pool:   Arc<Pool<ConnectionManager<PgConnection>>>) {
    thread::spawn(move || loop {
                      check_expiration(&pool);
                      thread::sleep(Duration::from_secs(60));
                  });
}

fn check_expiration(pool: &Pool<ConnectionManager<PgConnection>>) {
    use schema::sessions;

    if let Ok(conn) = pool.get() {
        match diesel::delete(sessions::table.filter(sessions::expire_date.lt(SystemTime::now())))
                  .execute(&*conn) {
            Ok(n) if n > 0 => println!("Deleted {} sessions after expiration", n),
            Ok(_) => {}
            Err(e) => println!("Error deleted sessions: {}", e),
        }
    }
}

