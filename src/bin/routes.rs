use site_management::*;
use rocket::State;
use rocket::request::Form;
use rocket::response::NamedFile;
use rocket_contrib::Template;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct Page<'a> {
    title: &'a str
}

#[get("/<page>",rank=10)]
pub fn pages(page: &str) -> Template {
    let context = Page{title: page};
    Template::render(["pages/", page].concat(), &context)
}

#[derive(FromForm)]
pub struct Credentials {
    username: String,
    password: String
}

#[post("/auth", data="<credentials>")]
pub fn auth(credentials: Form<Credentials>, pool: State<ConnectionPool>) -> String {
    let credentials = credentials.into_inner();
    let conn = pool.get().unwrap();

    match authenticate_user(&credentials.username, &credentials.password, &conn) {
        Some(s) => s.key,
        None => "no login".to_string()
    }
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