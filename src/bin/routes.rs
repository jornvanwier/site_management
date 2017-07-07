use site_management::*;
use site_management::user_login::UserLogin;
use rocket::http::{Status, Cookies, Cookie};
use rocket::request::{Form, Request};
use rocket::response::{Failure, NamedFile};
use rocket::response::Redirect;
use rocket_contrib::Template;
use diesel;
use diesel::prelude::*;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use connection_from_pool::ConnectionFromPool;

#[get("/manage/<page>")]
pub fn pages(page: &str, login: UserLogin) -> Template {
    let mut context = HashMap::new();
    context.insert("title", [page, " - ", login.user.username.as_ref()].concat());
    context.insert("session_key", login.session.key);
    Template::render(["pages/", page].concat(), &context)
}

#[derive(FromForm)]
pub struct LoginRedirectReason<'a> {
    reason: Option<&'a str>,
}

impl<'a> LoginRedirectReason<'a> {
    pub fn empty() -> Self {
        Self { reason: None }
    }

    pub fn reason_desc(&self) -> Option<&'a str> {
        match self.reason {
            Some("failed") => Some("You entered the wrong username or password"),
            Some("unauthorized") => Some("You need to be logged in to view this page"),
            _ => None,
        }
    }
}

#[get("/login")]
pub fn login() -> Template {
    login_reason(LoginRedirectReason::empty())
}

#[get("/login?<reason>")]
pub fn login_reason(reason: LoginRedirectReason) -> Template {
    let mut context = HashMap::new();
    context.insert("title", "login");
    if let Some(reason) = reason.reason_desc() {
        context.insert("reason", reason);
    }
    Template::render("login", &context)
}

#[derive(FromForm)]
pub struct Credentials {
    username: String,
    password: String,
}

#[post("/auth", data="<credentials>")]
pub fn auth(credentials: Form<Credentials>,
            cookies: &Cookies,
            conn: ConnectionFromPool)
            -> Result<Redirect, Failure> {
    let credentials = credentials.into_inner();

    match authenticate_user(&credentials.username, &credentials.password, &conn) {
        Some(s) => {
            cookies.add(Cookie::new(SESSION_COOKIE, s.key));

            Ok(Redirect::to("/manage"))
        }
        None => Err(Failure(Status::Unauthorized)),
    }
}

#[get("/logout")]
pub fn logout(login: UserLogin,
              cookies: &Cookies,
              conn: ConnectionFromPool)
              -> Result<Redirect, Failure> {
    cookies.remove(SESSION_COOKIE);

    use schema::sessions;
    match diesel::delete(sessions::table.filter(sessions::key.eq(login.session.key)))
              .execute(&*conn) {
        Ok(_) => Ok(Redirect::to("/login")),
        Err(_) => Err(Failure(Status::InternalServerError)),
    }
}

#[error(401)]
fn unauthorized(req: &Request) -> Redirect {
    let uri = req.uri().path();
    if uri.starts_with("/auth") {
        return Redirect::to("/login?reason=failed");
    }

    Redirect::to("/login?reason=unauthorized")
}

// Javascript
#[get("/js/<file..>")]
fn javascript_files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("js/").join(file)).ok()
}
// CSS
#[get("/css/<file..>")]
fn css_files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("css/").join(file)).ok()
}

