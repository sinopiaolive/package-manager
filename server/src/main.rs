#![warn(clippy::all)]
#![recursion_limit = "128"]
#![feature(proc_macro_hygiene, decl_macro)] // for rocket

#[macro_use]
extern crate rocket;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rmp_serde;
#[macro_use]
extern crate rocket_contrib;
extern crate failure;
extern crate failure_derive;
#[macro_use]
extern crate quick_error;
extern crate data_encoding;
extern crate dotenv;
extern crate reqwest;
extern crate url;
#[macro_use]
extern crate diesel;
extern crate brotli;
extern crate im;
extern crate pm_lib;
extern crate tar;

mod auth;
mod error;
mod file;
mod github;
mod gitlab;
mod index;
mod package;
mod schema;
mod search;
mod store;
mod upload;
mod user;

use std::collections::HashMap;
use std::env;
use std::io::Cursor;

use rocket::config::{Config, Environment, Value};
use rocket::http::{ContentType, Status};
use rocket::request::{Form, FromRequest, Request};
use rocket::response::{content, Redirect, Response};
use rocket::{Data, Outcome};
use rocket_contrib::json::Json;

use url::Url;

use auth::{AuthProvider, AuthToken};
use error::{Error, Res};
use github::{Github, GITHUB_CLIENT_ID};
use gitlab::{Gitlab, GITLAB_CLIENT_ID};
use store::Store;

static STYLES: &'static str = "
body {
    background: white;
    color: black;
    margin: 0 4em;
    padding: 0;
    text-align: center;
    font-size: 1em;
    font-family: serif;
}

h1 {
    background: red;
    color: yellow;
    width: 100%;
    padding: 0.5em 0;
}

.btn {
    background: #3498db;
    background-image: -webkit-linear-gradient(top, #3498db, #2980b9);
    background-image: -moz-linear-gradient(top, #3498db, #2980b9);
    background-image: -ms-linear-gradient(top, #3498db, #2980b9);
    background-image: -o-linear-gradient(top, #3498db, #2980b9);
    background-image: linear-gradient(to bottom, #3498db, #2980b9);
    -webkit-border-radius: 4;
    -moz-border-radius: 4;
    border-radius: 4px;
    font-family: sans-serif;
    color: #ffffff;
    font-size: 20px;
    padding: 10px 20px 10px 20px;
    text-decoration: none;
}

.btn:hover {
    background: #3cb0fd;
    background-image: -webkit-linear-gradient(top, #3cb0fd, #3498db);
    background-image: -moz-linear-gradient(top, #3cb0fd, #3498db);
    background-image: -ms-linear-gradient(top, #3cb0fd, #3498db);
    background-image: -o-linear-gradient(top, #3cb0fd, #3498db);
    background-image: linear-gradient(to bottom, #3cb0fd, #3498db);
    text-decoration: none;
}

.pad { padding: 1em; }
";

fn html_doc(content: &str) -> content::Html<String> {
    content::Html(format!(
        "<!doctype html>
<html>
  <head>
    <style>{}</style>
  </head>
  <body>
    <h1>☭ People's Revolutionary Package Registry ☭</h1>
    {}
  </body>
</html>
",
        STYLES, content
    ))
}

fn parse_auth_header(header: &str) -> Option<&str> {
    let start = "Bearer ";
    if header.starts_with(start) {
        Some(&header[start.len()..])
    } else {
        None
    }
}

pub struct Authenticate(AuthToken);

impl<'a, 'r> FromRequest<'a, 'r> for Authenticate {
    type Error = Error;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, (Status, Self::Error), ()> {
        match request
            .headers()
            .get_one("Authorization")
            .and_then(parse_auth_header)
        {
            None => Outcome::Failure((Status::Unauthorized, Error::Status(Status::Unauthorized))),
            Some(token) => match AuthToken::decode(token.as_bytes()) {
                Ok(token) => Outcome::Success(Authenticate(token)),
                Err(err) => Outcome::Failure((Status::Unauthorized, err)),
            },
        }
    }
}

impl Authenticate {
    pub fn validate(&self, store: &Store) -> Res<AuthToken> {
        match self.0.user.provider.provider() {
            Err(_) => Err(Error::Status(Status::Unauthorized)),
            Ok(provider) => match provider.user(&self.0.token) {
                Err(_) => Err(Error::Status(Status::Unauthorized)),
                Ok(user) => {
                    if user.user()? == self.0.user {
                        store.update_user(&user)?;
                        Ok(self.0.clone())
                    } else {
                        Err(Error::Status(Status::Unauthorized))
                    }
                }
            },
        }
    }
}

#[get("/test")]
fn test(auth: Authenticate, store: Store) -> Res<String> {
    auth.validate(&store)?;
    Ok("Hello Joe".to_string())
}

#[derive(FromForm)]
struct SearchQuery {
    ns: String,
    q: String,
}

#[get("/files/tar-br/<namespace>/<name>/<version>")]
fn files(store: Store, namespace: String, name: String, version: String) -> Res<Response<'static>> {
    match store.get_tar_br(&namespace, &name, &version) {
        Err(_) => Err(Error::Status(Status::NotFound)),
        Ok(tar_br) => Response::build()
            .status(Status::Ok)
            .header(ContentType::new("application", "brotli"))
            .sized_body(Cursor::new(tar_br))
            .ok(),
    }
}

#[get("/search?<query..>")]
fn search(query: Form<SearchQuery>, store: Store) -> Res<Json<Vec<search::SearchResult>>> {
    Ok(Json(search::search(
        &store,
        &query.ns,
        query.q.split_whitespace().map(str::to_string).collect(),
    )?))
}

#[post("/publish", data = "<data>")]
fn publish(
    data: Data, auth: Authenticate, store: Store) -> Res<Json<()>> {
    let token = auth.validate(&store)?;
    Ok(Json(upload::process_upload(
        &store,
        &token.user,
        data.open(),
    )?))
}

#[get("/")]
fn index() -> Res<content::Html<String>> {
    Ok(html_doc(
        "
<p class=\"pad\">
  <a class=\"btn\" href=\"/login\">Log in?</a>
</p>
",
    ))
}

#[derive(FromForm)]
struct Login {
    token: String,
    callback: String,
}

#[get("/login_client?<login..>")]
fn login_client(store: Store, login: Form<Login>) -> Res<content::Html<String>> {
    store.register_login(&login.token, &login.callback)?;
    let github_url = format!(
        "https://github.com/login/oauth/authorize?scope=user:email&client_id={}&state={}",
        GITHUB_CLIENT_ID, login.token
    );
    let gitlab_url = format!(
        "https://gitlab.com/oauth/authorize?client_id={}&state={}&response_type=code&redirect_uri=http://localhost:8000/gitlab/callback&scope=read_user",
        GITLAB_CLIENT_ID,
        login.token
    );
    Ok(html_doc(&format!(
        "
<p>Use this decadent bourgeois identity provider to log in:</p>
<p class=\"pad\">
  <a class=\"btn\" href=\"{}\">Log in with GitHub</a>
</p>
<p>Or choose a service provided under the Glorious People's Licence:</p>
<p>
  <a class=\"btn\" href=\"{}\">Log in with GitLab</a>
</p>
",
        github_url, gitlab_url
    )))
}

#[derive(FromForm)]
struct OAuthCallback {
    code: String,
    state: String,
}

#[get("/github/callback?<callback..>")]
fn github_callback(store: Store, callback: Form<OAuthCallback>) -> Res<Redirect> {
    let mut redirect = Url::parse(&store.validate_login(&callback.state)?)?;
    let github = Github::new()?;
    let token = github.validate_callback(&callback.code)?;
    let user = github.user(&token.access_token)?;
    let auth = AuthToken::new(&user.user()?, &token.access_token);
    println!("User data: {:?}", user);
    store.update_user(&user)?;
    redirect
        .query_pairs_mut()
        .append_pair("token", &auth.encode()?)
        .append_pair("state", &callback.state);
    Ok(Redirect::to(redirect.as_str().to_string()))
}

#[get("/gitlab/callback?<callback..>")]
fn gitlab_callback(store: Store, callback: Form<OAuthCallback>) -> Res<Redirect> {
    let mut redirect = Url::parse(&store.validate_login(&callback.state)?)?;
    let gitlab = Gitlab::new()?;
    let token = gitlab.validate_callback(&callback.code)?;
    let user = gitlab.user(&token.access_token)?;
    let auth = AuthToken::new(&user.user()?, &token.access_token);
    println!("User data: {:?}", user);
    store.update_user(&user)?;
    redirect
        .query_pairs_mut()
        .append_pair("token", &auth.encode()?)
        .append_pair("state", &callback.state);
    Ok(Redirect::to(redirect.as_str().to_string()))
}

fn main() {
    #[cfg(not(test))]
    dotenv::dotenv().ok();

    let mut database_config = HashMap::new();
    let mut databases = HashMap::new();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL environment variable not set");
    database_config.insert("url", Value::from(database_url));
    databases.insert("registry", Value::from(database_config));
    let config = Config::build(Environment::Development)
        .extra("databases", databases)
        //.log_level(::rocket::config::LoggingLevel::Debug)
        .finalize()
        .unwrap();

    rocket::custom(config)
        .attach(Store::fairing())
        .mount(
            "/",
            routes![
                index,
                test,
                search,
                publish,
                files,
                login_client,
                github_callback,
                gitlab_callback,
            ],
        )
        .launch();
}
