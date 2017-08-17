#![feature(plugin, custom_derive, proc_macro, conservative_impl_trait)]
#![plugin(rocket_codegen)]
#![allow(resolve_trait_on_defaulted_unit)]

extern crate rocket;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rocket_contrib;
extern crate redis;
#[macro_use]
extern crate quick_error;
extern crate rand;
extern crate maud;
extern crate reqwest;
extern crate data_encoding;
extern crate url;

mod error;
mod user;
mod store;
mod auth;
mod github;
mod gitlab;

use rocket::request::{Request, FromRequest};
use rocket::response::Redirect;
use rocket::{Outcome, State};
use rocket::http::Status;

use maud::{html, Markup, PreEscaped};

use url::Url;

use error::{Res, Error};
use store::Store;
use github::{Github, GITHUB_CLIENT_ID};
use gitlab::{Gitlab, GITLAB_CLIENT_ID};
use auth::{AuthProvider, AuthToken};



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

fn html_doc(content: Markup) -> Markup {
    html! {
        (PreEscaped("<!doctype html>"));
        html {
            head {
                style { (STYLES) }
            }
            body {
                h1 { "☭ People's Revolutionary Package Registry ☭" }
                (content)
            }
        }
    }
}



fn parse_auth_header<'a>(header: &'a str) -> Option<&'a str> {
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
        match request.headers().get_one("Authorization").and_then(
            parse_auth_header,
        ) {
            None => Outcome::Failure((Status::Unauthorized, Error::Status(Status::Unauthorized))),
            Some(token) => {
                match AuthToken::decode(token.as_bytes()) {
                    Ok(token) => Outcome::Success(Authenticate(token)),
                    Err(err) => Outcome::Failure((Status::Unauthorized, err)),
                }
            }
        }
    }
}

impl Authenticate {
    pub fn validate(&self) -> Res<AuthToken> {
        match self.0.user.provider.provider() {
            Err(_) => Err(Error::Status(Status::Unauthorized)),
            Ok(provider) => {
                match provider.user(&self.0.token) {
                    Err(_) => Err(Error::Status(Status::Unauthorized)),
                    Ok(user) => {
                        if user == self.0.user {
                            Ok(self.0.clone())
                        } else {
                            Err(Error::Status(Status::Unauthorized))
                        }
                    }
                }
            }
        }
    }
}



#[get("/test")]
fn test(auth: Authenticate) -> Res<String> {
    auth.validate()?;
    Ok("Hello Joe".to_string())
}

#[get("/")]
fn index() -> Res<Markup> {
    Ok(html_doc(html!{
        p.pad {
            a.btn href="/login" "Log in?";
        }
    }))
}

#[derive(FromForm)]
struct Login {
    token: String,
    callback: String,
}

#[get("/login_client?<login>")]
fn login_client(store: State<Store>, login: Login) -> Res<Markup> {
    store.register_login(&login.token, &login.callback)?;
    let github_url = format!(
        "https://github.com/login/oauth/authorize?scope=user:email&client_id={}&state={}",
        GITHUB_CLIENT_ID,
        login.token
    );
    let gitlab_url = format!(
        "https://gitlab.com/oauth/authorize?client_id={}&state={}&response_type=code&redirect_uri=http://localhost:8000/gitlab/callback&scope=read_user",
        GITLAB_CLIENT_ID,
        login.token
    );
    Ok(html_doc(html!{
        p { "Use this decadent bourgeois identity provider to log in:" }
        p.pad {
            a.btn href=(github_url) "Log in with GitHub";
        }
        p { "Or choose a service provided under the Glorious People's Licence:" }
        p.pad {
            a.btn href=(gitlab_url) "Log in with GitLab";
        }
    }))
}

#[derive(FromForm)]
struct OAuthCallback {
    code: String,
    state: String,
}

#[get("/github/callback?<callback>")]
fn github_callback(store: State<Store>, callback: OAuthCallback) -> Res<Redirect> {
    let mut redirect = Url::parse(&store.validate_login(&callback.state)?)?;
    let github = Github::new()?;
    let token = github.validate_callback(&callback.code)?;
    let user = github.user(&token.access_token)?;
    let auth = AuthToken::new(&user, &token.access_token);
    println!("User data: {:?}", user);
    redirect
        .query_pairs_mut()
        .append_pair("token", &auth.encode()?)
        .append_pair("state", &callback.state);
    Ok(Redirect::to(redirect.as_str()))
}

#[get("/gitlab/callback?<callback>")]
fn gitlab_callback(store: State<Store>, callback: OAuthCallback) -> Res<Redirect> {
    let mut redirect = Url::parse(&store.validate_login(&callback.state)?)?;
    let gitlab = Gitlab::new()?;
    let token = gitlab.validate_callback(&callback.code)?;
    let user = gitlab.user(&token.access_token)?;
    let auth = AuthToken::new(&user, &token.access_token);
    println!("User data: {:?}", user);
    redirect
        .query_pairs_mut()
        .append_pair("token", &auth.encode()?)
        .append_pair("state", &callback.state);
    Ok(Redirect::to(redirect.as_str()))
}

fn main() {
    let store = Store::new().expect("couldn't connect to Redis server");
    rocket::ignite()
        .manage(store)
        .mount("/", routes![index, test, login_client, github_callback, gitlab_callback])
        .launch();
}
