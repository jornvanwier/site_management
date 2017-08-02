use site_management::*;
use site_management::models::{User, Website};
use site_management::user_login::UserLogin;
use rocket::http::{Cookie, Cookies, Status};
use rocket::request::{Form, Request};
use rocket::response::{Failure, NamedFile};
use rocket::response::Redirect;
use rocket_contrib::Template;
use diesel;
use diesel::prelude::*;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use connection_from_pool::ConnectionFromPool;

#[get("/manage")]
pub fn manage_home(login: UserLogin, conn: ConnectionFromPool) -> Result<Template, Failure> {
    let websites = login
        .user
        .get_websites(&conn)
        .map_err(|_| Failure(Status::NotFound))?;

    #[derive(Debug, Serialize)]
    struct WebsiteContext {
        admin: bool,
        website_id: i32,
        website_name: String,
    }

    let mut website_contexts = Vec::new();
    for (admin, website) in websites {

        website_contexts.push(WebsiteContext {
            admin: admin,
            website_id: website.id,
            website_name: website.name,
        });
    }

    #[derive(Debug, Serialize)]
    struct SiteListContext {
        websites: Vec<WebsiteContext>,
        superadmin: bool,
    }

    let context = SiteListContext {
        websites: website_contexts,
        superadmin: login.user.superadmin,
    };

    Ok(Template::render("pages/manage_home", &context))
}

#[get("/manage/<website_id>/<page>", rank=1)]
pub fn manage_website(
    page: String,
    website_id: i32,
    login: UserLogin,
    conn: ConnectionFromPool,
) -> Result<Template, Failure> {
    if login.user.is_member_of(website_id, &conn) {
        let website_name = Website::get_by_id(website_id, &conn)
            .map_err(|_| Failure(Status::NotFound))?
            .name;

        let mut context = HashMap::new();
        context.insert(
            "title",
            [&page, " - ", login.user.username.as_ref()].concat(),
        );
        context.insert("website_id", website_id.to_string());
        context.insert("website_name", website_name);
        context.insert("session_key", login.session.key);
        Ok(Template::render("pages/manage_website", &context))
    } else {
        Err(Failure(Status::Forbidden))
    }
}

#[get("/manage/<website_id>/admin")]
pub fn manage_website_admin(
    website_id: i32,
    login: UserLogin,
    conn: ConnectionFromPool,
) -> Result<Template, Failure> {
    // Check if user is authorized to edit this website's users
    if login.user.is_admin_of(website_id, &conn) {
        let website = Website::get_by_id(website_id, &conn)
            .map_err(|_| Failure(Status::InternalServerError))?;

        #[derive(Debug, Serialize)]
        struct UserListing {
            admin: bool,
            name: String,
            id: i32
        }
        let users = website
            .get_users(&conn)
            .map_err(|_| Failure(Status::InternalServerError))?
            .into_iter()
            .map(|(admin, user)| {
                UserListing {
                    admin: admin,
                    name: user.username,
                    id: user.id
                }
            })
            .collect();

        #[derive(Debug, Serialize)]
        struct PageContext {
            user_listings: Vec<UserListing>,
            title: String,
            website_id: i32
        }

        Ok(Template::render(
            "pages/manage_website_admin",
            &PageContext {
                user_listings: users,
                title: website.name,
                website_id: website_id
            },
        ))
    } else {
        Err(Failure(Status::Forbidden))
    }
}

#[get("/manage/new_website")]
pub fn new_website(login: UserLogin) -> Result<Template, Failure> {
    if !login.user.superadmin {
        return Err(Failure(Status::Forbidden));
    }

    Ok(Template::render("pages/new_website", String::new()))
}

#[derive(FromForm)]
pub struct NewUserForm {
    submit: String,
    name: String,
    admin: bool
}

#[post("/manage/<website_id>/admin/new_user", data = "<form>")]
pub fn manage_website_new_user(website_id: i32, form: Form<NewUserForm>, login: UserLogin, conn: ConnectionFromPool) -> Result<Redirect, Failure> {
    if login.user.is_admin_of(website_id, &conn) {
        if let Ok(website) = Website::get_by_id(website_id, &conn) {
            if let Ok(user) = User::get_by_name(&form.get().name, &conn) {
                if website.add_user(&user, form.get().admin, &conn).is_err() {
                    return Err(Failure(Status::InternalServerError))
                }

                Ok(Redirect::to(&format!("/manage/{website_id}/admin", website_id = website_id)))
            }
            else {
                Ok(Redirect::to(&format!("/manage/{website_id}/admin?wrong_user", website_id = website_id)))
            }
        }
        else {
            Err(Failure(Status::InternalServerError))
        }
    }
    else {
        Err(Failure(Status::Forbidden))
    }
}

#[derive(FromForm)]
pub struct FieldSubmitForm {
    submit: String,
    value: String
}

#[post("/manage/new_website", data = "<form>")]
pub fn new_website_submit(
    form: Form<FieldSubmitForm>,
    login: UserLogin,
    conn: ConnectionFromPool,
) -> Result<Redirect, Failure> {
    if !login.user.superadmin {
        return Err(Failure(Status::Forbidden));
    }

    if let Ok(website) = create_website(&form.get().value, &conn) {
        return Ok(Redirect::to(&format!("/manage/{website_id}/admin", website_id = website.id)))
    }
    
    Ok(Redirect::to("/manage"))
}

#[derive(FromForm)]
pub struct LoginRedirectReason {
    reason: Option<String>,
}

impl<'a> LoginRedirectReason {
    pub fn empty() -> Self {
        Self { reason: None }
    }

    pub fn reason_desc(&self) -> Option<&'a str> {
        if let Some(ref reason) = self.reason {
            return match reason.as_str() {
                "failed" => Some("You entered the wrong username or password"),
                "unauthorized" => Some("You need to be logged in to view this page"),
                _ => None,

            };
        };
        None
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

#[post("/auth", data = "<credentials>")]
pub fn auth(
    credentials: Form<Credentials>,
    mut cookies: Cookies,
    conn: ConnectionFromPool,
) -> Result<Redirect, Failure> {
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
pub fn logout(
    login: UserLogin,
    mut cookies: Cookies,
    conn: ConnectionFromPool,
) -> Result<Redirect, Failure> {

    cookies.remove(Cookie::named(SESSION_COOKIE));

    use schema::sessions;
    match diesel::delete(sessions::table.filter(sessions::key.eq(login.session.key)))
        .execute(&*conn)
    {
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
